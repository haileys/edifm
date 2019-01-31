#[macro_use]
extern crate diesel;

extern crate chrono;
extern crate dotenv;
extern crate lame;
extern crate minimp3;
extern crate rand;

mod db;
mod icecast;
mod stream;

use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::thread;
use std::time::{Instant, Duration};

use diesel::pg::PgConnection;
use dotenv::dotenv;
use minimp3::{Decoder, Frame};

use stream::{BroadcastEncoder, BroadcastError, StreamOutput, SAMPLE_RATE};

struct Reader<T: Read> {
    epoch: Instant,
    samples: usize,
    decoder: Decoder<T>,
}

impl<T> Reader<T> where T: Read {
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
}

#[derive(Debug)]
enum Error {
    Io(io::Error),
    Broadcast(BroadcastError),
    Database(diesel::result::Error),
}

struct Station<'a> {
    conn: PgConnection,
    outputs: Vec<&'a mut StreamOutput>,
}

impl<'a> Station<'a> {
    pub fn new(outputs: Vec<&'a mut StreamOutput>) -> Self {
        Station { conn: db::connect(), outputs }
    }

    fn load_next(&self) -> Result<Option<Reader<File>>, Error> {
        let (program, recording) = match db::select_next_recording(&self.conn).map_err(Error::Database)? {
            Some(result) => result,
            None => return Ok(None),
        };

        println!("Now playing: {} - {}", recording.title, recording.artist);

        let path = PathBuf::from("catalog").join(&recording.filename);

        let file = File::open(path).map_err(Error::Io)?;

        db::insert_play_record(&self.conn, program, recording)
            .map_err(Error::Database)?;

        Ok(Some(Reader::new(Decoder::new(file))))
    }

    fn play(&mut self, mut reader: Reader<File>) -> Result<(), Error> {
        loop {
            match reader.read() {
                Ok(frame) => {
                    for output in self.outputs.iter_mut() {
                        output.write(&frame).map_err(Error::Broadcast)?;
                    }
                }
                Err(minimp3::Error::Io(e)) => return Err(Error::Io(e)),
                Err(minimp3::Error::InsufficientData) => panic!("InsufficientData"),
                Err(minimp3::Error::SkippedData) => continue,
                Err(minimp3::Error::Eof) => break,
            }
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.load_next()? {
                None => {
                    // couldn't find anything to play in the database
                    // sleep for 1 second to avoid smashing the CPU
                    thread::sleep(Duration::from_secs(1));
                }
                Some(reader) => {
                    self.play(reader)?;
                }
            }
        }
    }
}

fn main() -> Result<(), Error> {
    dotenv().ok();

    let mut hi = BroadcastEncoder::new(320,
        icecast::SourceStream::new("127.0.0.1:8000", "/live.mp3")
            .map_err(Error::Io)?);

    let mut lo = BroadcastEncoder::new(128,
        icecast::SourceStream::new("127.0.0.1:8000", "/low.mp3")
            .map_err(Error::Io)?);

    let mut station = Station::new(vec![
        &mut hi,
        &mut lo,
    ]);

    station.run()
}
