use std::io::{self, Write};

use lame::{Lame, EncodeError};
use minimp3::Frame;

pub const SAMPLE_RATE: usize = 44100;
pub const CHANNELS: u8 = 2;
pub const QUALITY: u8 = 0;

#[derive(Debug)]
pub enum BroadcastError {
    Lame(EncodeError),
    Io(io::Error),
}

pub struct BroadcastEncoder<T> {
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

pub trait StreamOutput {
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
