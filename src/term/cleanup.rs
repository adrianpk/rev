use super::{Waker, process::Process, readiness::Readiness};
use std::{
    io,
    process::ExitStatus,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
    time::{Duration, Instant},
};

/// At most one cleanup job per admitted session. A failed job returns process
/// ownership; the registry keeps its capacity and diagnostic until a retry.
pub(super) struct Cleanup {
    owned: Option<Arc<Mutex<Option<Process>>>>,
    result: Receiver<io::Result<ExitStatus>>,
    deadline: Instant,
}

impl Cleanup {
    pub fn start(
        process: &mut Option<Process>,
        readiness: Option<Readiness>,
        exited: bool,
        ready: Arc<AtomicBool>,
        wake: Waker,
    ) -> io::Result<Self> {
        let requested = Instant::now();
        let owned = Arc::new(Mutex::new(None::<Process>));
        let worker = owned.clone();
        let (send, result) = mpsc::sync_channel(1);
        let (start, receive) = mpsc::sync_channel(1);
        // Transfer only after spawn succeeds. A rejected worker cannot drop a
        // live PTY (whose upstream destructor waits) on the caller's thread.
        std::thread::Builder::new()
            .name("terminal-cleanup".into())
            .spawn(move || {
                let Ok(readiness) = receive.recv() else {
                    return;
                };
                drop(readiness);
                let mut process = worker
                    .lock()
                    .expect("cleanup owner")
                    .take()
                    .expect("process");
                let outcome = process.stop(exited, requested);
                match &outcome {
                    Ok(_) => drop(process),
                    Err(error) => {
                        eprintln!("Terminal cleanup failed; process remains owned: {error}");
                        *worker.lock().expect("cleanup owner") = Some(process);
                    }
                }
                let _ = send.send(outcome);
                ready.store(true, Ordering::Release);
                wake();
            })?;
        *owned.lock().expect("cleanup owner") = process.take();
        start.send(readiness).expect("cleanup worker started");
        Ok(Self {
            owned: Some(owned),
            result,
            deadline: requested + Duration::from_secs(2),
        })
    }

    pub fn poll(&self) -> Option<io::Result<ExitStatus>> {
        match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("terminal cleanup worker failed"),
        }
    }

    pub fn process(&self) -> Option<Process> {
        self.owned
            .as_ref()
            .expect("cleanup owner")
            .lock()
            .expect("cleanup owner")
            .take()
    }

    pub fn wait(&self) -> io::Result<io::Result<ExitStatus>> {
        self.result
            .recv_timeout(self.deadline.saturating_duration_since(Instant::now()))
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("Terminal cleanup remains pending: {error}"),
                )
            })
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let owned = self.owned.take().expect("cleanup owner");
        // An active worker retains the last Arc. A completed failed job must
        // also retire on a worker: upstream PTY Drop can wait for a stuck child.
        if let Ok(owned) = Arc::try_unwrap(owned)
            && let Some(process) = owned.into_inner().expect("cleanup owner")
        {
            std::thread::spawn(move || {
                eprintln!(
                    "Terminal shutdown still owns failed cleanup for PID {}",
                    process.pid
                );
                drop(process);
            });
        }
    }
}
