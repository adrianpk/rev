//! Incremental framing for nonblocking pipes. One decoder belongs to one connection.

use crate::{limits::FRAME_BYTES, rpc::Message};
use std::{
    fmt,
    io::{self, Write},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    Malformed,
    LimitExceeded,
    Truncated,
    Closed,
}
impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Malformed => "invalid JSON-RPC frame",
            Self::LimitExceeded => "frame exceeds negotiated byte limit",
            Self::Truncated => "EOF within an NDJSON frame",
            Self::Closed => "connection decoder is retired",
        })
    }
}
impl std::error::Error for WireError {}

/// Consumes at most one frame per call; the caller retains unconsumed input.
/// A framing error retires the decoder instead of accepting an ambiguous suffix.
pub struct Decoder {
    bytes: Vec<u8>,
    limit: usize,
    closed: bool,
}
impl Decoder {
    pub fn new(limit: usize) -> Result<Self, WireError> {
        if limit == 0 || limit > FRAME_BYTES {
            return Err(WireError::LimitExceeded);
        }
        Ok(Self {
            bytes: Vec::new(),
            limit,
            closed: false,
        })
    }
    pub fn receive(&mut self, input: &[u8]) -> Result<(usize, Option<Message>), WireError> {
        if self.closed {
            return Err(WireError::Closed);
        }
        let newline = input.iter().position(|byte| *byte == b'\n');
        let count = newline.unwrap_or(input.len());
        if count > self.limit.saturating_sub(self.bytes.len()) {
            self.closed = true;
            self.bytes.clear();
            return Err(WireError::LimitExceeded);
        }
        self.bytes.extend_from_slice(&input[..count]);
        if newline.is_none() {
            return Ok((count, None));
        }
        let message = serde_json::from_slice::<Message>(&self.bytes)
            .ok()
            .filter(Message::validate);
        self.bytes.clear();
        match message {
            Some(message) => Ok((count + 1, Some(message))),
            None => {
                self.closed = true;
                Err(WireError::Malformed)
            }
        }
    }
    /// EOF after a complete newline is clean; an unfinished frame is an error.
    pub fn finish(&mut self) -> Result<(), WireError> {
        if self.closed {
            return Err(WireError::Closed);
        }
        self.closed = true;
        if self.bytes.is_empty() {
            Ok(())
        } else {
            self.bytes.clear();
            Err(WireError::Truncated)
        }
    }
    pub fn buffered_bytes(&self) -> usize {
        self.bytes.len()
    }
}

struct BoundedOutput {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("frame limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Serialize within a fixed allocation ceiling, including escaped UTF-8 text.
/// The limit excludes the single framing newline; queue accounting includes it.
pub fn encode(message: &Message, limit: usize) -> Result<Vec<u8>, WireError> {
    if limit == 0 || limit > FRAME_BYTES {
        return Err(WireError::LimitExceeded);
    }
    if !message.validate() {
        return Err(WireError::Malformed);
    }
    let mut output = BoundedOutput {
        bytes: Vec::new(),
        limit,
    };
    serde_json::to_writer(&mut output, message).map_err(|_| WireError::LimitExceeded)?;
    output.bytes.push(b'\n');
    Ok(output.bytes)
}
