use std::collections::VecDeque;
use std::io::{BufReader, Read, Write};
use std::net::Shutdown;
use std::os::{fd::OwnedFd, unix::net::UnixStream};
use std::process::{Child, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::{Control, frame};

#[derive(Default)]
struct Queue {
    records: VecDeque<Vec<u8>>,
    bytes: usize,
    closed: bool,
    woken: bool,
    failure: Option<String>,
}

#[derive(Default)]
pub(super) struct Inbox {
    queue: Mutex<Queue>,
    ready: Condvar,
    // Diagnostics stay separate from protocol and are never copied into a
    // prompt, configuration or user-facing provider error.
    stderr: Mutex<VecDeque<u8>>,
}

impl Inbox {
    pub fn wake(&self) {
        self.queue.lock().expect("Pi wake queue").woken = true;
        self.ready.notify_all();
    }
    pub fn next(&self, wait: Duration) -> Result<Option<Vec<u8>>, String> {
        let mut queue = self.queue.lock().expect("Pi inbox");
        if queue.records.is_empty() && !queue.closed && queue.failure.is_none() && !queue.woken {
            queue = self.ready.wait_timeout(queue, wait).expect("Pi wait").0;
        }
        queue.woken = false;
        if let Some(error) = &queue.failure {
            return Err(error.clone());
        }
        if let Some(bytes) = queue.records.pop_front() {
            queue.bytes -= bytes.len() + 2;
            return Ok(Some(bytes));
        }
        let closed = queue.closed;
        drop(queue);
        if closed {
            return Err("Pi stdout closed before the operation completed".into());
        }
        Ok(None)
    }

    fn close(&self, failure: Option<String>) {
        let mut queue = self.queue.lock().expect("Pi inbox");
        queue.closed = true;
        queue.failure = failure;
        drop(queue);
        self.ready.notify_all();
    }

    fn push(&self, bytes: Vec<u8>) -> bool {
        let mut queue = self.queue.lock().expect("Pi inbox");
        if queue.records.len() >= 256 || queue.bytes + bytes.len() + 2 > frame::QUEUE {
            queue.failure = Some("Pi protocol queue exceeds 4 MiB or 256 records".into());
            drop(queue);
            self.ready.notify_all();
            return false;
        }
        queue.bytes += bytes.len() + 2;
        queue.records.push_back(bytes);
        drop(queue);
        self.ready.notify_one();
        true
    }
}

pub(super) struct Pipes {
    input: UnixStream,
    output: UnixStream,
    error: UnixStream,
}

// Process ownership survives early stream-setup failures and supervisor unwind.
// A successful normal stop makes the drop recheck nonblocking.
pub(super) struct Owned {
    child: Child,
    stop_attempted: bool,
    control: Arc<Control>,
}

impl Owned {
    pub fn new(child: Child, control: Arc<Control>) -> Self {
        Self {
            child,
            stop_attempted: false,
            control,
        }
    }
    pub fn limit_history(&self) -> Result<(), String> {
        let pid = rustix::process::Pid::from_child(&self.child);
        rustix::process::prlimit(
            Some(pid),
            rustix::process::Resource::Fsize,
            rustix::process::Rlimit {
                current: Some(super::history::SESSION_BYTES),
                maximum: Some(super::history::SESSION_BYTES),
            },
        )
        .map_err(|_| "Pi history file limit could not be enforced")?;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        self.stop_attempted = true;
        let result = stop(&mut self.child);
        if result.is_err() {
            self.control
                .cleanup_failed
                .store(true, std::sync::atomic::Ordering::Release);
        }
        result
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.stop_attempted {
            let _ = self.stop();
        }
    }
}

struct Readers<'a>(&'a UnixStream, &'a UnixStream);

impl Drop for Readers<'_> {
    fn drop(&mut self) {
        let _ = self.0.shutdown(Shutdown::Both);
        let _ = self.1.shutdown(Shutdown::Both);
    }
}

impl Pipes {
    pub fn new() -> Result<(Self, [Stdio; 3]), String> {
        let (input, stdin) = pair()?;
        let (output, stdout) = pair()?;
        let (error, stderr) = pair()?;
        input
            .set_write_timeout(Some(Duration::from_millis(250)))
            .map_err(|_| "Pi stdin timeout setup failed")?;
        Ok((
            Self {
                input,
                output,
                error,
            },
            [stdin, stdout, stderr],
        ))
    }

