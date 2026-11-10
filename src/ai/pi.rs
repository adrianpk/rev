//! Owned Pi 1.3.0 operations and compatible idle connections. Pi owns history;
//! native state retains only binding metadata and bounded result/protocol data.
mod directory;
mod frame;
mod history;
mod io;
mod protocol;
mod resource;
mod worker;
pub(crate) use directory::{private_roots, source_path};

use crate::config::Pi;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Mailbox {
    result: Option<Answer>,
    pending: Option<Request>,
    idle_until: Option<Instant>,
    closed: bool,
}
#[derive(Default)]
pub(super) struct Control {
    cancel: AtomicBool,
    accepted: AtomicBool,
    cleanup_failed: AtomicBool,
    reset_applied: AtomicBool,
    mailbox: Mutex<Mailbox>,
    wake: Mutex<Weak<io::Inbox>>,
}
impl Control {
    fn canceled(&self) -> bool {
        self.cancel.load(Ordering::Acquire)
    }
    fn wake(&self) {
        let inbox = self.wake.lock().expect("Pi wake owner").upgrade();
        if let Some(inbox) = inbox {
            inbox.wake();
        }
    }
}

#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub model: String,
    pub directory: bool,
    pub history: String,
}
/// Pi-reported model identifiers only; authentication/provider settings stay in Pi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub provider: String,
    pub id: String,
}
impl Model {
    pub fn reference(&self) -> String {
        format!("{}/{}", self.provider, self.id)
    }
}
enum Answer {
    Generated(Output),
    Models(Vec<Model>),
}
enum Purpose {
    Generate(String),
    Models,
}
struct Request {
    purpose: Purpose,
    epoch: u64,
    deadline: Instant,
}
struct Profile {
    directory: bool,
    retained: bool,
    state: Option<PathBuf>,
    reset: bool,
}

/// History options for one explicit managed directory invocation.
pub struct History {
    /// Host XDG state namespace; None resolves the normal user namespace.
    pub state: Option<PathBuf>,
    /// Request fresh owned history instead of reusing a compatible binding.
    pub reset: bool,
}

