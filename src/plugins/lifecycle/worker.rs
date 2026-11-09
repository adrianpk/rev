use super::{Budget, Call, Connection, Permit, State, editor, inert, process::Process, reserve};
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
    event: Option<super::events::Delivery>,
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
    replies: Receiver<editor::Response>,
    requests: BTreeMap<OpaqueId, Instant>,
    state: Option<crate::plugins::state::Store>,
}

pub(super) fn run(
    package: declarations::Package,
    session: OpaqueId,
    generation: Generation,
    connection: Arc<Connection>,
    budget: Arc<Budget>,
    incoming: Receiver<Call>,
    replies: Receiver<editor::Response>,
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
            connection.state_root.is_some(),
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
    let state = connection
        .state_root
        .as_ref()
        .map(|root| crate::plugins::state::Store::new(root, &package.manifest.plugin.id));
    let mut worker = Worker {
        state,
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
        replies,
        requests: BTreeMap::new(),
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
        shared.requests.clear();
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
            self.editor_replies()?;
            self.event_notifications()?;
            let now = Instant::now();
            if self
                .output
                .iter()
                .any(|f| f.event.as_ref().is_some_and(|d| now >= d.deadline))
            {
                return Err(
                    "event invalidation/delivery deadline exceeded; generation retired".into(),
                );
            }
            if self.requests.values().any(|deadline| now >= *deadline)
                || (!self.handshake.accepted() && now >= self.startup)
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
        while let Some(mut call) = self.waiting.pop_front() {
            if !call.live() {
                continue;
            }
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
                call.complete(Err(RpcError::application(ErrorKind::CommandUnavailable)));
                drop(call);
                self.connection.notice(notice);
                continue;
            };
            let params = CommandInvokeParams {
                command: call.command.clone(),
                handler: handler.handler.clone(),
                arguments: call.arguments.clone(),
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
            event: None,
        });
        Ok(())
    }
    fn event_notifications(&mut self) -> Result<(), String> {
        for _ in 0..16 {
            let delivery = self
                .connection
                .events
                .lock()
                .expect("event mailbox")
                .queue
                .pop_front();
            let Some(delivery) = delivery else {
                break;
            };
            if delivery.cancelled.load(Ordering::Acquire) {
                continue;
            }
            self.queue(Message::Notification(Notification {
                jsonrpc: Version::V2,
                method: BUFFER_EVENT.into(),
                params: serde_json::to_value(&delivery.event).expect("typed event"),
            }))?;
            self.output.back_mut().expect("queued event").event = Some(delivery);
        }
        if !self
            .connection
            .events
            .lock()
            .expect("event mailbox")
            .queue
            .is_empty()
        {
            self.connection.signal();
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<(), String> {
        let mut budget = 65536;
        while budget > 0 {
            if let Some(frame) = self.output.pop_front_if(|f| {
                f.offset == 0
                    && f.event
                        .as_ref()
                        .is_some_and(|d| d.cancelled.load(Ordering::Acquire))
            }) {
                self.bytes -= frame.bytes.len();
                continue;
            }
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
                let frame = self.output.pop_front().expect("complete frame");
                if let Some(delivery) = frame.event.as_ref()
                    && !delivery.cancelled.load(Ordering::Acquire)
                    && delivery.event.kind != EventKind::Resnapshot
                {
                    let mut mailbox = self.connection.events.lock().expect("event mailbox");
                    if mailbox.grants.len() == 256 {
                        // An event callback expires when it leaves this finite
                        // delivered-event window, or its subscription retires.
                        mailbox.grants.pop_front();
                    }
                    mailbox
                        .grants
                        .push_back((delivery.event.clone(), delivery.cancelled.clone()));
                }
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
            .chain(self.requests.values().copied())
            .chain(
                self.output
                    .iter()
                    .filter_map(|f| f.event.as_ref().map(|d| d.deadline)),
            )
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
            let admitted = self
                .handshake
                .admit(&request)
                .and_then(|()| editor::decode(&request));
            let mut operation = match admitted {
                Ok(operation) => operation,
                Err(error) => return self.failure(request.id, error),
            };
            let mut cause = None;
            let mut cancelled = None;
            if let editor::Operation::Invoke(params) = &mut operation {
                let parent = self.pending.get(&params.context.invocation);
                let event = self
                    .connection
                    .events
                    .lock()
                    .expect("event mailbox")
                    .cause(&params.context.origin, Some(&params.context.invocation));
                let event = match event {
                    Ok(value) => value,
                    Err(error) => return self.failure(request.id, error),
                };
                let (origin, flag) = if let Some(parent) = parent {
                    if !parent.live() {
                        return self
                            .failure(request.id, RpcError::application(ErrorKind::Cancelled));
                    }
                    if params.context.generation != parent.context.generation
                        || params.context.origin != parent.context.origin
                    {
                        return self.failure(
                            request.id,
                            RpcError::application(ErrorKind::UnsupportedInvocation),
                        );
                    }
                    (parent.context.origin.clone(), parent.cancelled.clone())
                } else if let Some((origin, flag)) = event {
                    if params.context.generation != self.handshake.offer().generation
                        || params.context.origin != origin
                    {
                        return self.failure(
                            request.id,
                            RpcError::application(ErrorKind::UnsupportedInvocation),
                        );
                    }
                    (origin, flag)
                } else {
                    return self.failure(
                        request.id,
                        RpcError::application(ErrorKind::UnsupportedInvocation),
                    );
                };
                if origin.depth >= 8 {
                    return self
                        .failure(request.id, RpcError::application(ErrorKind::LimitExceeded));
                }
                params.context.origin.depth = origin.depth + 1;
                params.context.origin.package = Some(self.package.manifest.plugin.id.clone());
                cause = Some(origin);
                cancelled = Some(flag);
            } else if let editor::Operation::Apply(params) = &operation {
                if let Some(parent) = self.pending.values().find(|call| {
                    call.context.origin.operation == params.origin.operation
                        && call.context.origin.depth == params.origin.depth
                }) {
                    if !parent.live() {
                        return self
                            .failure(request.id, RpcError::application(ErrorKind::Cancelled));
                    }
                    cause = Some(parent.context.origin.clone());
                    cancelled = Some(parent.cancelled.clone());
                } else {
                    let event = self
                        .connection
                        .events
                        .lock()
                        .expect("event mailbox")
                        .cause(&params.origin, None);
                    match event {
                        Ok(Some((origin, flag))) => {
                            cause = Some(origin);
                            cancelled = Some(flag);
                        }
                        Ok(None)
                            if params.origin.depth == 0
                                && !params.origin.operation.as_str().starts_with("source-")
                                && !params.origin.operation.as_str().starts_with("host-") => {}
                        Ok(None) => {
                            return self.failure(
                                request.id,
                                RpcError::application(ErrorKind::UnsupportedInvocation),
                            );
                        }
                        Err(error) => return self.failure(request.id, error),
                    }
                }
            }
            if self.requests.contains_key(&request.id) {
                return Err("duplicate live editor request ID".into());
            }
            if !reserve(&self.budget.pending, 1, 64) {
                return self.failure(request.id, RpcError::application(ErrorKind::LimitExceeded));
            }
            if !reserve(&self.connection.pending, 1, 16) {
                self.budget.pending.fetch_sub(1, Ordering::AcqRel);
                return self.failure(request.id, RpcError::application(ErrorKind::LimitExceeded));
            }
            let deadline = Instant::now() + Duration::from_secs(60);
            self.requests.insert(request.id.clone(), deadline);
            let mut request = editor::Request {
                id: request.id,
                operation: Some(operation),
                deadline,
                package: self.package.manifest.plugin.id.clone(),
                generation: self.handshake.offer().generation.clone(),
                cause,
                cancelled,
                connection: self.connection.clone(),
                _ingress: None,
                _permit: Permit {
                    budget: self.budget.clone(),
                    package: self.connection.pending.clone(),
                },
            };
            if matches!(&request.operation, Some(editor::Operation::State(_))) {
                let Some(editor::Operation::State(operation)) = request.operation.take() else {
                    unreachable!("typed state operation")
                };
                let result = self
                    .state
                    .as_mut()
                    .expect("state capability has a configured root")
                    .execute(operation, || request.live());
                self.requests.remove(&request.id);
                match result {
                    Ok(result) => self.queue(Message::Success(Success {
                        jsonrpc: Version::V2,
                        id: request.id.clone(),
                        result,
                    }))?,
                    Err(error) => self.failure(request.id.clone(), error)?,
                }
            } else {
                self.submit(request);
            }
            return Ok(());
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
            shared.handlers = self
                .handshake
                .handlers()
                .iter()
                .map(|h| h.command.clone())
                .collect();
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
                (Some(success.id), Ok(result))
            }
            Message::Failure(failure) => (failure.id, Err(failure.error)),
            _ => return Ok(()),
        };
        let Some(mut call) = id.and_then(|id| self.pending.remove(&id)) else {
            return Ok(());
        };
        if self.connection.stop.load(Ordering::Acquire) || !call.live() {
            return Ok(());
        }
        if Instant::now() >= call.deadline {
            return Err("request deadline exceeded; generation retired".into());
        }
        let notice = format!(
            "Plugin command {} [{}]: {}",
            call.command,
            call.id,
            match &result {
                Ok(_) => "completed".to_owned(),
                Err(e) => inert(e.to_string().as_bytes()),
            }
        );
        call.complete(result);
        drop(call);
        self.connection.notice(notice);
        Ok(())
    }
    fn failure(&mut self, id: OpaqueId, error: RpcError) -> Result<(), String> {
        let error = error.for_request(&id);
        self.queue(Message::Failure(Failure {
            jsonrpc: Version::V2,
            id: Some(id),
            error,
        }))
    }
    fn submit(&self, request: editor::Request) {
        self.connection
            .shared
            .lock()
            .expect("plugin supervisor owner")
            .requests
            .push_back(request);
        self.connection.notify();
    }
    fn editor_replies(&mut self) -> Result<(), String> {
        for _ in 0..16 {
            let Ok(mut response) = self.replies.try_recv() else {
                break;
            };
            if !response.request.live() {
                self.requests.remove(&response.request.id);
                self.failure(
                    response.request.id.clone(),
                    RpcError::application(ErrorKind::Cancelled),
                )?;
                continue;
            }
            if let Ok(editor::Reply::Load(location)) = response.result {
                match editor::load(&location, &self.budget) {
                    Ok((loaded, ingress)) => {
                        response.request._ingress = ingress;
                        response.request.operation =
                            Some(editor::Operation::Install(location, loaded));
                        self.submit(response.request);
                        continue;
                    }
                    Err(error) => response.result = Err(error),
                }
            }
            if let Ok(editor::Reply::Stage(save, ingress)) = response.result {
                response.request._ingress = Some(ingress);
                response.result = match crate::file::stage(&save, &mut crate::file::Files) {
                    Ok(staged) => {
                        response.request.operation = Some(editor::Operation::Publish(save, staged));
                        self.submit(response.request);
                        continue;
                    }
                    Err(_) => Err(RpcError::application(ErrorKind::StateConflict)),
                };
            }
            self.requests.remove(&response.request.id);
            let result = match response.result {
                Ok(reply) => self.queue(Message::Success(Success {
                    jsonrpc: Version::V2,
                    id: response.request.id.clone(),
                    result: reply.value(),
                })),
                Err(error) => self.failure(response.request.id.clone(), error),
            };
            // The response retires its request permit before notifying UI state.
            drop(response.request);
            result?;
        }
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
