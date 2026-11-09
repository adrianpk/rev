//! Native handshake admission, separate from process and editor ownership.

use rev_plugin_sdk::{API, identity::*, limits::Limits, methods::*, rpc::*};

/// Protocol-only host offer. No editor capability is implemented by this layer.
pub struct Handshake {
    request: Request,
    offer: InitializeParams,
    accepted: bool,
    commands: Vec<CommandId>,
    handlers: List<Handler, 64>,
}
impl Handshake {
    pub fn new(
        session: OpaqueId,
        generation: Generation,
        package: Package,
    ) -> Result<Self, RpcError> {
        let offer = InitializeParams {
            session,
            generation,
            package,
            api: Label::new(API)?,
            capabilities: List::default(),
            units: Units::UnicodeScalar,
            limits: Limits::default(),
        };
        offer.validate(&offer.limits)?;
        let request = Request {
            jsonrpc: Version::V2,
            id: OpaqueId::new("initialize")
                .map_err(|_| RpcError::application(ErrorKind::InvalidArguments))?,
            method: Initialize::NAME.into(),
            params: serde_json::to_value(&offer)
                .map_err(|_| RpcError::standard(-32603, "cannot encode initialization"))?,
        };
        Ok(Self {
            request,
            offer,
            accepted: false,
            commands: Vec::new(),
            handlers: List::default(),
        })
    }
    /// Advertise the native operations implemented by the connected host.
    pub(crate) fn commands(
        mut self,
        commands: Vec<CommandId>,
        state: bool,
    ) -> Result<Self, RpcError> {
        self.offer.capabilities = List::new(vec![
            Capability::Commands,
            Capability::Buffers,
            Capability::Display,
            Capability::Edits,
            Capability::Invocation,
            Capability::FileLifecycle,
            Capability::Keymaps,
            Capability::Events,
        ])?;
        if state {
            let mut capabilities = self.offer.capabilities.into_vec();
            capabilities.push(Capability::State);
            self.offer.capabilities = List::new(capabilities)?;
        }
        self.request.id = OpaqueId::new(format!("initialize:{}", self.offer.generation))
            .map_err(|_| RpcError::application(ErrorKind::InvalidArguments))?;
        self.request.params = serde_json::to_value(&self.offer)
            .map_err(|_| RpcError::standard(-32603, "cannot encode initialization"))?;
        self.commands = commands;
        Ok(self)
    }
    pub(crate) fn handlers(&self) -> &List<Handler, 64> {
        &self.handlers
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn offer(&self) -> &InitializeParams {
        &self.offer
    }
    pub fn accepted(&self) -> bool {
        self.accepted
    }

    /// Admission is one-shot and bound to the exact offered generation and ID.
    pub fn accept(&mut self, response: &Message) -> Result<(), RpcError> {
        if self.accepted {
            return Err(RpcError::application(ErrorKind::InvalidArguments));
        }
        let result = match response {
            Message::Success(success) if success.id == self.request.id => &success.result,
            Message::Failure(failure) if failure.id.as_ref() == Some(&self.request.id) => {
                return Err(failure.error.clone());
            }
            _ => return Err(RpcError::application(ErrorKind::InvalidArguments)),
        };
        let result: InitializeResult = serde_json::from_value(result.clone())
            .map_err(|_| RpcError::application(ErrorKind::InvalidArguments))?;
        self.offer.accepts(&result)?;
        if result
            .handlers
            .iter()
            .any(|h| !self.commands.contains(&h.command))
        {
            return Err(RpcError::application(ErrorKind::UnsupportedMethod));
        }
        self.handlers = result.handlers;
        self.accepted = true;
        Ok(())
    }

    /// Gate requests against actual host capabilities, not the schema catalog.
    pub fn admit(&self, request: &Request) -> Result<(), RpcError> {
        if !self.accepted {
            return Err(
                RpcError::application(ErrorKind::UnsupportedInvocation).for_request(&request.id)
            );
        }
        let capability = HOST_METHODS
            .iter()
            .find(|(method, _)| *method == request.method)
            .map(|(_, capability)| capability)
            .ok_or_else(|| {
                RpcError::application(ErrorKind::UnsupportedMethod).for_request(&request.id)
            })?;
        if self.offer.capabilities.contains(capability) {
            Ok(())
        } else {
            Err(RpcError::application(ErrorKind::UnsupportedMethod).for_request(&request.id))
        }
    }
}

// This explicit native implementation map is independent of the SDK schema
// catalog: adding a schema never admits a host operation automatically.
const HOST_METHODS: &[(&str, Capability)] = &[
    (BuffersList::NAME, Capability::Buffers),
    (BuffersOpen::NAME, Capability::Buffers),
    (BuffersCreate::NAME, Capability::Buffers),
    (BuffersDescribe::NAME, Capability::Buffers),
    (BuffersRead::NAME, Capability::Buffers),
    (BuffersShow::NAME, Capability::Display),
    (EditsApply::NAME, Capability::Edits),
    (BuffersSave::NAME, Capability::FileLifecycle),
    (BuffersClose::NAME, Capability::FileLifecycle),
    (CommandsList::NAME, Capability::Commands),
    (CommandsInvoke::NAME, Capability::Invocation),
    (KeymapsBind::NAME, Capability::Keymaps),
    (KeymapsUnbind::NAME, Capability::Keymaps),
    (EventsSubscribe::NAME, Capability::Events),
    (EventsUnsubscribe::NAME, Capability::Events),
    (StateList::NAME, Capability::State),
    (StateGet::NAME, Capability::State),
    (StatePut::NAME, Capability::State),
    (StateDelete::NAME, Capability::State),
];
