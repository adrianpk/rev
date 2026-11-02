use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::json;

use super::{
    Capabilities, Status,
    documents::Documents,
    frame::Message,
    inbox::{Event, Inbox},
    io::{self, Outbox, Pipes},
    protocol, roots,
};
use crate::config::{Lsp, Server};

pub(super) struct Connection {
    pub status: Arc<Mutex<Status>>,
    pub stderr: io::Stderr,
    pub inbox: Arc<Inbox>,
    worker: Option<JoinHandle<()>>,
    pub retiring: bool,
    pub generation: u64,
}

impl Connection {
    pub fn start(
        server: Server,
        root: PathBuf,
        policy: Lsp,
        generation: u64,
    ) -> Result<Self, String> {
        let status = Arc::new(Mutex::new(Status::Starting));
        let stderr = Arc::new(Mutex::new(VecDeque::new()));
        let inbox = Arc::new(Inbox::default());
        let mut supervisor = Supervisor {
            server,
            root,
            policy: policy.clone(),
            status: status.clone(),
            inbox: inbox.clone(),
            stopping: false,
            deadline: None,
            capabilities: None,
            documents: Documents::default(),
            requests: super::requests::Requests::new(&policy),
        };
        let retained = stderr.clone();
        let worker = std::thread::Builder::new()
            .name("rev-lsp".into())
            .spawn(move || {
                let result = supervisor.run(retained);
                supervisor.inbox.close();
                if let Err(error) = result {
                    supervisor.set(Status::Failed(error));
                }
            })
            .map_err(|e| format!("LSP worker: {e}"))?;
        Ok(Self {
            status,
            stderr,
            inbox,
            worker: Some(worker),
            retiring: false,
            generation,
        })
    }

    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }

    pub fn join(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.inbox.stop();
        self.join();
    }
}

struct Supervisor {
    server: Server,
    root: PathBuf,
    policy: Lsp,
    status: Arc<Mutex<Status>>,
    inbox: Arc<Inbox>,
    stopping: bool,
    deadline: Option<Instant>,
    capabilities: Option<Capabilities>,
    documents: Documents,
    requests: super::requests::Requests,
}

impl Supervisor {
    fn run(&mut self, stderr: io::Stderr) -> Result<(), String> {
        let root = roots::uri(&self.root)?;
        let timeout = Duration::from_millis(self.policy.request_timeout_ms as u64);
        let (pipes, stdio) = Pipes::new(timeout)?;
        let _payload_lease = super::install::launch_lease(&self.server, &self.root)?;
        let mut child = match self.spawn(stdio) {
            Ok(child) => child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.set(Status::Missing(self.server.command.clone()));
                return Ok(());
            }
            Err(error) => return Err(format!("LSP launch: {error}")),
        };
        self.deadline =
            Some(Instant::now() + Duration::from_millis(self.policy.initialize_timeout_ms as u64));
        let mut result = Ok(());
        let io_result = io::drive(
            pipes,
            self.inbox.clone(),
            stderr,
            self.policy.max_message_bytes,
            |output| {
                result = self.communicate(&output, &root);
            },
        );
        reap(&mut child);
        io_result?;
        result
    }

    fn spawn(&self, stdio: [std::process::Stdio; 3]) -> std::io::Result<Child> {
        let [stdin, stdout, stderr] = stdio;
        Command::new(&self.server.command)
            .args(&self.server.args)
            .envs(&self.server.env)
            .current_dir(&self.root)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
    }

    fn set(&self, status: Status) {
        *self.status.lock().expect("LSP status lock") = status;
    }

    fn communicate(&mut self, output: &Outbox, root: &str) -> Result<(), String> {
        output.send(&protocol::initialize(&self.server, root))?;
        loop {
            self.requests.sweep(output)?;
            let remaining = self
                .deadline
                .into_iter()
                .chain(self.requests.deadline())
                .min()
                .map(|deadline| deadline.saturating_duration_since(Instant::now()));
            if self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return self.timeout();
            }
            if self.event(
                self.inbox.next(remaining, self.capabilities.is_some()),
                output,
                root,
            )? {
                return Ok(());
            }
        }
    }

    fn event(&mut self, event: Event, output: &Outbox, root: &str) -> Result<bool, String> {
        match event {
            Event::Message(message) => self.message(*message, output, root).map(|_| false),
            Event::Document(event, permit) => self.document(*event, permit, output).map(|_| false),
            Event::Completion(query, permit) => {
                self.complete(*query, permit, output).map(|_| false)
            }
            control => self.control(control, output),
        }
    }

    fn complete(
        &mut self,
        query: super::completion::Query,
        permit: super::budget::Permit,
        output: &Outbox,
    ) -> Result<(), String> {
        if self.stopping {
            return Ok(());
        }
        let caps = self
            .capabilities
            .as_ref()
            .ok_or("LSP completion before initialize")?;
        self.requests.start(query, permit, (output, caps))
    }

    fn document(
        &mut self,
        event: super::documents::Event,
        permit: super::budget::Permit,
        output: &Outbox,
    ) -> Result<(), String> {
        if self.stopping {
            return Ok(());
        }
        let caps = self
            .capabilities
            .as_ref()
            .ok_or("LSP document before initialize")?;
        self.documents.process(event, permit, output, caps)
    }

    fn control(&mut self, event: Event, output: &Outbox) -> Result<bool, String> {
        match event {
            Event::Stop => self.shutdown(output).map(|_| false),
            Event::Failed(_) if self.stopping => Ok(true),
            Event::Failed(error) => Err(error),
            Event::Timeout | Event::Cancelled => Ok(false),
            _ => unreachable!("document and protocol events have their own owners"),
        }
    }

    fn timeout(&self) -> Result<(), String> {
        if self.stopping {
            Ok(())
        } else {
            Err("LSP initialization timed out".into())
        }
    }

    fn shutdown(&mut self, output: &Outbox) -> Result<(), String> {
        if self.stopping {
            return Ok(());
        }
        self.stopping = true;
        self.set(Status::Stopping);
        self.deadline =
            Some(Instant::now() + Duration::from_millis(self.policy.shutdown_timeout_ms as u64));
        output.send(&json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}))
    }

    fn message(&mut self, message: Message, output: &Outbox, root: &str) -> Result<(), String> {
        if message.method.is_some() {
            if message.id.is_some() {
                output.send(&protocol::request(&message, &self.server, root))?;
            }
            return Ok(());
        }
        if message.id == Some(json!(2)) && self.stopping {
            return output.send(&json!({"jsonrpc":"2.0","method":"exit"}));
        }
        if message.id == Some(json!(1)) && !self.stopping && self.deadline.is_some() {
            return self.initialized(message, output);
        }
        self.requests.response(message);
        Ok(())
    }

    fn initialized(&mut self, message: Message, output: &Outbox) -> Result<(), String> {
        if message.error.is_some() {
            return Err("LSP initialize rejected by server".into());
        }
        let capabilities =
            protocol::capabilities(message.result.ok_or("LSP initialize lacks result")?)?;
        output.send(&json!({"jsonrpc":"2.0","method":"initialized","params":{}}))?;
        self.set(Status::Ready(capabilities.clone()));
        self.capabilities = Some(capabilities);
        self.deadline = None;
        Ok(())
    }
}

fn reap(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
}