/// One invocation slot and, for managed directory operations, its idle child.
/// No request can queue behind another active invocation or a stopping child.
pub struct Operation {
    epoch: u64,
    policy: Pi,
    root: PathBuf,
    control: Arc<Control>,
    task: Option<JoinHandle<Result<(), String>>>,
}
impl Operation {
    /// Fresh input-only invocation, independent of directory persistence settings.
    pub fn start(policy: Pi, root: PathBuf, message: String, epoch: u64) -> Result<Self, String> {
        Self::launch(
            policy,
            root,
            Purpose::Generate(message),
            epoch,
            Profile {
                directory: false,
                retained: false,
                state: None,
                reset: false,
            },
        )
    }
    /// Explicit fresh directory grant, without retained process/history state.
    pub fn directory(
        policy: Pi,
        root: PathBuf,
        message: String,
        epoch: u64,
    ) -> Result<Self, String> {
        Self::launch(
            policy,
            root,
            Purpose::Generate(message),
            epoch,
            Profile {
                directory: true,
                retained: false,
                state: None,
                reset: false,
            },
        )
    }
    /// Managed directory history using the host's explicit state/reset options.
    pub fn retained(
        policy: Pi,
        root: PathBuf,
        message: String,
        epoch: u64,
        history: History,
    ) -> Result<Self, String> {
        Self::launch(
            policy,
            root,
            Purpose::Generate(message),
            epoch,
            Profile {
                directory: true,
                retained: true,
                state: history.state,
                reset: history.reset,
            },
        )
    }
    /// Read Pi's model catalog in a fresh tool-free memory-only child.
    pub fn models(policy: Pi, root: PathBuf, epoch: u64) -> Result<Self, String> {
        Self::launch(
            policy,
            root,
            Purpose::Models,
            epoch,
            Profile {
                directory: false,
                retained: false,
                state: None,
                reset: false,
            },
        )
    }
    fn launch(
        policy: Pi,
        root: PathBuf,
        purpose: Purpose,
        epoch: u64,
        profile: Profile,
    ) -> Result<Self, String> {
        admitted(&policy, &purpose, epoch)?;
        let root_handle = if profile.directory {
            let handle = rustix::fs::open(
                &root,
                rustix::fs::OFlags::RDONLY
                    | rustix::fs::OFlags::DIRECTORY
                    | rustix::fs::OFlags::NOFOLLOW
                    | rustix::fs::OFlags::CLOEXEC,
                rustix::fs::Mode::empty(),
            )
            .map_err(|_| "Pi directory root could not be pinned")?;
            if root
                .canonicalize()
                .map_err(|_| "Pi directory root unavailable")?
                .as_os_str()
                != root.as_os_str()
            {
                return Err("Pi directory root must retain its canonical identity".into());
            }
            Some(std::fs::File::from(handle))
        } else {
            None
        };
        let request = Request {
            purpose,
            epoch,
            deadline: Instant::now() + Duration::from_millis(policy.request_timeout_ms as u64),
        };
        let control = Arc::new(Control::default());
        let worker_control = Arc::clone(&control);
        let worker_policy = policy.clone();
        let worker_root = root.clone();
        let task = std::thread::Builder::new()
            .name("rev-pi".into())
            .spawn(move || {
                let result = worker::Launch {
                    policy: worker_policy,
                    root: worker_root,
                    root_handle,
                    request,
                    profile,
                }
                .run(&worker_control);
                let mut mailbox = worker_control.mailbox.lock().expect("Pi mailbox");
                mailbox.closed = true;
                let result = match result {
                    Ok(answer) => {
                        if let Some(answer) = answer {
                            mailbox.result = Some(answer);
                        }
                        Ok(())
                    }
                    Err(error) => Err(error),
                };
                drop(mailbox);
                result
            })
            .map_err(|_| "Pi supervisor could not start")?;
        Ok(Self {
            epoch,
            policy,
            root,
            control,
            task: Some(task),
        })
    }
    pub fn matches(&self, policy: &Pi, root: &std::path::Path) -> bool {
        self.policy == *policy && self.root == root
    }
    pub fn can_reuse(&self) -> bool {
        let state = self.control.mailbox.lock().expect("Pi mailbox");
        !state.closed
            && !self.is_canceled()
            && state.idle_until.is_some_and(|end| Instant::now() < end)
            && state.result.is_none()
    }
    pub fn resume(&mut self, message: String, epoch: u64) -> Result<(), String> {
        self.begin(Purpose::Generate(message), epoch)
    }
    pub fn query_models(&mut self, epoch: u64) -> Result<(), String> {
        self.begin(Purpose::Models, epoch)
    }
    fn begin(&mut self, purpose: Purpose, epoch: u64) -> Result<(), String> {
        admitted(&self.policy, &purpose, epoch)?;
        let mut state = self.control.mailbox.lock().expect("Pi mailbox");
        if state.closed
            || self.is_canceled()
            || state.result.is_some()
            || state.pending.is_some()
            || state.idle_until.is_none_or(|end| Instant::now() >= end)
        {
            return Err("Pi idle connection expired or is stopping; retry after cleanup".into());
        }
        state.pending = Some(Request {
            purpose,
            epoch,
            deadline: Instant::now() + Duration::from_millis(self.policy.request_timeout_ms as u64),
        });
        state.idle_until = None;
        self.epoch = epoch;
        self.control.accepted.store(false, Ordering::Release);
        drop(state);
        self.control.wake();
        Ok(())
    }
    pub fn reset_applied(&self) -> bool {
        self.control.reset_applied.load(Ordering::Acquire)
    }
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
        self.control.wake();
    }
    pub fn is_canceled(&self) -> bool {
        self.control.canceled()
    }
    pub fn worker_finished(&self) -> bool {
        self.task.as_ref().is_none_or(JoinHandle::is_finished)
    }
    pub fn finished(&self) -> bool {
        self.control
            .mailbox
            .lock()
            .expect("Pi mailbox")
            .result
            .is_some()
            || self.worker_finished()
    }
    fn answer(&mut self) -> Option<Result<Answer, String>> {
        let ready = self
            .control
            .mailbox
            .lock()
            .expect("Pi mailbox")
            .result
            .take();
        let result = if let Some(answer) = ready {
            Some(Ok(answer))
        } else if self.worker_finished() {
            match self.task.take()?.join() {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(Err(e)),
                Err(_) => Some(Err("Pi supervisor failed".into())),
            }
        } else {
            None
        };
        result.map(|r| {
            if self.is_canceled() {
                Err("Pi invocation canceled".into())
            } else {
                r
            }
        })
    }
    pub fn take(&mut self) -> Option<Result<Output, String>> {
        self.answer().map(|r| {
            r.and_then(|a| match a {
                Answer::Generated(out) => Ok(out),
                Answer::Models(_) => Err("Pi operation returned a model catalog".into()),
            })
        })
    }
    pub fn take_models(&mut self) -> Option<Result<Vec<Model>, String>> {
        self.answer().map(|r| {
            r.and_then(|a| match a {
                Answer::Models(models) => Ok(models),
                Answer::Generated(_) => Err("Pi operation returned generated text".into()),
            })
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
fn admitted(policy: &Pi, purpose: &Purpose, epoch: u64) -> Result<(), String> {
    if let Purpose::Generate(message) = purpose {
        if message.len() > policy.max_input_bytes {
            return Err("Pi editor envelope exceeds max_input_bytes".into());
        }
        frame::encode(&json!({"id":format!("{epoch}:prompt"),"type":"prompt","message":message}))?;
    }
    Ok(())
}
