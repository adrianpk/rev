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
    pub(crate) fn commands(mut self, commands: Vec<CommandId>) -> Result<Self, RpcError> {
        self.offer.capabilities = List::new(vec![
            Capability::Commands,
            Capability::Buffers,
            Capability::Display,
            Capability::Edits,
            Capability::Invocation,
            Capability::FileLifecycle,
            Capability::Keymaps,
        ])?;
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
        let capability = match request.method.as_str() {
            BuffersList::NAME
            | BuffersOpen::NAME
            | BuffersCreate::NAME
            | BuffersDescribe::NAME
            | BuffersRead::NAME => Capability::Buffers,
            BuffersShow::NAME => Capability::Display,
            EditsApply::NAME => Capability::Edits,
            BuffersSave::NAME | BuffersClose::NAME => Capability::FileLifecycle,
            CommandsList::NAME => Capability::Commands,
            CommandsInvoke::NAME => Capability::Invocation,
            KeymapsBind::NAME | KeymapsUnbind::NAME => Capability::Keymaps,
            _ => {
                return Err(
                    RpcError::application(ErrorKind::UnsupportedMethod).for_request(&request.id)
                );
            }
        };
        if self.offer.capabilities.contains(&capability) {
            Ok(())
        } else {
            Err(RpcError::application(ErrorKind::UnsupportedMethod).for_request(&request.id))
        }
    }
}
