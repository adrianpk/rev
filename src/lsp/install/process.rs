use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, WaitId, WaitIdOptions, kill_process_group, waitid};

/// Retain the unreaped leader until every group signal is complete. NOWAIT is
/// essential: try_wait would free its PID and permit signaling a reused group.
pub(super) struct Process {
    child: Option<Child>,
    pid: Pid,
    status: Option<ExitStatus>,
}

impl Process {
    pub fn start(command: &mut Command) -> Result<Self, String> {
        let child = command
            .process_group(0)
            .spawn()
            .map_err(|e| format!("launch installation helper: {e}"))?;
        let pid = Pid::from_raw(child.id() as i32).expect("spawned child has a positive PID");
        Ok(Self {
            child: Some(child),
            pid,
            status: None,
        })
    }

    pub fn exited(&self) -> Result<bool, String> {
        if self.status.is_some() {
            return Ok(true);
        }
        waitid(
            WaitId::Pid(self.pid),
            WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT,
        )
        .map(|status| status.is_some())
        .map_err(|e| format!("observe owned helper: {e}"))
    }

    pub fn finish<T>(&mut self, result: Result<T, String>) -> Result<T, String> {
        if result.is_ok() && self.child.is_some() {
            let _ = self.quiet(Instant::now() + Duration::from_secs(2));
        }
        let cleanup = self.stop();
        match (result, cleanup) {
            (Ok(value), Ok(status)) if status.success() => Ok(value),
            (Ok(_), Ok(status)) => Err(format!("installation helper exited with {status}")),
            (Err(error), Ok(_)) => Err(error),
            (Ok(_), Err(cleanup)) => {
                Err(format!("installation helper cleanup uncertain: {cleanup}"))
            }
            (Err(error), Err(cleanup)) => {
                Err(format!("{error}; helper cleanup uncertain: {cleanup}"))
            }
        }
    }

    pub fn stop(&mut self) -> Result<ExitStatus, String> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        self.signal(Signal::TERM)?;
        let graceful = self
            .quiet(Instant::now() + Duration::from_secs(2))
            .unwrap_or(false);
        if !graceful {
            self.signal(Signal::KILL)?;
            if !self.quiet(Instant::now() + Duration::from_secs(2))? {
                return Err("owned process group did not stop within the force/reap bound".into());
            }
        }
        // quiet observes leader exit without reaping. wait cannot now block on
        // running work, and no future signal will use this process-group ID.
        let status = self
            .child
            .as_mut()
            .expect("owned unreaped helper")
            .wait()
            .map_err(|e| format!("reap owned helper: {e}"))?;
        self.status = Some(status);
        self.child = None;
        Ok(status)
    }

    fn signal(&self, signal: Signal) -> Result<(), String> {
        match kill_process_group(self.pid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(error) => Err(format!("signal owned helper group: {error}")),
        }
    }

    fn quiet(&self, deadline: Instant) -> Result<bool, String> {
        while Instant::now() < deadline {
            if self.exited()?
                && !crate::process::group::running(
                    self.pid.as_raw_nonzero().get() as u32,
                    deadline,
                )?
            {
                return Ok(true);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(false)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.child.is_some() {
            let _ = self.stop();
        }
    }
}
