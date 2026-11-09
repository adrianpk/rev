use rustix::{
    event::{EventfdFlags, PollFd, PollFlags, eventfd, poll},
    fd::OwnedFd,
};
use std::{
    fs::File,
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
};

/// One coalesced notification per service boundary. The callback may wake the
/// native host input owner; it must not read input, render or block.
pub type Waker = Arc<dyn Fn() + Send + Sync>;

pub(super) struct Readiness {
    control: Arc<OwnedFd>,
    stop: Arc<AtomicBool>,
    writable: Arc<AtomicBool>,
    child_live: Arc<AtomicBool>,
    readable: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Readiness {
    pub fn new(
        file: File,
        child: Arc<OwnedFd>,
        ready: Arc<AtomicBool>,
        wake: Waker,
    ) -> io::Result<Self> {
        let control = Arc::new(eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?);
        let stop = Arc::new(AtomicBool::new(false));
        let writable = Arc::new(AtomicBool::new(false));
        let child_live = Arc::new(AtomicBool::new(true));
        let readable = Arc::new(AtomicBool::new(true));
        let worker = Worker {
            file,
            child,
            control: std::sync::Arc::clone(&control),
            stop: std::sync::Arc::clone(&stop),
            writable: std::sync::Arc::clone(&writable),
            child_live: std::sync::Arc::clone(&child_live),
            readable: std::sync::Arc::clone(&readable),
            ready,
            wake,
        };
        let thread = std::thread::Builder::new()
            .name("terminal-readiness".into())
            .spawn(move || worker.run())?;
        Ok(Self {
            control,
            stop,
            writable,
            child_live,
            readable,
            thread: Some(thread),
        })
    }
    pub fn rearm(&self, writable: bool, child_live: bool, readable: bool) {
        self.writable.store(writable, Ordering::Release);
        self.child_live.store(child_live, Ordering::Release);
        self.readable.store(readable, Ordering::Release);
        signal(&self.control);
    }
}
impl Drop for Readiness {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        signal(&self.control);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("terminal readiness worker failed");
        }
    }
}
struct Worker {
    file: File,
    child: Arc<OwnedFd>,
    control: Arc<OwnedFd>,
    stop: Arc<AtomicBool>,
    writable: Arc<AtomicBool>,
    child_live: Arc<AtomicBool>,
    readable: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    wake: Waker,
}
impl Worker {
    fn run(self) {
        let mut armed = true;
        loop {
            if self.stop.load(Ordering::Acquire) {
                break;
            }
            let events = self.events(armed);
            let Some((control, active)) = events else {
                self.notify();
                break;
            };
            if control {
                drain(&self.control);
                armed = true;
            }
            if active {
                self.notify();
                armed = false;
            }
        }
    }
    fn events(&self, armed: bool) -> Option<(bool, bool)> {
        let mut fds = vec![PollFd::new(self.control.as_ref(), PollFlags::IN)];
        if armed {
            let flags = if self.writable.load(Ordering::Acquire) {
                PollFlags::IN | PollFlags::OUT
            } else {
                PollFlags::IN
            };
            if self.readable.load(Ordering::Acquire) {
                fds.push(PollFd::new(&self.file, flags));
            }
            if self.child_live.load(Ordering::Acquire) {
                fds.push(PollFd::new(self.child.as_ref(), PollFlags::IN));
            }
        }
        match poll(&mut fds, None) {
            Ok(_) => Some((
                !fds[0].revents().is_empty(),
                fds.iter().skip(1).any(|fd| !fd.revents().is_empty()),
            )),
            Err(rustix::io::Errno::INTR) => Some((false, false)),
            Err(_) => None,
        }
    }
    fn notify(&self) {
        if !self.ready.swap(true, Ordering::AcqRel) {
            (self.wake)();
        }
    }
}
fn signal(fd: &OwnedFd) {
    match rustix::io::write(fd, &1u64.to_ne_bytes()) {
        Ok(_) | Err(rustix::io::Errno::AGAIN) => {}
        Err(error) => panic!("terminal readiness control failed: {error}"),
    }
}
fn drain(fd: &OwnedFd) {
    let mut bytes = [0; 8];
    match rustix::io::read(fd, &mut bytes) {
        Ok(_) | Err(rustix::io::Errno::AGAIN) => {}
        Err(error) => panic!("terminal readiness control failed: {error}"),
    }
}