    /// Both streams are continuously drained while the supervisor writes.
    /// Socket shutdown also releases readers if a child inherited a descriptor.
    pub fn drive<T>(
        mut self,
        child: &mut Owned,
        run: impl FnOnce(&Inbox, &mut UnixStream, &mut Child) -> Result<T, String>,
    ) -> Result<T, String> {
        let output = self
            .output
            .try_clone()
            .map_err(|_| "Pi stdout clone failed")?;
        let mut error = self
            .error
            .try_clone()
            .map_err(|_| "Pi stderr clone failed")?;
        let inbox = Arc::new(Inbox::default());
        *child.control.wake.lock().expect("Pi wake owner") = Arc::downgrade(&inbox);
        std::thread::scope(|scope| {
            // Release scoped readers before joining them even if setup/run
            // unwinds. Inherited descriptors cannot retain these local reads.
            let _readers = Readers(&self.output, &self.error);
            let reader = Arc::clone(&inbox);
            scope.spawn(move || {
                let mut output = BufReader::new(output);
                loop {
                    match frame::read(&mut output) {
                        Ok(Some(bytes)) => {
                            if !reader.push(bytes) {
                                return;
                            }
                        }
                        Ok(None) => {
                            reader.close(None);
                            return;
                        }
                        Err(_) => {
                            reader.close(Some(
                                "Pi stdout has invalid or oversized LF framing".into(),
                            ));
                            return;
                        }
                    }
                }
            });
            let diagnostic = Arc::clone(&inbox);
            scope.spawn(move || {
                let mut bytes = [0; 4096];
                while let Ok(n) = error.read(&mut bytes) {
                    if n == 0 {
                        break;
                    }
                    let mut tail = diagnostic.stderr.lock().expect("Pi diagnostics");
                    let excess = (tail.len() + n).saturating_sub(64 * 1024);
                    tail.drain(..excess);
                    tail.extend(&bytes[..n]);
                }
            });
            let result = run(&inbox, &mut self.input, &mut child.child);
            let _ = self.input.shutdown(Shutdown::Write);
            let stopped = child.stop();
            let _ = self.input.shutdown(Shutdown::Both);
            let _ = self.output.shutdown(Shutdown::Both);
            let _ = self.error.shutdown(Shutdown::Both);
            stopped.and(result)
        })
    }
}

fn pair() -> Result<(UnixStream, Stdio), String> {
    let (local, child) = UnixStream::pair().map_err(|_| "Pi stream allocation failed")?;
    Ok((local, Stdio::from(OwnedFd::from(child))))
}

pub(super) fn send(input: &mut UnixStream, value: &serde_json::Value) -> Result<(), String> {
    let bytes = frame::encode(value)?;
    let deadline = Instant::now() + Duration::from_millis(250);
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err("Pi stdin write deadline exceeded".into());
        }
        input
            .set_write_timeout(Some(remaining))
            .map_err(|_| "Pi stdin timeout setup failed")?;
        match input.write(&bytes[offset..]) {
            Ok(0) => return Err("Pi stdin closed".into()),
            Ok(n) => offset += n,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err("Pi stdin stalled or closed".into()),
        }
    }
    input.flush().map_err(|_| "Pi stdin flush failed".into())
}

fn stop(child: &mut Child) -> Result<(), String> {
    let grace = Instant::now() + Duration::from_secs(2);
    loop {
        if child.try_wait().map_err(|_| "Pi wait failed")?.is_some() {
            return Ok(());
        }
        if Instant::now() >= grace {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child
        .kill()
        .map_err(|_| "Pi owned child could not be killed")?;
    let reap = Instant::now() + Duration::from_secs(2);
    while Instant::now() < reap {
        if child.try_wait().map_err(|_| "Pi reap failed")?.is_some() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("Pi owned child did not reap within two seconds".into())
}

pub(super) fn next(
    inbox: &Inbox,
    control: &Control,
    deadline: Instant,
) -> Result<Option<serde_json::Value>, String> {
    if control.canceled() {
        return Err("Pi invocation canceled".into());
    }
    if Instant::now() >= deadline {
        return Err("Pi invocation deadline exceeded".into());
    }
    inbox
        .next(Duration::from_millis(20))?
        .map(|bytes| serde_json::from_slice(&bytes).map_err(|_| "Pi emitted invalid JSON".into()))
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Slow consumers cannot accumulate unbounded tiny events or large frames;
    // dequeue returns byte ownership before another record can be admitted.
    #[test]
    fn queue_bounds() {
        let inbox = Inbox::default();
        assert!(inbox.push(vec![0; frame::RECORD - 2]));
        assert!(inbox.push(vec![0; frame::RECORD - 2]));
        assert!(inbox.next(Duration::ZERO).unwrap().is_some());
        assert!(inbox.push(vec![0; frame::RECORD - 2]));
        assert!(!inbox.push(vec![1]));
        assert!(inbox.next(Duration::ZERO).is_err());
        let inbox = Inbox::default();
        for _ in 0..256 {
            assert!(inbox.push(Vec::new()));
        }
        assert!(!inbox.push(Vec::new()));
    }
}
