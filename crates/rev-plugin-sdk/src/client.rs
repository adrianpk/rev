//! Bounded plugin-side correlation, handshake and byte queues without blocking I/O.

use crate::{
    identity::*,
    limits::Limits,
    methods::*,
    rpc::*,
    transport::{self, Decoder, WireError},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, VecDeque},
    fmt,
    time::Duration,
};

#[derive(Debug)]
pub enum ClientError {
    Wire(WireError),
    Protocol(RpcError),
}
impl From<WireError> for ClientError {
    fn from(error: WireError) -> Self {
        Self::Wire(error)
    }
}
impl From<RpcError> for ClientError {
    fn from(error: RpcError) -> Self {
        Self::Protocol(error)
    }
}
impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wire(error) => error.fmt(f),
            Self::Protocol(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ClientError {}
fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}

#[derive(Debug)]
pub enum Event {
    Initialized(InitializeParams),
    Request(Callback),
    Notification(Notification),
    Response {
        id: OpaqueId,
        method: &'static str,
        result: Result<Value, RpcError>,
    },
    /// Canceled, expired, duplicate or foreign-generation replies cannot resolve a call.
    StaleResponse(OpaqueId),
}
/// A callback token retains connection and admission identity. Reusing a wire
/// request ID cannot make an earlier canceled callback current again.
#[derive(Debug)]
pub struct Callback {
    request: Request,
    session: OpaqueId,
    generation: Generation,
    serial: u64,
}
impl Callback {
    pub fn request(&self) -> &Request {
        &self.request
    }
}
type CheckResult = fn(Value, &Limits) -> Result<Value, RpcError>;
struct Pending {
    method: &'static str,
    deadline: Duration,
    check: CheckResult,
}
struct Incoming {
    method: String,
    invocation: Option<OpaqueId>,
    serial: u64,
}
struct Output {
    bytes: Vec<u8>,
    offset: usize,
}

