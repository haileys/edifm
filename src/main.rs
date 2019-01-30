extern crate lame;
extern crate minimp3;
extern crate rand;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::thread;
use std::time::{Instant, Duration};

use lame::{Lame, EncodeError};
use minimp3::{Decoder, Frame};
use rand::{thread_rng, Rng};

const SAMPLE_RATE: usize = 44100;
const CHANNELS: u8 = 2;
const QUALITY: u8 = 0;

#[derive(Debug)]
enum BroadcastError {
    Lame(EncodeError),
    Io(io::Error),
}

struct BroadcastEncoder<T> {
    output: T,
    lame: Lame,
    mp3_buffer: Vec<u8>,
}

impl<T> BroadcastEncoder<T> where T: Write {
    pub fn new(kilobitrate: usize, output: T) -> Self {
        let mut lame = Lame::new().expect("Lame::new");
        lame.set_kilobitrate(kilobitrate as i32).expect("set_kilobitrate");
        lame.set_quality(QUALITY).expect("set_quality");
        lame.set_channels(CHANNELS).expect("set_channels");
        lame.set_sample_rate(SAMPLE_RATE as u32).expect("set_sample_rate");
        lame.init_params().expect("init_params");

        BroadcastEncoder { output, lame, mp3_buffer: vec![0; 4096] }
    }
}

trait StreamOutput {
    fn write(&mut self, frame: &Frame) -> Result<(), BroadcastError>;
}

impl<T> StreamOutput for BroadcastEncoder<T> where T: Write {
    fn write(&mut self, frame: &Frame) -> Result<(), BroadcastError> {
        fn deinterleave(channels: usize, samples: &[i16]) -> (Vec<i16>, Vec<i16>) {
            match channels {
                0 => panic!("channels = 0"),
                1 => (samples.to_vec(), samples.to_vec()),
                _ => {
                    let mut left = Vec::new();
                    let mut right = Vec::new();

                    for chunk in samples.chunks(channels) {
                        left.push(chunk[0]);
                        right.push(chunk[1]);
                    }

                    (left, right)
                }
            }
        }

        let (left, right) = deinterleave(frame.channels, &frame.data);

        loop {
            match self.lame.encode(&left, &right, &mut self.mp3_buffer) {
                Ok(len) => {
                    self.output.write(&self.mp3_buffer[0..len])
                        .map_err(BroadcastError::Io)?;
                    return Ok(());
                }
                Err(EncodeError::OutputBufferTooSmall) => {
                    // double length of output buffer:
                    self.mp3_buffer.resize(self.mp3_buffer.len() * 2, 0);
                    // try again:
                    continue;
                }
                Err(e) => {
                    return Err(BroadcastError::Lame(e));
                }
            }
        }
    }
}

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
}

fn run_station(outputs: &mut [&mut StreamOutput]) -> Result<(), Error> {
    let tracks = fs::read_dir("catalog").map_err(Error::Io)?
        .collect::<Result<Vec<_>, _>>().map_err(Error::Io)?;

    let mut rng = thread_rng();
    let track = rng.choose(&tracks).expect("rng.choose");

    let mp3 = File::open(track.path()).map_err(Error::Io)?;

    let mut reader = Reader::new(Decoder::new(mp3));

    loop {
        match reader.read() {
            Ok(frame) => {
                for output in outputs.iter_mut() {
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

fn main() -> Result<(), Error> {
    let mut hi = BroadcastEncoder::new(320,
        File::create("stream-hi.mp3")
            .map_err(Error::Io)?);

    let mut lo = BroadcastEncoder::new(128,
        File::create("stream-lo.mp3")
            .map_err(Error::Io)?);

    run_station(&mut [
        &mut hi,
        &mut lo,
    ])
}
