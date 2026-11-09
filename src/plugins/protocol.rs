//! Native handshake admission, separate from process and editor ownership.

use rev_plugin_sdk::{API, identity::*, limits::Limits, methods::*, rpc::*};

/// Protocol-only host offer. No editor capability is implemented by this layer.
pub struct Handshake {
    request: Request,
    offer: InitializeParams,
    accepted: bool,
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
        })
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
        if !result.handlers.is_empty() {
            return Err(RpcError::application(ErrorKind::UnsupportedMethod));
        }
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
        // The protocol slice deliberately publishes no executable editor method.
        Err(RpcError::application(ErrorKind::UnsupportedMethod).for_request(&request.id))
    }
}