/// Own one client per process generation. Feed bytes only when its pipe is ready,
/// consume output only after a successful write, and call `expire` on deadlines.
/// One queue and the combined incoming/outgoing pending set obey package limits.
pub struct Client {
    package: Package,
    handlers: List<Handler, 64>,
    offer: Option<InitializeParams>,
    limits: Limits,
    decoder: Decoder,
    retired: bool,
    serial: u64,
    callback_serial: u64,
    pending: BTreeMap<OpaqueId, Pending>,
    incoming: BTreeMap<OpaqueId, Incoming>,
    output: VecDeque<Output>,
    output_bytes: usize,
}
impl Client {
    /// Create a client with the finite default framing profile.
    ///
    /// # Errors
    /// Returns a framing error if the built-in ceiling cannot initialize a decoder.
    pub fn new(package: Package, handlers: List<Handler, 64>) -> Result<Self, ClientError> {
        let limits = Limits::default();
        Ok(Self {
            package,
            handlers,
            offer: None,
            decoder: Decoder::new(limits.frame_bytes as usize)?,
            limits,
            retired: false,
            serial: 0,
            callback_serial: 0,
            pending: BTreeMap::new(),
            incoming: BTreeMap::new(),
            output: VecDeque::new(),
            output_bytes: 0,
        })
    }
    pub fn initialized(&self) -> bool {
        !self.retired && self.offer.is_some()
    }
    pub fn pending_len(&self) -> usize {
        self.pending.len() + self.incoming.len()
    }
    pub fn queued_bytes(&self) -> usize {
        self.output_bytes
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn next_deadline(&self) -> Option<Duration> {
        self.pending.values().map(|p| p.deadline).min()
    }

    /// Admit a package-local dynamic callback before `commands.register`.
    /// Native registration remains a separate checked RPC operation.
    ///
    /// # Errors
    /// Returns `Disconnected` before initialization/after retirement,
    /// `NamespaceConflict` for a foreign or duplicate handler, or `LimitExceeded`.
    pub fn add_handler(&mut self, handler: Handler) -> Result<(), RpcError> {
        if !self.initialized() {
            return Err(error(ErrorKind::Disconnected));
        }
        if !handler.command.belongs_to(&self.package.id)
            || self
                .handlers
                .iter()
                .any(|h| h.command == handler.command || h.handler == handler.handler)
        {
            return Err(error(ErrorKind::NamespaceConflict));
        }
        if self.handlers.len() >= self.limits.commands_per_package as usize {
            return Err(error(ErrorKind::LimitExceeded));
        }
        let mut handlers = self.handlers.clone().into_vec();
        handlers.push(handler);
        self.handlers = List::new(handlers)?;
        Ok(())
    }

    /// Remove a handler from future callback admission after native unregistration.
    pub fn remove_handler(&mut self, command: &CommandId) {
        self.handlers.retain(|handler| &handler.command != command);
    }

    fn queue(&mut self, message: &Message) -> Result<(), ClientError> {
        let bytes = transport::encode(message, self.limits.frame_bytes as usize)?;
        if bytes.len()
            > (self.limits.outbound_per_package as usize).saturating_sub(self.output_bytes)
        {
            return Err(error(ErrorKind::LimitExceeded).into());
        }
        self.output_bytes += bytes.len();
        self.output.push_back(Output { bytes, offset: 0 });
        Ok(())
    }

    /// Slice to write next. A caller can handle partial writes without losing bytes.
    pub fn output(&self) -> &[u8] {
        self.output
            .front()
            .map_or(&[], |frame| &frame.bytes[frame.offset..])
    }
    /// Acknowledge bytes written from the current output slice.
    ///
    /// # Errors
    /// Returns `InvalidArguments` when `written` exceeds that slice's length.
    pub fn consume_output(&mut self, written: usize) -> Result<(), RpcError> {
        if written > self.output().len() {
            return Err(error(ErrorKind::InvalidArguments));
        }
        if let Some(frame) = self.output.front_mut() {
            frame.offset += written;
            self.output_bytes -= written;
            if frame.offset == frame.bytes.len() {
                self.output.pop_front();
            }
        }
        Ok(())
    }

    /// Start one typed package-to-host operation. `now` is a caller's monotonic
    /// clock, never wall time. No request is admitted before initialization.
    ///
    /// # Errors
    /// Returns a protocol error for disconnected/unsupported operations, invalid
    /// parameters or exhausted identity/deadline/queue limits; framing errors
    /// reject invalid or oversized serialized requests before admission.
    pub fn request<M: Method>(
        &mut self,
        params: M::Params,
        now: Duration,
    ) -> Result<OpaqueId, ClientError> {
        let offer = self
            .offer
            .as_ref()
            .filter(|_| !self.retired)
            .ok_or_else(|| error(ErrorKind::Disconnected))?;
        if M::HOST_TO_PACKAGE || !M::CAPABILITY.is_some_and(|cap| offer.capabilities.contains(&cap))
        {
            return Err(error(ErrorKind::UnsupportedMethod).into());
        }
        if self.pending_len() >= self.limits.pending_per_package as usize {
            return Err(error(ErrorKind::LimitExceeded).into());
        }
        params.validate(&self.limits)?;
        let (serial, id) = self.next_id(&offer.generation)?;
        let deadline = now
            .checked_add(Duration::from_millis(u64::from(self.limits.command_ms)))
            .ok_or_else(|| error(ErrorKind::LimitExceeded))?;
        let request = Request {
            jsonrpc: Version::V2,
            id: id.clone(),
            method: M::NAME.into(),
            params: serde_json::to_value(params).map_err(|_| error(ErrorKind::InvalidArguments))?,
        };
        self.queue(&Message::Request(request))?;
        self.serial = serial;
        self.pending.insert(
            id.clone(),
            Pending {
                method: M::NAME,
                deadline,
                check: check_result::<M>,
            },
        );
        Ok(id)
    }

    fn next_id(&self, generation: &Generation) -> Result<(u64, OpaqueId), RpcError> {
        let serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| error(ErrorKind::LimitExceeded))?;
        let id = OpaqueId::new(format!("{}:{serial}", generation))
            .map_err(|_| error(ErrorKind::LimitExceeded))?;
        Ok((serial, id))
    }

    /// Complete an admitted host callback. Method and ID must match its pending
    /// registration, and enqueueing must succeed before retiring the callback.
    ///
    /// # Errors
    /// Returns `Disconnected` for a retired, canceled, reused or foreign callback,
    /// the concrete result validation error, or a framing/output-limit error.
    pub fn respond<M: Method>(
        &mut self,
        callback: &Callback,
        result: Result<M::Result, RpcError>,
    ) -> Result<(), ClientError> {
        let id = &callback.request.id;
        let current = self
            .incoming
            .get(id)
            .is_some_and(|call| call.method == M::NAME && call.serial == callback.serial);
        let connection = self.offer.as_ref().is_some_and(|offer| {
            offer.session == callback.session && offer.generation == callback.generation
        });
        if self.retired || !current || !connection {
            return Err(error(ErrorKind::Disconnected).into());
        }
        let message = match result {
            Ok(result) => {
                result.validate(&self.limits)?;
                Message::Success(Success {
                    jsonrpc: Version::V2,
                    id: id.clone(),
                    result: serde_json::to_value(result)
                        .map_err(|_| error(ErrorKind::InvalidArguments))?,
                })
            }
            Err(error) => Message::Failure(Failure {
                jsonrpc: Version::V2,
                id: Some(id.clone()),
                error: error.for_request(id),
            }),
        };
        self.queue(&message)?;
        self.incoming.remove(id);
        Ok(())
    }

