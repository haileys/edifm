#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate crossbeam;
extern crate dotenv;
extern crate lame;
extern crate minimp3;
extern crate rand;
extern crate signal_hook;

mod db;
mod icecast;
mod stream;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{Ordering, AtomicBool};
use std::thread;
use std::time::{Instant, Duration};

use diesel::pg::PgConnection;
use dotenv::dotenv;
use minimp3::{Decoder, Frame};
use num_rational::Ratio;
use signal_hook::{iterator::Signals, SIGHUP};

use stream::{BroadcastEncoder, BroadcastError, StreamOutput};

struct Reader<T: Read> {
    epoch: Instant,
    elapsed: Ratio<u64>,
    decoder: Decoder<T>,
}

impl<T> Reader<T> where T: Read + Seek {
    pub fn new(decoder: Decoder<T>) -> Self {
        Reader {
            epoch: Instant::now(),
            elapsed: Ratio::new(0, 1),
            decoder,
        }
    }

    pub fn read(&mut self) -> Result<Frame, minimp3::Error> {
        fn sleep_until(instant: Instant) {
            let now = Instant::now();

            if instant > now {
                thread::sleep(instant - now)
            }
        }

        let frame = self.decoder.next_frame()?;
        let elapsed_nanos = (self.elapsed * Ratio::new(1_000_000_000, 1)).to_integer();
        let until = self.epoch + Duration::from_nanos(elapsed_nanos);
        let sample_count = frame.data.len() / frame.channels;

        self.elapsed += Ratio::new(sample_count as u64, frame.sample_rate as u64);

        sleep_until(until);

        Ok(frame)
    }

    pub fn seek(&mut self, pos: u64) -> Result<(), io::Error> {
        self.decoder.reader_mut().seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    pub fn file_position(&mut self) -> Result<u64, io::Error> {
        self.decoder.reader_mut().seek(SeekFrom::Current(0))
    }
}

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Broadcast(BroadcastError),
    Database(diesel::result::Error),
}

enum PlayResult {
    Eof,
    Stopped(u64),
}

struct LoadedRecording {
    recording: db::models::Recording,
    reader: Reader<File>,
}

fn load_recording(recording: db::models::Recording) -> Result<LoadedRecording, io::Error> {
    let path = PathBuf::from("catalog").join(&recording.filename);
    let file = File::open(path)?;
    let reader = Reader::new(Decoder::new(file));

    Ok(LoadedRecording { recording, reader })
}

#[derive(Debug)]
struct ResumeInfo {
    recording: db::models::Recording,
    file_pos: u64,
}

impl ResumeInfo {
    pub fn load(self) -> Result<LoadedRecording, io::Error> {
        let mut loaded = load_recording(self.recording)?;
        loaded.reader.seek(self.file_pos)?;
        Ok(loaded)
    }
}

struct Station<'a> {
    run: &'a AtomicBool,
    conn: PgConnection,
    outputs: Vec<Box<StreamOutput>>,
}

impl<'a> Station<'a> {
    pub fn new(conn: PgConnection, run: &'a AtomicBool, outputs: Vec<Box<StreamOutput>>) -> Self {
        Station { run, conn, outputs }
    }

    fn load_next(&self) -> Result<Option<LoadedRecording>, Error> {
        let (program, recording) = match db::select_next_recording(&self.conn).map_err(Error::Database)? {
            Some(result) => result,
            None => return Ok(None),
        };

        println!("Now playing: {} - {}", recording.title, recording.artist);

        let loaded = load_recording(recording).map_err(Error::Io)?;

        db::insert_play_record(&self.conn, &program, &loaded.recording)
            .map_err(Error::Database)?;

        Ok(Some(loaded))
    }

     fn play(&mut self, mut reader: Reader<File>) -> Result<PlayResult, Error> {
        while self.run.load(Ordering::Relaxed) {
            match reader.read() {
                Ok(frame) => {
                    for output in self.outputs.iter_mut() {
                        output.write(&frame).map_err(Error::Broadcast)?;
                    }
                }
                Err(minimp3::Error::Io(e)) => return Err(Error::Io(e)),
                Err(minimp3::Error::InsufficientData) => panic!("InsufficientData"),
                Err(minimp3::Error::SkippedData) => continue,
                Err(minimp3::Error::Eof) => return Ok(PlayResult::Eof),
            }
        }

        reader.file_position()
            .map(PlayResult::Stopped)
            .map_err(Error::Io)
    }

