//! Exact OS argv and retained failure facts for native Git. Collection and
//! cleanup run on the operation worker; existing text-tool callers keep their
//! own stricter success/error contract.
use super::{Cancellation, streams::Streams};
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fd::OwnedFd,
    process::{Pid, Signal},
};
use std::{
    ffi::OsString,
    os::unix::{ffi::OsStrExt, process::CommandExt},
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

pub(crate) enum Editors {
    Disabled,
    AcceptMessage,
    Todo(std::fs::File),
}

pub(crate) struct Program {
    pub argv: Vec<OsString>,
    pub directory: PathBuf,
    pub seconds: usize,
    pub output_bytes: usize,
    pub input: Option<std::fs::File>,
    pub editors: Editors,
    pub git_index: Option<PathBuf>,
    pub workspace: Option<std::sync::Arc<crate::file::temporary::Directory>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum End {
    Exited,
    Cancelled,
    TimedOut,
    OutputLimit,
    Failed,
}

pub(crate) struct Collected {
    pub status: Option<ExitStatus>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub end: End,
    pub diagnostic: Option<String>,
    pub retained: Option<Owned>,
}

/// A failed cleanup returns this owner. Keeping it also keeps the streams and
/// unreaped leader; no group ID can be recycled before its last signal.
pub(crate) struct Owned {
    child: Option<Child>,
    streams: Streams,
    exit: Option<OwnedFd>,
    observe_error: Option<String>,
    pid: Pid,
    _todo: Option<std::fs::File>,
    _workspace: Option<std::sync::Arc<crate::file::temporary::Directory>>,
}

impl Program {
    pub fn run(mut self, cancel: &Cancellation, mut phase: impl FnMut(u32, bool)) -> Collected {
        let input = self.input.take();
        let mut owned = match self.launch(input, cancel) {
            Ok(owned) => owned,
            Err(error) => {
                return Collected {
                    status: None,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    end: if cancel.is_cancelled() {
                        End::Cancelled
                    } else {
                        End::Failed
                    },
                    diagnostic: Some(error),
                    retained: None,
                };
            }
        };
        phase(owned.pid.as_raw_nonzero().get() as u32, false);
        let (mut end, mut diagnostic) = owned.collect(&self, cancel);
        phase(owned.pid.as_raw_nonzero().get() as u32, true);
        let cleanup = owned.stop(end == End::Exited);
        if cleanup.is_ok()
            && end != End::OutputLimit
            && let Err(error) = owned.drain(self.output_bytes)
        {
            if end == End::Exited {
                end = if error == "External job output limit reached" {
                    End::OutputLimit
                } else {
                    End::Failed
                };
                diagnostic = Some(error);
            } else {
                diagnostic = Some(format!("{}; {error}", diagnostic.unwrap_or_default()));
            }
        }
        let status = match &cleanup {
            Ok(status) => Some(*status),
            Err(error) => {
                diagnostic = Some(format!(
                    "{}Cleanup remains owned: {error}",
                    diagnostic.map_or(String::new(), |s| format!("{s}; "))
                ));
                None
            }
        };
        let (stdout, stderr) = owned.streams.take();
        Collected {
            status,
            stdout,
            stderr,
            end,
            diagnostic,
            retained: cleanup.is_err().then_some(owned),
        }
    }

    fn launch(
        &mut self,
        input: Option<std::fs::File>,
        cancel: &Cancellation,
    ) -> Result<Owned, String> {
        if self.argv.is_empty()
            || self.argv.len() > 64
            || self
                .argv
                .iter()
                .any(|a| a.as_bytes().len() > 4096 || a.as_bytes().contains(&0))
            || self.argv.iter().map(|a| a.as_bytes().len()).sum::<usize>() > 65536
        {
            return Err(
                "Git argv exceeds 64 tokens, 4096 bytes per token or 65536 total bytes".into(),
            );
        }
        if !(1..=600).contains(&self.seconds) || !(1..=16777216).contains(&self.output_bytes) {
            return Err("Git requires finite time and output bounds".into());
        }
        if cancel.is_cancelled() {
            return Err("Git cancelled before launch".into());
        }
        // Check fixed noninteractive drivers before Git can change anything.
        use std::os::unix::fs::PermissionsExt;
        let drivers: &[&str] = match self.editors {
            Editors::Disabled => &[],
            Editors::AcceptMessage => &["/bin/true"],
            Editors::Todo(_) => &["/bin/true", "/bin/cp"],
        };
        for path in drivers {
            if !std::fs::metadata(path)
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            {
                return Err("Fixed noninteractive Git editor driver is unavailable".into());
            }
        }
        let (streams, stdout, stderr) = Streams::new()?;
        let mut command = Command::new(&self.argv[0]);
        command
            .args(&self.argv[1..])
            .current_dir(&self.directory)
            .stdin(input.map_or_else(Stdio::null, Stdio::from))
            .stdout(stdout)
            .stderr(stderr)
            .process_group(0)
            .env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_EDITOR", "false")
            .env("GIT_SEQUENCE_EDITOR", "false")
            .env("GIT_PAGER", "cat");
        match &self.editors {
            Editors::Disabled => {}
            Editors::AcceptMessage => {
                command.env("GIT_EDITOR", "/bin/true");
            }
            Editors::Todo(file) => {
                use std::os::fd::AsRawFd;
                command.env("GIT_EDITOR", "/bin/true");
                command.env(
                    "GIT_SEQUENCE_EDITOR",
                    format!(
                        "/bin/cp -- /proc/{}/fd/{}",
                        std::process::id(),
                        file.as_raw_fd()
                    ),
                );
            }
        }
        // Ambient repository overrides must not redirect an explicitly
        // captured directory to an unrelated worktree or index.
        for key in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
            "GIT_PREFIX",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_NAMESPACE",
        ] {
            command.env_remove(key);
        }
        if let Some(index) = &self.git_index {
            if !index.is_absolute()
                || index.as_os_str().len() > 4096
                || index.as_os_str().as_bytes().contains(&0)
            {
                return Err("Owned private index path is invalid".into());
            }
            command.env("GIT_INDEX_FILE", index);
        }
        let child = command.spawn().map_err(|e| format!("Start Git: {e}"))?;
        let pid = Pid::from_child(&child);
        let observed = rustix::process::pidfd_open(pid, rustix::process::PidfdFlags::NONBLOCK);
        let observe_error = observed
            .as_ref()
            .err()
            .map(|error| format!("Observe Git child: {error}"));
        Ok(Owned {
            child: Some(child),
            streams,
            exit: observed.ok(),
            observe_error,
            pid,
            _workspace: self.workspace.take(),
            _todo: match std::mem::replace(&mut self.editors, Editors::Disabled) {
                Editors::Todo(file) => Some(file),
                _ => None,
            },
        })
    }
}

impl Owned {
    pub fn pending(&self) -> bool {
        self.child.is_some()
    }
    fn exited(&self) -> Result<bool, String> {
        let mut fds = [PollFd::new(
            self.exit
                .as_ref()
                .ok_or("Git child readiness is unavailable")?,
            PollFlags::IN,
        )];
        match poll(&mut fds, Some(&Timespec::default())) {
            Ok(_) => Ok(!fds[0].revents().is_empty()),
            Err(rustix::io::Errno::INTR) => Ok(false),
            Err(error) => Err(format!("Observe Git exit: {error}")),
        }
    }

    fn collect(&mut self, program: &Program, cancel: &Cancellation) -> (End, Option<String>) {
        if let Some(error) = self.observe_error.take() {
            return (End::Failed, Some(error));
        }
        let deadline = Instant::now() + Duration::from_secs(program.seconds as u64);
        loop {
            if cancel.is_cancelled() {
                return (End::Cancelled, Some("Git cancelled".into()));
            }
            if Instant::now() >= deadline {
                return (End::TimedOut, Some("Git timed out".into()));
            }
            if let Err(error) = self.streams.read(program.output_bytes) {
                let end = if error == "External job output limit reached" {
                    End::OutputLimit
                } else {
                    End::Failed
                };
                return (end, Some(error));
            }
            match self.exited() {
                Ok(true) => return (End::Exited, None),
                Err(error) => return (End::Failed, Some(error)),
                Ok(false) => (),
            }
            // The worker blocks on actual stream/exit readiness. The finite
            // upper bound lets cooperative cancellation reach a quiet child.
            if let Err(error) = self.streams.wait(
                self.exit.as_ref().expect("admitted Git child readiness"),
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(20)),
            ) {
                return (End::Failed, Some(error));
            }
        }
    }

    pub fn stop(&mut self, exited: bool) -> Result<ExitStatus, String> {
        let start = Instant::now();
        let deadline = start + Duration::from_secs(2);
        self.signal(if exited { Signal::KILL } else { Signal::TERM })?;
        // The leader remains unreaped, so its group ID cannot be reused.
        // After SIGKILL and a complete no-live-member observation, another
        // whole process-table scan cannot establish more cleanup authority.
        if exited && !self.running(deadline)? {
            return self.reap();
        }
        let grace = start + Duration::from_millis(500);
        while Instant::now() < grace && self.running(deadline)? {
            std::thread::sleep(Duration::from_millis(2));
        }
        if self.running(deadline)? {
            self.signal(Signal::KILL)?;
        }
        while Instant::now() < deadline && self.running(deadline)? {
            std::thread::sleep(Duration::from_millis(2));
        }
        if self.running(deadline)? {
            return Err("Git process-group cleanup timed out".into());
        }
        self.reap()
    }

    fn reap(&mut self) -> Result<ExitStatus, String> {
        // No child reaping precedes the last group signal/observation.
        let status = self
            .child
            .as_mut()
            .expect("owned Git child")
            .wait()
            .map_err(|e| format!("Reap Git child: {e}"))?;
        self.child = None;
        Ok(status)
    }

    fn signal(&self, signal: Signal) -> Result<(), String> {
        match rustix::process::kill_process_group(self.pid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(format!("Signal owned Git group: {error}")),
        }
    }

    fn running(&self, deadline: Instant) -> Result<bool, String> {
        super::group::running(self.pid.as_raw_nonzero().get() as u32, deadline)
    }

    pub fn drain(&mut self, limit: usize) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_millis(250);
        while !self.streams.closed() && Instant::now() < deadline {
            self.streams.read(limit)?;
        }
        if !self.streams.closed() {
            return Err("Git output did not close after cleanup".into());
        }
        Ok(())
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = self.signal(Signal::KILL);
            // A library owner can be dropped outside the guarded application
            // exit path. A reaper thread retains that child off the UI thread.
            let workspace = self._workspace.take();
            std::thread::spawn(move || {
                let _ = child.wait();
                drop(workspace);
            });
        }
    }
}

#[cfg(test)]
mod tests;