    /// Decode at most one frame. Returned consumption permits several frames in
    /// one pipe read without an unbounded internal inbound queue.
    ///
    /// # Errors
    /// Returns framing errors for invalid input and protocol errors for invalid
    /// initialization, capabilities, callbacks or results. A failed initial
    /// agreement retires the client; an error reply may itself exceed the queue.
    pub fn receive(&mut self, input: &[u8]) -> Result<(usize, Option<Event>), ClientError> {
        if self.retired {
            return Err(error(ErrorKind::Disconnected).into());
        }
        let (count, message) = match self.decoder.receive(input) {
            Ok(result) => result,
            Err(error) => {
                self.retire();
                return Err(error.into());
            }
        };
        let Some(message) = message else {
            return Ok((count, None));
        };
        let request_id = if let Message::Request(request) = &message {
            Some(request.id.clone())
        } else {
            None
        };
        let event = self.accept(message);
        let reply_allowed = !self.retired;
        if event.is_err() && self.offer.is_none() {
            self.retired = true;
        }
        if let Err(ClientError::Protocol(error)) = &event
            && let Some(id) = request_id.filter(|_| reply_allowed)
        {
            self.queue(&Message::Failure(Failure {
                jsonrpc: Version::V2,
                id: Some(id.clone()),
                error: error.clone().for_request(&id),
            }))?;
        }
        Ok((count, Some(event?)))
    }

    fn accept(&mut self, message: Message) -> Result<Event, ClientError> {
        if self.offer.is_none() {
            return self.initialize(message);
        }
        match message {
            Message::Success(success) => Ok(self.response(success.id, Ok(success.result))),
            Message::Failure(failure) => {
                let id = failure
                    .id
                    .ok_or_else(|| error(ErrorKind::InvalidArguments))?;
                Ok(self.response(id, Err(failure.error)))
            }
            Message::Request(request) => self.callback(request),
            Message::Notification(notification) => self.notification(notification),
        }
    }

    fn initialize(&mut self, message: Message) -> Result<Event, ClientError> {
        let Message::Request(request) = message else {
            return Err(error(ErrorKind::UnsupportedInvocation).into());
        };
        if request.method != Initialize::NAME {
            return Err(error(ErrorKind::UnsupportedInvocation).into());
        }
        let offer: InitializeParams = serde_json::from_value(request.params)
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        let result = offer.acknowledge(&self.package, self.handlers.clone())?;
        self.limits = offer.limits.clone();
        self.decoder = Decoder::new(self.limits.frame_bytes as usize)?;
        self.queue(&Message::Success(Success {
            jsonrpc: Version::V2,
            id: request.id,
            result: serde_json::to_value(result).map_err(|_| error(ErrorKind::InvalidArguments))?,
        }))?;
        self.offer = Some(offer.clone());
        Ok(Event::Initialized(offer))
    }

    fn callback(&mut self, request: Request) -> Result<Event, ClientError> {
        self.host_method(&request.method, &request.params, false)?;
        if self.incoming.contains_key(&request.id) {
            self.retire();
            return Err(error(ErrorKind::Disconnected).into());
        }
        if request.method == CommandCancel::NAME {
            self.cancel_callbacks(&request.params)?;
        }
        if self.pending_len() >= self.limits.pending_per_package as usize {
            return Err(error(ErrorKind::LimitExceeded).into());
        }
        let invocation = if request.method == CommandInvoke::NAME {
            let call: CommandInvokeParams = serde_json::from_value(request.params.clone())
                .map_err(|_| error(ErrorKind::InvalidArguments))?;
            Some(call.context.invocation)
        } else {
            None
        };
        let serial = self
            .callback_serial
            .checked_add(1)
            .ok_or_else(|| error(ErrorKind::LimitExceeded))?;
        self.callback_serial = serial;
        self.incoming.insert(
            request.id.clone(),
            Incoming {
                method: request.method.clone(),
                invocation,
                serial,
            },
        );
        let offer = self
            .offer
            .as_ref()
            .ok_or_else(|| error(ErrorKind::Disconnected))?;
        Ok(Event::Request(Callback {
            request,
            session: offer.session.clone(),
            generation: offer.generation.clone(),
            serial,
        }))
    }

    fn notification(&mut self, notification: Notification) -> Result<Event, ClientError> {
        self.host_method(&notification.method, &notification.params, true)?;
        if notification.method == CommandCancel::NAME {
            self.cancel_callbacks(&notification.params)?;
        }
        Ok(Event::Notification(notification))
    }