    pub fn run(&mut self, mut resume: Option<ResumeInfo>) -> Result<ResumeInfo, Error> {
        loop {
            let next = resume.take()
                .map(|resume_info| resume_info
                    .load()
                    .map(Some)
                    .map_err(Error::Io))
                .unwrap_or_else(|| self.load_next())?;

            match next {
                None => {
                    // couldn't find anything to play in the database
                    // sleep for 1 second to avoid smashing the CPU
                    thread::sleep(Duration::from_secs(1));
                }
                Some(LoadedRecording { recording, reader }) => {
                    match self.play(reader)? {
                        PlayResult::Eof => continue,
                        PlayResult::Stopped(pos) => {
                            return Ok(ResumeInfo { recording, file_pos: pos })
                        }
                    }
                }
            }
        }
    }
}

fn outputs() -> Result<Vec<Box<StreamOutput>>, io::Error> {
    match env::var("EDIFM_TARGET").as_ref().map(String::as_str) {
        Ok("icecast") => Ok(vec![
            Box::new(
                BroadcastEncoder::new(320,
                    icecast::SourceStream::new("127.0.0.1:8000", "/live.mp3")?)
            ) as Box<StreamOutput>,

            Box::new(
                BroadcastEncoder::new(128,
                    icecast::SourceStream::new("127.0.0.1:8000", "/low.mp3")?)
            ) as Box<StreamOutput>,
        ]),
        _ => Ok(vec![
            Box::new(
                BroadcastEncoder::new(320,
                    OpenOptions::new().create(true).append(true).open("stream.mp3")?)
            ) as Box<StreamOutput>,
        ]),
    }
}

fn run_station(conn: PgConnection, run: &AtomicBool, resume: Option<ResumeInfo>) -> Result<ResumeInfo, Error> {
    let outputs = outputs().map_err(Error::Io)?;
    Station::new(conn, run, outputs).run(resume)
}

#[derive(Debug)]
enum ParseResumeInfoError {
    MalformedEnv,
    Database(diesel::result::Error),
}

fn parse_resume_info(conn: &PgConnection, resume_info_str: &str) -> Result<ResumeInfo, ParseResumeInfoError> {
    let colon_pos = resume_info_str.find(':')
        .ok_or(ParseResumeInfoError::MalformedEnv)?;

    let (recording_id_str, file_pos_str) = resume_info_str.split_at(colon_pos);

    let recording_id = recording_id_str.parse()
        .map_err(|_| ParseResumeInfoError::MalformedEnv)?;

    let file_pos = file_pos_str[1..].parse()
        .map_err(|_| ParseResumeInfoError::MalformedEnv)?;

    let recording = db::find_recording(conn, recording_id)
        .map_err(ParseResumeInfoError::Database)?;

    Ok(ResumeInfo { recording, file_pos })
}

fn main() -> Result<(), Error> {
    dotenv().ok();

    let conn = db::connect();

    let signals = Signals::new(&[SIGHUP]).expect("Signals::new");
    let run = AtomicBool::new(true);

    let resume = match env::var("EDIFM_RESUME") {
        Ok(val) => match parse_resume_info(&conn, &val) {
            Ok(resume_info) => Some(resume_info),
            Err(e) => {
                eprintln!("could not resume from EDIFM_RESUME: {:?}", e);
                None
            }
        },
        Err(_) => None,
    };

    crossbeam::scope(|scope| {
        let station_thread = scope.spawn(|_| match run_station(conn, &run, resume) {
            Ok(resume_info) => resume_info,
            Err(e) => panic!("run_station: {:?}", e),
        });

        for signal in signals.forever() {
            match signal {
                SIGHUP => {
                    println!("SIGHUP received, gracefully restarting...");

                    // tell station to stop
                    run.store(false, Ordering::Relaxed);

                    // collect resume info when station quiesces
                    let resume_info = station_thread.join().expect("station_thread.join");

                    // re-exec ourselves with the same command line we were invoked with
                    let args = std::env::args_os().collect::<Vec<_>>();
                    let err = Command::new(&args[0])
                        .env("EDIFM_RESUME", format!("{}:{}", resume_info.recording.id, resume_info.file_pos))
                        .args(&args[1..])
                        .exec();

                    panic!("could not re-exec self! {:?}", err)
                }
                _ => {}
            }
        }
    }).expect("crossbeam::scope");

    Ok(())
}
