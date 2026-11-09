use super::{Budget, Call, Connection, State, inert, process::Process, reserve};
use crate::plugins::{declarations, protocol::Handshake};
use rev_plugin_sdk::{
    identity::*,
    limits::Limits,
    methods::*,
    rpc::*,
    transport::{self, Decoder},
};
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use std::{
    collections::{BTreeMap, VecDeque},
    io::{self, Read, Write},
    sync::{Arc, atomic::Ordering, mpsc::Receiver},
    time::{Duration, Instant},
};

struct Frame {
    bytes: Vec<u8>,
    offset: usize,
    budget: Arc<Budget>,
}
impl Drop for Frame {
    fn drop(&mut self) {
        self.budget
            .bytes
            .fetch_sub(self.bytes.len() - self.offset, Ordering::AcqRel);
    }
}
struct Worker {
    package: declarations::Package,
    connection: Arc<Connection>,
    budget: Arc<Budget>,
    process: Process,
    handshake: Handshake,
    decoder: Decoder,
    startup: Instant,
    waiting: VecDeque<Call>,
    pending: BTreeMap<OpaqueId, Call>,
    output: VecDeque<Frame>,
    bytes: usize,
    stderr_open: bool,
    incoming: Option<Receiver<Call>>,
}

pub(super) fn run(
    package: declarations::Package,
    session: OpaqueId,
    generation: Generation,
    connection: Arc<Connection>,
    budget: Arc<Budget>,
    incoming: Receiver<Call>,
) {
    let start = Instant::now();
    if connection.stop.load(Ordering::Acquire) {
        drop(incoming);
        finish(
            &connection,
            &package,
            None,
            "no admitted process".into(),
            true,
        );
        return;
    }
    let result = Process::start(&package);
    let process = match result {
        Ok(process) => process,
        Err(error) => {
            connection.stop.store(true, Ordering::Release);
            drop(incoming);
            finish(
                &connection,
                &package,
                Some(format!("spawn failed: {}", error.error)),
                error.cleanup,
                error.settled,
            );
            drop(error.retained);
            return;
        }
    };
    {
        let mut shared = connection.shared.lock().expect("plugin supervisor owner");
        shared.pid = Some(process.id());
    }
    let handshake = Handshake::new(
        session,
        generation,
        Package {
            id: package.manifest.plugin.id.clone(),
            version: package.manifest.plugin.version.clone(),
        },
    )
    .and_then(|h| {
        h.commands(
            package
                .manifest
                .commands
                .iter()
                .map(|c| package.manifest.plugin.id.command(&c.name))
                .collect(),
        )
    });
    let handshake = match handshake {
        Ok(handshake) => handshake,
        Err(error) => {
            connection.stop.store(true, Ordering::Release);
            drop(incoming);
            let mut process = process;
            let (cleanup, settled) = cleanup(&mut process);
            finish(
                &connection,
                &package,
                Some(error.to_string()),
                cleanup,
                settled,
            );
            return;
        }
    };
    let mut worker = Worker {
        package,
        connection,
        budget,
        process,
        handshake,
        decoder: Decoder::new(Limits::default().frame_bytes as usize).expect("finite profile"),
        startup: start + Duration::from_secs(5),
        waiting: VecDeque::new(),
        pending: BTreeMap::new(),
        output: VecDeque::new(),
        bytes: 0,
        stderr_open: true,
        incoming: Some(incoming),
    };
    let result = worker.drive();
    // Authority is retired before cancellation output, signals or any wait.
    worker.connection.stop.store(true, Ordering::Release);
    worker.incoming.take();
    {
        let mut shared = worker
            .connection
            .shared
            .lock()
            .expect("plugin supervisor owner");
        shared.state = State::Stopping;
    }
    worker.connection.notify();
    worker.cancel();
    worker.waiting.clear();
    worker.pending.clear();
    worker.output.clear();
    worker.bytes = 0;
    let (cleanup, settled) = cleanup(&mut worker.process);
    finish(
        &worker.connection,
        &worker.package,
        result.err(),
        cleanup,
        settled,
    );
}
fn cleanup(process: &mut Process) -> (String, bool) {
    match process.stop() {
        Ok(cleanup) => (cleanup, true),
        Err(error) => (
            format!("owned group cleanup uncertain: {error}; supervisor retains child ownership"),
            false,
        ),
    }
}
fn finish(
    connection: &Connection,
    package: &declarations::Package,
    error: Option<String>,
    cleanup: String,
    settled: bool,
) {
    let state = if error.is_some() || !settled {
        State::Failed
    } else {
        State::Inactive
    };
    let diagnostic = error
        .map(|text| inert(&text.as_bytes()[..text.len().min(4096)]))
        .unwrap_or_default();
    let mut shared = connection.shared.lock().expect("plugin supervisor owner");
    shared.state = state;
    shared.diagnostic = diagnostic.clone();
    shared.cleanup = cleanup;
    shared.finished = settled;
    if shared.notices.len() == super::NOTICES {
        shared.notices.pop_front();
    }
    shared.notices.push_back(format!(
        "Plugin {}: {}{}{}",
        package.manifest.plugin.id,
        state.name(),
        if diagnostic.is_empty() { "" } else { ": " },
        diagnostic
    ));
    drop(shared);
    connection.notify();
}
impl Worker {
    fn drive(&mut self) -> Result<(), String> {
        self.queue(Message::Request(self.handshake.request().clone()))?;
        loop {
            if self.connection.stop.load(Ordering::Acquire) {
                return Ok(());
            }
            self.controls();
            let now = Instant::now();
            if (!self.handshake.accepted() && now >= self.startup)
                || self
                    .pending
                    .values()
                    .chain(self.waiting.iter())
                    .any(|c| now >= c.deadline)
            {
                return Err("request deadline exceeded; generation retired".into());
            }
            self.start_calls()?;
            self.flush()?;
            let (control, stdout, stderr, exited) = self.events()?;
            if control {
                self.drain_control();
            }
            if self.connection.stop.load(Ordering::Acquire) {
                return Ok(());
            }
            if stderr {
                self.read_stderr()?;
            }
            if stdout {
                self.read_stdout()?;
            }
            if exited {
                return Err("process exited; generation disconnected".into());
            }
        }
    }
    fn controls(&mut self) {
        self.waiting.extend(
            self.incoming
                .as_ref()
                .expect("live connection receiver")
                .try_iter()
                .take(16),
        );
    }
    fn start_calls(&mut self) -> Result<(), String> {
        if !self.handshake.accepted() {
            return Ok(());
        }
        while let Some(call) = self.waiting.pop_front() {
            let handler = self
                .handshake
                .handlers()
                .iter()
                .find(|h| h.command == call.command);
            let Some(handler) = handler else {
                let notice = format!(
                    "Plugin command unavailable: {} (no admitted handler)",
                    call.command
                );
                drop(call);
                self.connection.notice(notice);
                continue;
            };
            let params = CommandInvokeParams {
                command: call.command.clone(),
                handler: handler.handler.clone(),
                arguments: serde_json::json!({}),
                context: call.context.clone(),
            };
            params
                .validate(&Limits::default())
                .map_err(|e| e.to_string())?;
            self.queue(Message::Request(Request {
                jsonrpc: Version::V2,
                id: call.id.clone(),
                method: CommandInvoke::NAME.into(),
                params: serde_json::to_value(params).map_err(|e| e.to_string())?,
            }))?;
            self.pending.insert(call.id.clone(), call);
        }
        Ok(())
    }
    fn queue(&mut self, message: Message) -> Result<(), String> {
        let limits = Limits::default();
        let bytes =
            transport::encode(&message, limits.frame_bytes as usize).map_err(|e| e.to_string())?;
        if bytes.len() > (limits.outbound_per_package as usize).saturating_sub(self.bytes)
            || !reserve(
                &self.budget.bytes,
                bytes.len(),
                limits.outbound_total as usize,
            )
        {
            return Err("outbound queue limit exceeded".into());
        }
        self.bytes += bytes.len();
        self.output.push_back(Frame {
            bytes,
            offset: 0,
            budget: self.budget.clone(),
        });
        Ok(())
    }
    fn flush(&mut self) -> Result<(), String> {
        let mut budget = 65536;
        while budget > 0 {
            let Some(frame) = self.output.front_mut() else {
                break;
            };
            let bytes = &frame.bytes[frame.offset..];
            let count = match self
                .process
                .child
                .as_mut()
                .expect("owned child")
                .stdin
                .as_mut()
                .expect("owned stdin")
                .write(&bytes[..bytes.len().min(budget)])
            {
                Ok(0) => return Err("stdin disconnected".into()),
                Ok(count) => count,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("stdin: {e}")),
            };
            frame.offset += count;
            self.bytes -= count;
            budget -= count;
            self.budget.bytes.fetch_sub(count, Ordering::AcqRel);
            if frame.offset == frame.bytes.len() {
                self.output.pop_front();
            }
        }
        Ok(())
    }
    fn events(&self) -> Result<(bool, bool, bool, bool), String> {
        let mut fds = vec![
            PollFd::new(&self.connection.control, PollFlags::IN),
            PollFd::new(
                self.process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stdout
                    .as_ref()
                    .expect("owned stdout"),
                PollFlags::IN,
            ),
            PollFd::new(
                self.process.exit.as_ref().expect("owned pidfd"),
                PollFlags::IN,
            ),
        ];
        let stderr = if self.stderr_open {
            fds.push(PollFd::new(
                self.process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stderr
                    .as_ref()
                    .expect("owned stderr"),
                PollFlags::IN,
            ));
            Some(fds.len() - 1)
        } else {
            None
        };
        if !self.output.is_empty() {
            fds.push(PollFd::new(
                self.process
                    .child
                    .as_ref()
                    .expect("owned child")
                    .stdin
                    .as_ref()
                    .expect("owned stdin"),
                PollFlags::OUT,
            ));
        }
        let deadline = self
            .pending
            .values()
            .chain(self.waiting.iter())
            .map(|c| c.deadline)
            .chain((!self.handshake.accepted()).then_some(self.startup))
            .min();
        let timeout = deadline.map(|d| {
            Timespec::try_from(d.saturating_duration_since(Instant::now()))
                .expect("finite deadline")
        });
        match poll(&mut fds, timeout.as_ref()) {
            Ok(_) => Ok((
                !fds[0].revents().is_empty(),
                !fds[1].revents().is_empty(),
                stderr.is_some_and(|i| !fds[i].revents().is_empty()),
                !fds[2].revents().is_empty(),
            )),
            Err(rustix::io::Errno::INTR) => Ok((false, false, false, false)),
            Err(error) => Err(format!("pipe readiness: {error}")),
        }
    }
    fn drain_control(&self) {
        let mut bytes = [0; 8];
        match rustix::io::read(&self.connection.control, &mut bytes) {
            Ok(_) | Err(rustix::io::Errno::AGAIN) | Err(rustix::io::Errno::INTR) => {}
            Err(e) => panic!("plugin control descriptor: {e}"),
        }
    }
    fn read_stderr(&mut self) -> Result<(), String> {
        let mut bytes = [0; 4096];
        for _ in 0..16 {
            match self
                .process
                .child
                .as_mut()
                .expect("owned child")
                .stderr
                .as_mut()
                .expect("owned stderr")
                .read(&mut bytes)
            {
                Ok(0) => {
                    self.stderr_open = false;
                    break;
                }
                Ok(n) => {
                    let mut shared = self
                        .connection
                        .shared
                        .lock()
                        .expect("plugin supervisor owner");
                    let discard = (shared.stderr.len() + n).saturating_sub(128 * 1024);
                    shared.stderr.drain(..discard);
                    shared.stderr.extend(&bytes[..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(format!("stderr: {e}")),
            }
        }
        Ok(())
    }
    fn read_stdout(&mut self) -> Result<(), String> {
        let mut bytes = [0; 8192];
        let count = match self
            .process
            .child
            .as_mut()
            .expect("owned child")
            .stdout
            .as_mut()
            .expect("owned stdout")
            .read(&mut bytes)
        {
            Ok(0) => {
                self.decoder.finish().map_err(|e| e.to_string())?;
                return Err("stdout EOF; generation disconnected".into());
            }
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                return Ok(());
            }
            Err(e) => return Err(format!("stdout: {e}")),
        };
        let mut offset = 0;
        while offset < count {
            if self.connection.stop.load(Ordering::Acquire) {
                return Ok(());
            }
            let (used, message) = self
                .decoder
                .receive(&bytes[offset..count])
                .map_err(|e| e.to_string())?;
            offset += used;
            if let Some(message) = message {
                self.receive(message)?;
            }
        }
        Ok(())
    }
    fn receive(&mut self, message: Message) -> Result<(), String> {
        if let Message::Request(request) = message {
            let error = self
                .handshake
                .admit(&request)
                .expect_err("editor capabilities remain unavailable");
            return self.queue(Message::Failure(Failure {
                jsonrpc: Version::V2,
                id: Some(request.id),
                error,
            }));
        }
        if matches!(message, Message::Notification(_)) {
            return Ok(());
        }
        if !self.handshake.accepted() {
            self.handshake.accept(&message).map_err(|e| e.to_string())?;
            let mut shared = self
                .connection
                .shared
                .lock()
                .expect("plugin supervisor owner");
            shared.state = State::Active;
            drop(shared);
            self.connection.notice(format!(
                "Plugin {}: active",
                self.package.manifest.plugin.id
            ));
            return Ok(());
        }
        let (id, result) = match message {
            Message::Success(success) => {
                let result: CommandsInvokeResult =
                    serde_json::from_value(success.result).map_err(|_| "invalid command result")?;
                result
                    .validate(&Limits::default())
                    .map_err(|e| e.to_string())?;
                (Some(success.id), Ok(()))
            }
            Message::Failure(failure) => (failure.id, Err(failure.error.to_string())),
            _ => return Ok(()),
        };
        let Some(call) = id.and_then(|id| self.pending.remove(&id)) else {
            return Ok(());
        };
        if self.connection.stop.load(Ordering::Acquire) {
            return Ok(());
        }
        if Instant::now() >= call.deadline {
            return Err("request deadline exceeded; generation retired".into());
        }
        let notice = format!(
            "Plugin command {} [{}]: {}",
            call.command,
            call.id,
            result
                .map(|()| "completed".to_owned())
                .unwrap_or_else(|e| inert(&e.as_bytes()[..e.len().min(4096)]))
        );
        drop(call);
        self.connection.notice(notice);
        Ok(())
    }
    fn cancel(&mut self) {
        if !self.handshake.accepted() {
            return;
        }
        self.output.clear();
        self.bytes = 0;
        let messages: Vec<_> = self
            .pending
            .values()
            .map(|call| {
                Message::Notification(Notification {
                    jsonrpc: Version::V2,
                    method: CommandCancel::NAME.into(),
                    params: serde_json::to_value(CommandCancelParams {
                        invocation: call.context.invocation.clone(),
                        generation: call.context.generation.clone(),
                    })
                    .expect("finite cancellation"),
                })
            })
            .collect();
        for message in messages {
            if self.queue(message).is_err() {
                break;
            }
        }
        let _ = self.flush();
    }
}
