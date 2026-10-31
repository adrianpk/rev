use std::collections::VecDeque;
use std::io::{BufReader, Read};
use std::net::Shutdown;
use std::os::{fd::OwnedFd, unix::net::UnixStream};
use std::process::Stdio;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
    mpsc::{self, SyncSender},
};
use std::time::Duration;

use serde_json::Value;

use super::{
    frame,
    inbox::{Inbox, QUEUE_BYTES, QUEUE_MESSAGES},
};

pub(super) type Stderr = Arc<Mutex<VecDeque<u8>>>;

pub(super) struct Pipes {
    pub input: UnixStream,
    pub output: UnixStream,
    pub error: UnixStream,
}

impl Pipes {
    pub fn new(timeout: Duration) -> Result<(Self, [Stdio; 3]), String> {
        let (input, stdin) = pair()?;
        let (output, stdout) = pair()?;
        let (error, stderr) = pair()?;
        input
            .set_write_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        Ok((
            Self {
                input,
                output,
                error,
            },
            [stdin, stdout, stderr],
        ))
    }
}

fn pair() -> Result<(UnixStream, Stdio), String> {
    let (read, write) = UnixStream::pair().map_err(|e| e.to_string())?;
    Ok((read, Stdio::from(OwnedFd::from(write))))
}

pub(super) struct Outbox {
    send: SyncSender<Vec<u8>>,
    bytes: Arc<AtomicUsize>,
    limit: usize,
}

impl Outbox {
    pub fn send(&self, value: &Value) -> Result<(), String> {
        let data = frame::encode(value, self.limit)?;
        let count = data.len();
        self.bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |bytes| {
                bytes
                    .checked_add(count)
                    .filter(|total| *total <= QUEUE_BYTES)
            })
            .map_err(|_| "LSP outbound byte queue overflow")?;
        if self.send.try_send(data).is_err() {
            self.bytes.fetch_sub(count, Ordering::AcqRel);
            return Err("LSP outbound message queue overflow or closed".into());
        }
        Ok(())
    }
}

/// IO threads block on their sockets/channels; the supervisor owns all deadlines.
/// Shutdown closes cloned sockets before joining, including inherited streams.
pub(super) fn drive(
    pipes: Pipes,
    inbox: Arc<Inbox>,
    stderr: Stderr,
    limit: usize,
    run: impl FnOnce(Outbox),
) -> Result<(), String> {
    let input = pipes.input.try_clone().map_err(|e| e.to_string())?;
    let output = pipes.output.try_clone().map_err(|e| e.to_string())?;
    let error = pipes.error.try_clone().map_err(|e| e.to_string())?;
    let (send, receive) = mpsc::sync_channel::<Vec<u8>>(QUEUE_MESSAGES);
    let bytes = Arc::new(AtomicUsize::new(0));
    std::thread::scope(|scope| {
        let read_inbox = inbox.clone();
        scope.spawn(move || read_output(output, &read_inbox, limit));
        scope.spawn(move || read_stderr(error, stderr));
        let written = bytes.clone();
        scope.spawn(move || {
            let mut input = input;
            for data in receive {
                let result = frame::write(&mut input, &data);
                written.fetch_sub(data.len(), Ordering::AcqRel);
                if let Err(error) = result {
                    inbox.fail(format!("LSP stdin: {error}"));
                    break;
                }
            }
        });
        run(Outbox { send, bytes, limit });
        for socket in [&pipes.input, &pipes.output, &pipes.error] {
            let _ = socket.shutdown(Shutdown::Both);
        }
    });
    Ok(())
}

fn read_output(output: UnixStream, inbox: &Inbox, limit: usize) {
    let mut input = BufReader::new(output);
    loop {
        match frame::read(&mut input, limit) {
            Ok((message, count)) => inbox.push(message, count),
            Err(error) => {
                inbox.fail(error);
                break;
            }
        }
    }
}

fn read_stderr(mut input: UnixStream, stderr: Stderr) {
    let mut bytes = [0; 4096];
    while let Ok(count) = input.read(&mut bytes) {
        if count == 0 {
            break;
        }
        let mut retained = stderr.lock().expect("LSP stderr lock");
        let excess = (retained.len() + count).saturating_sub(65_536);
        retained.drain(..excess);
        retained.extend(&bytes[..count]);
    }
}
