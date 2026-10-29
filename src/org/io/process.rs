//! One owned Unix process group with bounded nonblocking output and cleanup.
//! This is lifetime ownership for explicitly selected tools, not a sandbox.
mod group;
mod streams;
use crate::org::Cancellation;
use std::{
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};
use streams::Streams;
pub(crate) struct Program {
    pub argv: Vec<String>,
    pub directory: PathBuf,
    pub seconds: usize,
    pub output_bytes: usize,
}
pub(crate) struct Outcome {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
struct Owned {
    child: Option<Child>,
    streams: Streams,
    pid: u32,
    group_stopped: bool,
}
impl Program {
    pub(crate) fn run(self, cancel: &Cancellation) -> Result<Outcome, String> {
        self.validate()?;
        if cancel.is_cancelled() {
            return Err("External job cancelled before launch".into());
        }
        let mut owned = self.launch()?;
        let result = owned.collect(&self, cancel);
        let cleanup = owned.stop();
        cleanup?;
        result
    }
    fn validate(&self) -> Result<(), String> {
        if self.argv.is_empty()
            || self.argv.len() > 64
            || self.argv.iter().any(|a| a.len() > 4096 || a.contains('\0'))
        {
            return Err("Program argv exceeds its 64-token/4096-byte profile".into());
        }
        if !(1..=600).contains(&self.seconds) || !(1..=16777216).contains(&self.output_bytes) {
            return Err("Program requires finite time and output bounds".into());
        }
        group::available()
    }
    fn launch(&self) -> Result<Owned, String> {
        let (streams, stdout, stderr) = Streams::new()?;
        let mut command = Command::new(&self.argv[0]);
        command
            .args(&self.argv[1..])
            .current_dir(&self.directory)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .process_group(0);
        let child = command
            .spawn()
            .map_err(|e| format!("Start configured program: {e}"))?;
        let pid = child.id();
        Ok(Owned {
            child: Some(child),
            streams,
            pid,
            group_stopped: false,
        })
    }
}
impl Owned {
    fn collect(&mut self, program: &Program, cancel: &Cancellation) -> Result<Outcome, String> {
        let deadline = Instant::now() + Duration::from_secs(program.seconds as u64);
        let mut exited = None;
        let mut draining = None;
        loop {
            if cancel.is_cancelled() {
                return Err("External job cancelled".into());
            }
            if Instant::now() >= deadline {
                return Err("External job timed out".into());
            }
            self.streams.read(program.output_bytes)?;
            exited = self.status(exited)?;
            if let Some(status) = exited
                && self.drained(&mut draining)?
            {
                return Ok(self.streams.outcome(status));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn status(&mut self, exited: Option<ExitStatus>) -> Result<Option<ExitStatus>, String> {
        if exited.is_some() {
            return Ok(exited);
        }
        self.child
            .as_mut()
            .expect("owned child")
            .try_wait()
            .map_err(|e| e.to_string())
    }
    fn drained(&mut self, deadline: &mut Option<Instant>) -> Result<bool, String> {
        if deadline.is_none() {
            self.stop_group()?;
            *deadline = Some(Instant::now() + Duration::from_millis(250));
        }
        if self.streams.closed() {
            return Ok(true);
        }
        if deadline.is_some_and(|end| Instant::now() >= end) {
            return Err("External output did not close after process-group cleanup".into());
        }
        Ok(false)
    }
    fn stop_group(&mut self) -> Result<(), String> {
        if !self.group_stopped {
            group::stop(self.pid)?;
            self.group_stopped = true;
        }
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        let group = self.stop_group();
        let Some(mut child) = self.child.take() else {
            return group;
        };
        let kill = match child.try_wait() {
            Ok(Some(_)) => Ok(()),
            _ => child.kill(),
        };
        let wait = child.wait();
        group?;
        kill.map_err(|e| format!("Stop owned child: {e}"))?;
        wait.map_err(|e| format!("Reap owned child: {e}"))?;
        Ok(())
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        if self.child.is_some() {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests;
