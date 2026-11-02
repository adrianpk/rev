use std::collections::BTreeMap;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::lsp::{frame, io::Pipes};
use crate::org::Cancellation;

use super::process::Process;

/// Shared bounded sockets for optional acquisition and protocol probes. Reading
/// and writing each take one nonblocking step, then return to deadline checks.
pub(super) struct Streams {
    pipes: Pipes,
    pub output: Vec<u8>,
    error: Vec<u8>,
    input: Vec<u8>,
    count: usize,
    pub closed: bool,
    error_closed: bool,
}

impl Streams {
    pub fn new() -> Result<(Self, [Stdio; 3]), String> {
        let (pipes, stdio) = Pipes::new(Duration::from_millis(50))?;
        for socket in [&pipes.input, &pipes.output, &pipes.error] {
            socket.set_nonblocking(true).map_err(|e| e.to_string())?;
        }
        Ok((
            Self {
                pipes,
                output: Vec::new(),
                error: Vec::new(),
                input: Vec::new(),
                count: 0,
                closed: false,
                error_closed: false,
            },
            stdio,
        ))
    }

    pub fn send(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let bytes = frame::encode(value, super::OUTPUT_BYTES)?;
        if self.input.len() + bytes.len() > super::OUTPUT_BYTES {
            return Err("probe input queue exceeds 1 MiB".into());
        }
        self.input.extend(bytes);
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        self.write()?;
        let remaining = super::OUTPUT_BYTES.saturating_sub(self.count);
        let (output, closed) = read(&mut self.pipes.output, remaining)?;
        self.closed |= closed;
        self.count += output.len();
        self.output.extend(output);
        let (error, closed) = read(
            &mut self.pipes.error,
            super::OUTPUT_BYTES.saturating_sub(self.count),
        )?;
        self.error_closed |= closed;
        self.count += error.len();
        self.error.extend(error);
        let excess = self.error.len().saturating_sub(super::DISPLAY_BYTES);
        self.error.drain(..excess);
        Ok(())
    }

    fn write(&mut self) -> Result<(), String> {
        if self.input.is_empty() {
            return Ok(());
        }
        match self.pipes.input.write(&self.input) {
            Ok(0) => Err("probe input closed".into()),
            Ok(count) => {
                self.input.drain(..count);
                Ok(())
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => Ok(()),
            Err(e) => Err(format!("write probe input: {e}")),
        }
    }

    pub fn flushed(&self) -> bool {
        self.input.is_empty()
    }

    pub fn message(&mut self) -> Result<Option<frame::Message>, String> {
        let Some(end) = self
            .output
            .windows(4)
            .position(|s| s == b"\r\n\r\n")
            .map(|end| end + 4)
        else {
            if self.output.len() > frame::HEADER_BYTES {
                return Err("probe header exceeds 16 KiB".into());
            }
            return Ok(None);
        };
        let length = frame::header(
            &mut std::io::Cursor::new(&self.output[..end]),
            super::OUTPUT_BYTES,
        )?;
        if self.output.len() < end + length {
            return Ok(None);
        }
        let (message, _) = frame::read(
            &mut std::io::Cursor::new(&self.output[..end + length]),
            super::OUTPUT_BYTES,
        )?;
        self.output.drain(..end + length);
        Ok(Some(message))
    }
}

fn read(stream: &mut impl Read, remaining: usize) -> Result<(Vec<u8>, bool), String> {
    let mut bytes = [0; 8192];
    let limit = bytes.len().min(remaining + 1);
    match stream.read(&mut bytes[..limit]) {
        Ok(count) if count > remaining => Err("installation helper output exceeds 1 MiB".into()),
        Ok(count) => Ok((bytes[..count].to_vec(), count == 0)),
        Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {
            Ok((Vec::new(), false))
        }
        Err(e) => Err(format!("read installation helper: {e}")),
    }
}

/// A finite structured invocation; no shell, user workspace or implicit runtime
/// installation is involved. File output has separate transfer/payload limits.
pub(super) struct Helper {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub directory: PathBuf,
    pub env: BTreeMap<String, String>,
    pub seconds: u64,
    pub watch: Vec<(PathBuf, u64)>,
}

struct Captured {
    output: Vec<u8>,
    error: Vec<u8>,
}

impl Helper {
    pub fn run(&self, cancel: &Cancellation, file: Option<File>) -> Result<Vec<u8>, String> {
        self.execute(cancel, file, None)
            .map(|captured| captured.output)
    }

