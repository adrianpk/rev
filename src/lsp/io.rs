use std::collections::VecDeque;
use std::io::{BufReader, Read};
use std::net::Shutdown;
use std::os::{fd::OwnedFd, unix::net::UnixStream};
use std::process::Stdio;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, SyncSender},
};
use std::time::Duration;

use serde_json::Value;

use super::{
    budget::{Budget, Permit},
    frame,
    inbox::{Inbox, QUEUE_MESSAGES},
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

struct Frame {
    bytes: Vec<u8>,
    _permit: Permit,
}

pub(super) struct Outbox {
    send: SyncSender<Frame>,
    budget: Arc<Budget>,
    limit: usize,
}

impl Outbox {
    pub fn send(&self, value: &Value) -> Result<(), String> {
        let data = frame::encode(value, self.limit)?;
        let permit = self.budget.reserve(data.len())?;
        self.enqueue(data, permit)
    }

    pub fn document(&self, value: &Value, mut permit: Permit) -> Result<(), String> {
        let data = frame::encode(value, self.limit)?;
        permit.resize(data.len())?;
        self.enqueue(data, permit)
    }

    fn enqueue(&self, bytes: Vec<u8>, permit: Permit) -> Result<(), String> {
        self.send
            .try_send(Frame {
                bytes,
                _permit: permit,
            })
            .map_err(|_| "LSP outbound message queue overflow or closed".into())
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
    let Pipes {
        input: owned_input,
        output: owned_output,
        error: owned_error,
    } = pipes;
    let input = owned_input.try_clone().map_err(|e| e.to_string())?;
    let output = owned_output.try_clone().map_err(|e| e.to_string())?;
    let error = owned_error.try_clone().map_err(|e| e.to_string())?;
    let (send, receive) = mpsc::sync_channel::<Frame>(QUEUE_MESSAGES);
    let budget = Arc::clone(&inbox.budget);
    std::thread::scope(|scope| {
        let read_inbox = Arc::clone(&inbox);
        scope.spawn(move || read_output(output, &read_inbox, limit));
        scope.spawn(move || read_stderr(error, &stderr));
        scope.spawn(move || {
            let mut input = input;
            for data in receive {
                let result = frame::write(&mut input, &data.bytes);
                if let Err(error) = result {
                    inbox.fail(format!("LSP stdin: {error}"));
                    break;
                }
            }
        });
        run(Outbox {
            send,
            budget,
            limit,
        });
        for socket in [&owned_input, &owned_output, &owned_error] {
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

fn read_stderr(mut input: UnixStream, stderr: &Mutex<VecDeque<u8>>) {
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
