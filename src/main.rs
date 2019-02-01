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
use std::path::PathBuf;
use std::thread;
use std::time::{Instant, Duration};
use std::sync::atomic::{Ordering, AtomicBool};

use diesel::pg::PgConnection;
use dotenv::dotenv;
use minimp3::{Decoder, Frame};
use signal_hook::{iterator::Signals, SIGHUP};

use stream::{BroadcastEncoder, BroadcastError, StreamOutput, SAMPLE_RATE};

struct Reader<T: Read> {
    epoch: Instant,
    samples: usize,
    decoder: Decoder<T>,
}

impl<T> Reader<T> where T: Read + Seek {
    pub fn new(decoder: Decoder<T>) -> Self {
        Reader {
            epoch: Instant::now(),
            samples: 0,
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

        if frame.sample_rate != SAMPLE_RATE as i32 {
            // XXX: we don't support variable sample rates at the moment. that shit is just way too hard
            panic!("expected frame.sample_rate to be {}, was {}", SAMPLE_RATE, frame.sample_rate);
        }

        let until = self.epoch + Duration::from_millis(self.samples as u64 * 1_000 / SAMPLE_RATE as u64);

        self.samples += frame.data.len() / frame.channels;

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
    pub fn new(run: &'a AtomicBool, outputs: Vec<Box<StreamOutput>>) -> Self {
        Station { run, conn: db::connect(), outputs }
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

fn run_station(run: &AtomicBool) -> Result<ResumeInfo, Error> {
    let outputs = outputs().map_err(Error::Io)?;
    Station::new(run, outputs).run(None)
}

fn main() -> Result<(), Error> {
    dotenv().ok();

    let signals = Signals::new(&[SIGHUP]).expect("Signals::new");
    let run = AtomicBool::new(true);

    crossbeam::scope(|scope| {
        let station_thread = scope.spawn(|_| match run_station(&run) {
            Ok(resume_info) => resume_info,
            Err(e) => panic!("run_station: {:?}", e),
        });

        for signal in signals.forever() {
            match signal {
                SIGHUP => {
                    run.store(false, Ordering::Relaxed);
                    let resume_info = station_thread.join().expect("station_thread.join");
                    println!("resume_info: {:?}", resume_info);
                    break;
                }
                _ => {}
            }
        }
    }).expect("crossbeam::scope");

    Ok(())
}
