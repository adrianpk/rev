use super::*;
use std::{
    io::{ErrorKind, Read},
    os::{fd::OwnedFd, unix::net::UnixStream},
};
struct Stream {
    read: UnixStream,
    bytes: Vec<u8>,
    closed: bool,
}
pub(super) struct Streams {
    stdout: Stream,
    stderr: Stream,
}
impl Stream {
    fn new() -> Result<(Self, Stdio), String> {
        let (read, write) = UnixStream::pair().map_err(|e| e.to_string())?;
        read.set_nonblocking(true).map_err(|e| e.to_string())?;
        Ok((
            Self {
                read,
                bytes: Vec::new(),
                closed: false,
            },
            Stdio::from(OwnedFd::from(write)),
        ))
    }
    fn read(&mut self, remaining: usize) -> Result<(), String> {
        let mut buffer = [0u8; 8192];
        let limit = buffer.len().min(remaining + 1);
        match self.read.read(&mut buffer[..limit]) {
            Ok(0) => self.closed = true,
            Ok(n) => {
                if n > remaining {
                    return Err("External job output limit reached".into());
                }
                self.bytes.extend_from_slice(&buffer[..n]);
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => (),
            Err(e) => return Err(format!("Read external output: {e}")),
        }
        Ok(())
    }
}
impl Streams {
    pub(super) fn new() -> Result<(Self, Stdio, Stdio), String> {
        let (stdout, out) = Stream::new()?;
        let (stderr, err) = Stream::new()?;
        Ok((Self { stdout, stderr }, out, err))
    }
    pub(super) fn read(&mut self, limit: usize) -> Result<(), String> {
        self.stdout
            .read(limit.saturating_sub(self.stdout.bytes.len() + self.stderr.bytes.len()))?;
        self.stderr
            .read(limit.saturating_sub(self.stdout.bytes.len() + self.stderr.bytes.len()))
    }
    pub(super) fn closed(&self) -> bool {
        self.stdout.closed && self.stderr.closed
    }
    pub(super) fn outcome(&mut self, status: ExitStatus) -> Outcome {
        Outcome {
            status,
            stdout: std::mem::take(&mut self.stdout.bytes),
            stderr: std::mem::take(&mut self.stderr.bytes),
        }
    }
}
