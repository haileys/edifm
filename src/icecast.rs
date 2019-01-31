use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver, TryRecvError, RecvError};
use std::thread;
use std::time::Duration;

pub struct SourceStream {
    tx: SyncSender<Box<[u8]>>,
}

fn connect_stream(remote: &str, mountpoint: &str) -> Result<TcpStream, io::Error> {
    let mut stream = TcpStream::connect(remote)?;

    write!(stream, "SOURCE {} HTTP/1.1\r\n", mountpoint)?;
    write!(stream, "Authorization:  Basic c291cmNlOnNvdXJjZQ==\r\n")?;
    write!(stream, "Content-Type: audio/mpeg\r\n")?;
    write!(stream, "\r\n")?;

    stream.flush()?;

    Ok(stream)
}

fn run_stream_thread(rx: Receiver<Box<[u8]>>, remote: String, mountpoint: String) {
    loop {
        // connect to icecast:
        let mut stream = match connect_stream(&remote, &mountpoint) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("could not connect to {}{}: {:?}", remote, mountpoint, e);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        // clear out the pending receive queue:
        loop {
            match rx.try_recv() {
                Ok(_) => continue,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // start streaming new chunks of audio to icecast:
        loop {
            match rx.recv() {
                Ok(buf) => {
                    match stream.write_all(&buf) {
                        Ok(()) => {
                            let _ = stream.flush();
                        }
                        Err(e) => {
                            eprintln!("could not write to {}{}: {:?}", remote, mountpoint, e);
                            thread::sleep(Duration::from_secs(1));
                            break;
                        }
                    }
                }
                Err(RecvError) => return
            }
        }
    }
}

impl SourceStream {
    pub fn new(remote: &str, mountpoint: &str) -> Result<Self, io::Error> {
        let (tx, rx) = sync_channel(2);

        let remote = remote.to_owned();
        let mountpoint = mountpoint.to_owned();

        thread::spawn(move || run_stream_thread(rx, remote, mountpoint));

        Ok(SourceStream { tx })
    }
}

impl Write for SourceStream {
    fn write(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        // don't block if outbound queue is full:
        let _ = self.tx.try_send(data.to_owned().into_boxed_slice());
        Ok(data.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        // no-op, SourceStream streams in real time and may drop data during reconnects
        Ok(())
    }
}
