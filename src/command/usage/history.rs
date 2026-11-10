use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::{Error, Store, Usage, WINDOW};
use crate::command::Command;

const SHUTDOWN_WAIT: Duration = Duration::from_millis(250);

struct Event {
    sequence: u64,
    name: String,
}

#[derive(Default)]
struct Memory {
    usage: Usage,
    pending: VecDeque<Event>,
    next: u64,
    enabled: bool,
    notice: Option<String>,
    last_error: Option<String>,
}

impl Memory {
    fn error(&mut self, error: &impl ToString) {
        let message = error.to_string();
        if self.last_error.as_ref() != Some(&message) {
            self.notice = Some(message.clone());
        }
        self.last_error = Some(message);
    }
}

/// Explicit session state owner. Default construction is isolated and performs
/// no discovery or IO. Open reads one bounded path; publication uses at most one
/// worker and one 256-event pending queue, including events already in flight.
pub struct History {
    memory: Arc<Mutex<Memory>>,
    store: Option<Store>,
    task: Option<JoinHandle<Store>>,
    last_attempt: u64,
}

impl Default for History {
    fn default() -> Self {
        Self {
            memory: Arc::new(Mutex::new(Memory::default())),
            store: None,
            task: None,
            last_attempt: 0,
        }
    }
}

impl std::fmt::Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History").finish_non_exhaustive()
    }
}

impl History {
    /// Load before interactive input. Missing state stays absent; rejected
    /// state remains untouched and reports a nonfatal diagnostic through poll.
    pub fn open(path: &Path) -> Self {
        let mut history = Self::default();
        match Store::open(path) {
            Ok((store, events)) => {
                history.store = Some(store);
                let mut memory = history.memory.lock().expect("command usage owner");
                memory.usage.events = events.into();
                memory.enabled = true;
            }
            Err(error) => history
                .memory
                .lock()
                .expect("command usage owner")
                .error(&error),
        }
        history
    }

    pub(crate) fn snapshot(&self) -> Usage {
        self.memory
            .lock()
            .expect("command usage owner")
            .usage
            .clone()
    }

    pub(crate) fn record(&mut self, command: Command) {
        let mut memory = self.memory.lock().expect("command usage owner");
        memory.usage.record(command);
        // Qualified experimental calls participate in session ranking. Native
        // schema 1 retains its existing name grammar and publication bounds.
        if !memory.enabled || matches!(command, Command::PluginDefined(_)) {
            return;
        }
        let Some(next) = memory.next.checked_add(1) else {
            memory.enabled = false;
            memory.error(&Error::Unavailable(
                "event identity exhausted; persistence suspended",
            ));
            return;
        };
        memory.next = next;
        if memory.pending.len() == WINDOW {
            memory.pending.pop_front();
        }
        memory.pending.push_back(Event {
            sequence: next,
            name: command.name().to_owned(),
        });
    }

    /// Request one nonblocking publication attempt from owned idle work. A busy
    /// lock is retried only on another explicit request or newer named usage.
    pub fn publish(&mut self) {
        self.start(true);
    }

    pub(crate) fn publish_new(&mut self) {
        self.start(false);
    }

    fn start(&mut self, retry: bool) {
        self.reap();
        if self.task.is_some() || self.store.is_none() {
            return;
        }
        let memory = self.memory.lock().expect("command usage owner");
        if !memory.enabled
            || memory.pending.is_empty()
            || (!retry && memory.next == self.last_attempt)
        {
            return;
        }
        let through = memory.next;
        let batch: Vec<_> = memory
            .pending
            .iter()
            .map(|event| event.name.clone())
            .collect();
        drop(memory);
        self.last_attempt = through;
        let mut store = self.store.take().expect("idle store");
        let shared = Arc::clone(&self.memory);
        match thread::Builder::new()
            .name("command-usage".into())
            .spawn(move || {
                let result = store.publish(&batch);
                complete(&shared, result, through);
                store
            }) {
            Ok(task) => self.task = Some(task),
            Err(_) => {
                let mut memory = self.memory.lock().expect("command usage owner");
                memory.enabled = false;
                memory.error(&Error::Unavailable(
                    "cannot start publication worker; persistence suspended",
                ));
            }
        }
    }

    fn reap(&mut self) {
        if !self.task.as_ref().is_some_and(JoinHandle::is_finished) {
            return;
        }
        match self.task.take().expect("finished worker").join() {
            Ok(store) => self.store = Some(store),
            Err(_) => {
                let mut memory = self.memory.lock().expect("command usage owner");
                memory.enabled = false;
                memory.error(&Error::Unavailable(
                    "publication worker failed; persistence suspended",
                ));
            }
        }
    }

    /// Collect a completed diagnostic without waiting or scheduling a retry.
    pub fn poll(&mut self) -> Option<String> {
        self.reap();
        self.memory
            .lock()
            .expect("command usage owner")
            .notice
            .take()
    }

    pub(crate) fn remaining(&self) -> Option<Duration> {
        self.task.as_ref().map(|_| Duration::from_millis(10))
    }

    /// Wait at most 250 ms for owned work and one final publication attempt.
    /// Errors retain editing state; an unresolved worker keeps its sole slot.
    pub fn flush(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + SHUTDOWN_WAIT;
        self.wait(deadline)?;
        self.publish();
        self.wait(deadline)?;
        let memory = self.memory.lock().expect("command usage owner");
        if let Some(error) = &memory.last_error {
            return Err(error.clone());
        }
        if !memory.pending.is_empty() {
            return Err(Error::Unavailable("publication remains pending").to_string());
        }
        drop(memory);
        Ok(())
    }

    fn wait(&mut self, deadline: Instant) -> Result<(), String> {
        while self.task.is_some() {
            self.reap();
            if self.task.is_none() {
                break;
            }
            if Instant::now() >= deadline {
                return Err(Error::Unavailable("shutdown publication deadline reached").to_string());
            }
            thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }
}

fn complete(shared: &Mutex<Memory>, result: Result<Vec<String>, Error>, through: u64) {
    let mut memory = shared.lock().expect("command usage owner");
    match result {
        Ok(events) if memory.enabled => {
            while memory
                .pending
                .front()
                .is_some_and(|event| event.sequence <= through)
            {
                memory.pending.pop_front();
            }
            let mut usage = Usage {
                events: events.into(),
            };
            for event in &memory.pending {
                usage.append(event.name.clone());
            }
            memory.usage = usage;
            memory.notice = None;
            memory.last_error = None;
        }
        Ok(_) => {}
        Err(error) => {
            if error.suspends() {
                memory.enabled = false;
            }
            memory.error(&error);
        }
    }
}
