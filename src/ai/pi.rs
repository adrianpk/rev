//! One lazy, owned Pi 1.3.0 RPC operation. Input-only grants always use a new
//! child with no tools or persisted session. All process IO is off the UI thread.

mod frame;
mod io;
mod protocol;

use std::path::PathBuf;
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::config::Pi;

#[derive(Default)]
pub(super) struct Control {
    cancel: AtomicBool,
    accepted: AtomicBool,
    cleanup_failed: AtomicBool,
}

impl Control {
    fn canceled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
}

/// A settled explanation and observed backend identity, without provider secrets.
#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub model: String,
}

/// Owns the only supervisor slot until all child IO/shutdown has completed.
/// Dropping requests cancellation and waits only for the finite cleanup owner.
pub struct Operation {
    epoch: u64,
    control: Arc<Control>,
    task: Option<JoinHandle<Result<Output, String>>>,
}

impl Operation {
    /// Start an explicit bounded input-only invocation; no shell interpolation,
    /// filesystem discovery or previous Pi conversation is admitted.
    pub fn start(policy: Pi, root: PathBuf, message: String, epoch: u64) -> Result<Self, String> {
        if message.len() > policy.max_input_bytes {
            return Err("Pi editor envelope exceeds max_input_bytes".into());
        }
        frame::encode(&json!({"id":format!("{epoch}:prompt"),"type":"prompt","message":&message}))?;
        let control = Arc::new(Control::default());
        let worker = control.clone();
        let deadline = Instant::now() + Duration::from_millis(policy.request_timeout_ms as u64);
        let task = std::thread::Builder::new()
            .name("rev-pi".into())
            .spawn(move || {
                let root = root
                    .canonicalize()
                    .map_err(|_| "Pi working directory is unavailable")?;
                if !root.is_dir() {
                    return Err("Pi working directory is not a directory".into());
                }
                probe(&policy, &root, &worker, deadline)?;
                if worker.canceled() {
                    return Err("Pi invocation canceled".into());
                }
                let (pipes, stdio) = io::Pipes::new()?;
                let [stdin, stdout, stderr] = stdio;
                let mut command = Command::new(&policy.command);
                command.args([
                    "--mode",
                    "rpc",
                    "--no-session",
                    "--no-tools",
                    "--no-extensions",
                    "--no-mcp",
                    "--no-skills",
                    "--no-prompt-templates",
                    "--no-context-files",
                    "--no-themes",
                    "--no-approve",
                    "--offline",
                ]);
                if let Some(model) = &policy.model {
                    command.arg("--model").arg(model);
                }
                let mut child = io::Owned::new(
                    command
                        .current_dir(&root)
                        .stdin(stdin)
                        .stdout(stdout)
                        .stderr(stderr)
                        .spawn()
                        .map_err(|_| "Pi RPC launch failed; check ai.pi.command")?,
                    worker.clone(),
                );
                pipes.drive(&mut child, |inbox, input, _child| {
                    let result =
                        protocol::run(inbox, input, &worker, &policy, &message, epoch, deadline);
                    if result.is_err() {
                        protocol::cancel(inbox, input, epoch);
                    }
                    result
                })
            })
            .map_err(|_| "Pi supervisor could not start")?;
        Ok(Self {
            epoch,
            control,
            task: Some(task),
        })
    }

    /// Failed OS cleanup blocks replacement children for this editor owner.
    pub fn cleanup_failed(&self) -> bool {
        self.control.cleanup_failed.load(Ordering::Acquire)
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn accepted(&self) -> bool {
        self.control.accepted.load(Ordering::Acquire)
    }
    pub fn cancel(&self) {
        self.control.cancel.store(true, Ordering::Release);
    }
    pub fn is_canceled(&self) -> bool {
        self.control.canceled()
    }
    pub fn finished(&self) -> bool {
        self.task.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Only joins a completed worker. Cancellation fences even a settled result
    /// that raced with native input before the UI took the completion.
    pub fn take(&mut self) -> Option<Result<Output, String>> {
        if !self.finished() {
            return None;
        }
        let result = self
            .task
            .take()?
            .join()
            .unwrap_or_else(|_| Err("Pi supervisor failed".into()));
        Some(if self.is_canceled() {
            Err("Pi invocation canceled".into())
        } else {
            result
        })
    }
}

impl Drop for Operation {
    fn drop(&mut self) {
        self.cancel();
        if let Some(task) = self.task.take() {
            let _ = task.join();
        }
    }
}

fn probe(
    policy: &Pi,
    root: &std::path::Path,
    control: &Arc<Control>,
    deadline: Instant,
) -> Result<(), String> {
    let (pipes, stdio) = io::Pipes::new()?;
    let [stdin, stdout, stderr] = stdio;
    let mut child = io::Owned::new(
        Command::new(&policy.command)
            .arg("--version")
            .env("PI_OFFLINE", "1")
            .current_dir(root)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|_| "Pi is unavailable; install Pi 1.3.0 or set ai.pi.command")?,
        control.clone(),
    );
    pipes.drive(&mut child, |inbox, _input, child| {
        let deadline = deadline.min(Instant::now() + Duration::from_secs(5));
        loop {
            if control.canceled() {
                return Err("Pi invocation canceled".into());
            }
            if Instant::now() >= deadline {
                return Err("Pi version probe exceeded its deadline".into());
            }
            if let Some(bytes) = inbox.next(Duration::from_millis(20))? {
                if bytes != b"1.3.0" {
                    return Err(
                        "Unsupported Pi version; this adapter requires checked Pi 1.3.0".into(),
                    );
                }
                // Version output is an admission observation, not model work.
                // The same executable is used for the owned RPC child.
                let exit = Instant::now() + Duration::from_secs(1);
                while Instant::now() < exit {
                    if let Some(status) = child.try_wait().map_err(|_| "Pi version wait failed")? {
                        return if status.success() {
                            Ok(())
                        } else {
                            Err("Pi version probe failed".into())
                        };
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                return Err("Pi version probe did not exit".into());
            }
        }
    })
}
