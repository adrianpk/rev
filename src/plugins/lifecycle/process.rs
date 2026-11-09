use super::super::declarations::Package;
use rustix::{
    fd::OwnedFd,
    fs::{OFlags, fcntl_getfl, fcntl_setfl},
    process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid},
};
use std::{
    io,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Keep the leader unreaped through all group signals. This prevents reuse of
/// its numeric group identity while descendants are still being settled.
pub(super) struct Process {
    pub child: Option<Child>,
    pub exit: Option<OwnedFd>,
    pid: Pid,
    settled: bool,
}
pub(super) struct LaunchError {
    pub error: io::Error,
    pub cleanup: String,
    pub settled: bool,
    pub retained: Option<Process>,
}
impl From<io::Error> for LaunchError {
    fn from(error: io::Error) -> Self {
        Self {
            error,
            cleanup: "no admitted process".into(),
            settled: true,
            retained: None,
        }
    }
}
impl Process {
    pub fn start(package: &Package) -> Result<Self, LaunchError> {
        let child = Command::new(&package.program)
            .args(package.manifest.process.args.iter().map(|a| a.as_str()))
            .current_dir(
                package
                    .source
                    .parent()
                    .ok_or_else(|| io::Error::other("manifest directory missing"))?,
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()?;
        let pid = Pid::from_child(&child);
        let mut process = Self {
            child: Some(child),
            exit: None,
            pid,
            settled: false,
        };
        let setup = (|| {
            process.exit = Some(rustix::process::pidfd_open(
                pid,
                rustix::process::PidfdFlags::NONBLOCK,
            )?);
            nonblocking(
                process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stdin
                    .as_ref()
                    .expect("owned stdin"),
            )?;
            nonblocking(
                process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stdout
                    .as_ref()
                    .expect("owned stdout"),
            )?;
            nonblocking(
                process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stderr
                    .as_ref()
                    .expect("owned stderr"),
            )?;
            Ok::<_, io::Error>(())
        })();
        if let Err(error) = setup {
            let (cleanup, settled) = match process.stop() {
                Ok(cleanup) => (cleanup, true),
                Err(error) => (format!("owned group cleanup uncertain: {error}"), false),
            };
            return Err(LaunchError {
                error: io::Error::other(format!("pipe setup: {error}")),
                cleanup,
                settled,
                retained: if settled { None } else { Some(process) },
            });
        }
        Ok(process)
    }
    pub fn id(&self) -> u32 {
        u32::try_from(self.pid.as_raw_nonzero().get()).expect("spawned child has a positive PID")
    }
    fn exited(&self) -> io::Result<bool> {
        waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .map(|s| s.is_some())
        .map_err(Into::into)
    }
    fn signal(&self, signal: Signal) -> io::Result<()> {
        match kill_process_group(self.pid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
    fn quiet(&self, deadline: Instant, observation_deadline: Instant) -> io::Result<bool> {
        while Instant::now() < deadline {
            if self.exited()?
                && !crate::process::group::running(self.id(), observation_deadline)
                    .map_err(io::Error::other)?
            {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(false)
    }
    pub fn stop(&mut self) -> io::Result<String> {
        if self.settled {
            return Ok(
                "owned group reaped; escaped descendant settlement uncertain (unobservable)".into(),
            );
        }
        let child = self.child.as_mut().expect("owned child");
        child.stdin.take();
        child.stdout.take();
        child.stderr.take();
        self.signal(Signal::TERM)?;
        let start = Instant::now();
        let force_deadline = start + Duration::from_secs(4);
        // A scan straddling the grace boundary must not turn an ordinary TERM
        // timeout into an observation failure before the required escalation.
        if !self.quiet(start + Duration::from_secs(2), force_deadline)? {
            self.signal(Signal::KILL)?;
            if !self.quiet(force_deadline, force_deadline)? {
                return Err(io::Error::other("owned group exceeded force/reap deadline"));
            }
        }
        self.child.as_mut().expect("owned child").wait()?;
        self.settled = true;
        Ok("owned group reaped; escaped descendant settlement uncertain (unobservable)".into())
    }
}
fn nonblocking(fd: &impl std::os::fd::AsFd) -> io::Result<()> {
    fcntl_setfl(fd, fcntl_getfl(fd)? | OFlags::NONBLOCK).map_err(Into::into)
}
impl Drop for Process {
    fn drop(&mut self) {
        if !self.settled {
            let _ = self.signal(Signal::KILL);
            if let Some(mut child) = self.child.take() {
                child.stdin.take();
                child.stdout.take();
                child.stderr.take();
                // Only the supervisor owns Process. Its failed receipt is
                // already visible before this exceptional wait. The UI's
                // bounded shutdown barrier reports uncertainty while this
                // same worker retains an unobservable child for reaping.
                let _ = child.wait();
            }
        }
    }
}