    fn cancel_callbacks(&mut self, params: &Value) -> Result<(), ClientError> {
        let cancel: CommandCancelParams = serde_json::from_value(params.clone())
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        let canceled: Vec<_> = self
            .incoming
            .iter()
            .filter(|(_, call)| call.invocation.as_ref() == Some(&cancel.invocation))
            .map(|(id, _)| id.clone())
            .collect();
        // Retire all authority before any fallible output operation.
        for id in &canceled {
            self.incoming.remove(id);
        }
        for id in canceled {
            self.queue(&Message::Failure(Failure {
                jsonrpc: Version::V2,
                id: Some(id.clone()),
                error: error(ErrorKind::Cancelled).for_request(&id),
            }))?;
        }
        Ok(())
    }

    fn host_method(
        &self,
        method: &str,
        params: &Value,
        notification: bool,
    ) -> Result<(), RpcError> {
        let offer = self
            .offer
            .as_ref()
            .ok_or_else(|| error(ErrorKind::Disconnected))?;
        match method {
            BUFFER_EVENT if notification && offer.capabilities.contains(&Capability::Events) => {
                self.buffer_event(params, offer)
            }
            CommandInvoke::NAME
                if !notification && offer.capabilities.contains(&Capability::Commands) =>
            {
                self.invocation(params, offer)
            }
            CommandCancel::NAME if offer.capabilities.contains(&Capability::Commands) => {
                self.cancellation(params, offer)
            }
            _ => Err(error(ErrorKind::UnsupportedMethod)),
        }
    }

    fn buffer_event(&self, params: &Value, offer: &InitializeParams) -> Result<(), RpcError> {
        let event: BufferEvent = serde_json::from_value(params.clone())
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        event.validate(&self.limits)?;
        if event.generation != offer.generation
            || event
                .stamp
                .as_ref()
                .is_some_and(|s| s.buffer.session != offer.session)
        {
            return Err(error(ErrorKind::Disconnected));
        }
        Ok(())
    }

    fn invocation(&self, params: &Value, offer: &InitializeParams) -> Result<(), RpcError> {
        let call: CommandInvokeParams = serde_json::from_value(params.clone())
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        call.validate(&self.limits)?;
        if call.context.generation != offer.generation
            || call
                .context
                .stamp
                .as_ref()
                .is_some_and(|s| s.buffer.session != offer.session)
            || call
                .context
                .pane
                .as_ref()
                .is_some_and(|p| p.session != offer.session)
            || !self
                .handlers
                .iter()
                .any(|h| h.command == call.command && h.handler == call.handler)
        {
            return Err(error(ErrorKind::Disconnected));
        }
        Ok(())
    }

    fn cancellation(&self, params: &Value, offer: &InitializeParams) -> Result<(), RpcError> {
        let call: CommandCancelParams = serde_json::from_value(params.clone())
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        if call.generation != offer.generation {
            return Err(error(ErrorKind::Disconnected));
        }
        Ok(())
    }

    fn response(&mut self, id: OpaqueId, result: Result<Value, RpcError>) -> Event {
        let Some(pending) = self.pending.remove(&id) else {
            return Event::StaleResponse(id);
        };
        Event::Response {
            id,
            method: pending.method,
            result: result.and_then(|value| (pending.check)(value, &self.limits)),
        }
    }

    /// Stop waiting locally. This cannot undo native work already published, and
    /// does not invent a package-to-host cancellation method.
    pub fn cancel(&mut self, id: &OpaqueId) -> bool {
        self.pending.remove(id).is_some()
    }
    /// Retire timed-out correlation slots; late results become `StaleResponse`.
    pub fn expire(&mut self, now: Duration) -> Vec<OpaqueId> {
        let expired: Vec<_> = self
            .pending
            .iter()
            .filter(|(_, p)| p.deadline <= now)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &expired {
            self.pending.remove(id);
        }
        expired
    }
    /// Clear connection-local authority, pending work and byte queues. Native
    /// buffers and package-owned persistent records are outside SDK ownership.
    pub fn retire(&mut self) {
        self.retired = true;
        self.pending.clear();
        self.incoming.clear();
        self.output.clear();
        self.output_bytes = 0;
    }
    /// Retire local authority and queues after observing input EOF.
    ///
    /// # Errors
    /// Returns a wire error for an already retired decoder or incomplete frame.
    pub fn finish(&mut self) -> Result<(), ClientError> {
        let result = self.decoder.finish();
        self.retire();
        result.map_err(Into::into)
    }
}
fn check_result<M: Method>(value: Value, limits: &Limits) -> Result<Value, RpcError> {
    let result: M::Result =
        serde_json::from_value(value.clone()).map_err(|_| error(ErrorKind::InvalidArguments))?;
    result.validate(limits)?;
    Ok(value)
}