    /// Git acquisition discards inherited Git redirections/configuration before
    /// applying its explicit owned paths. Other helpers keep existing behavior.
    pub fn run_isolated(&self, cancel: &Cancellation, prefix: &str) -> Result<Vec<u8>, String> {
        self.execute(cancel, None, Some(prefix))
            .map(|captured| captured.output)
    }

    /// Build wrappers may hide a compiler's nonzero status. Retain both bounded
    /// streams so checking the required output can report the actual failure.
    pub fn run_build(&self, cancel: &Cancellation) -> Result<Vec<u8>, String> {
        self.execute(cancel, None, None).map(|mut captured| {
            captured.output.extend(captured.error);
            captured.output
        })
    }

    fn execute(
        &self,
        cancel: &Cancellation,
        file: Option<File>,
        prefix: Option<&str>,
    ) -> Result<Captured, String> {
        checkpoint(cancel, Instant::now() + Duration::from_secs(self.seconds))?;
        let (mut streams, [stdin, stdout, stderr]) = Streams::new()?;
        drop(stdin);
        let mut command = self.command();
        if let Some(prefix) = prefix {
            for (key, _) in
                std::env::vars_os().filter(|(key, _)| key.to_string_lossy().starts_with(prefix))
            {
                command.env_remove(key);
            }
            command.envs(&self.env);
        }
        command
            .stdin(Stdio::null())
            .stdout(file.map_or(stdout, Stdio::from))
            .stderr(stderr);
        let mut process = Process::start(&mut command)?;
        // Command retains its configured descriptors after spawn. Drop the
        // parent writer ends so EOF can acknowledge helper termination.
        drop(command);
        let result = self.collect(&mut process, &mut streams, cancel);
        process
            .finish(result)
            .map(|output| Captured {
                output,
                error: std::mem::take(&mut streams.error),
            })
            .map_err(|e| {
                let tail = String::from_utf8_lossy(&streams.error);
                format!(
                    "{}: {e}; stderr: {tail}",
                    self.executable
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )
            })
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command
            .args(&self.args)
            .current_dir(&self.directory)
            .envs(&self.env)
            .env("LC_ALL", "C");
        command
    }

    fn collect(
        &self,
        process: &mut Process,
        streams: &mut Streams,
        cancel: &Cancellation,
    ) -> Result<Vec<u8>, String> {
        let deadline = Instant::now() + Duration::from_secs(self.seconds);
        let mut account_at = Instant::now();
        loop {
            checkpoint(cancel, deadline)?;
            streams.step()?;
            self.monitor(&mut account_at)?;
            if settled(process, streams)? {
                self.account()?;
                return Ok(std::mem::take(&mut streams.output));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn monitor(&self, account_at: &mut Instant) -> Result<(), String> {
        if Instant::now() >= *account_at {
            self.account()?;
            *account_at = Instant::now() + Duration::from_millis(100);
        }
        Ok(())
    }

    pub fn account(&self) -> Result<(), String> {
        for (path, limit) in &self.watch {
            super::tree::bytes(path, *limit)?;
        }
        Ok(())
    }
}

fn settled(process: &mut Process, streams: &Streams) -> Result<bool, String> {
    if !process.exited()? {
        return Ok(false);
    }
    process.stop()?;
    Ok(streams.closed && streams.error_closed)
}

pub(super) fn checkpoint(cancel: &Cancellation, deadline: Instant) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err("installation cancelled or total deadline reached".into());
    }
    if Instant::now() >= deadline {
        return Err("installation helper timed out".into());
    }
    Ok(())
}

impl Drop for Streams {
    fn drop(&mut self) {
        for socket in [&self.pipes.input, &self.pipes.output, &self.pipes.error] {
            let _ = socket.shutdown(Shutdown::Both);
        }
    }
}

pub(super) fn executable(name: &str, path: Option<&std::ffi::OsStr>) -> Result<PathBuf, String> {
    match crate::lsp::catalog::executable(name, path) {
        crate::lsp::catalog::Executable::Available(path) => Ok(path),
        _ => Err(format!(
            "missing prerequisite: install {name} separately and provide an absolute executable search path"
        )),
    }
}

pub(super) fn owned_file(path: &Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;
    File::options()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| format!("create owned output: {e}"))
}
