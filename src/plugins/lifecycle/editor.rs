//! Typed supervisor/native exchange. A permit follows every request through
//! preparation, publication and its response; retirement revokes queued work.
use super::{Budget, Connection, Permit, reserve};
use crate::file;
use rev_plugin_sdk::{identity::*, limits::Limits, methods::*, rpc::*};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Instant,
};

pub(crate) struct Ingress {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Drop for Ingress {
    fn drop(&mut self) {
        self.budget.ingress.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
pub(crate) enum Operation {
    List(BuffersListParams),
    Open(file::Location),
    Install(file::Location, Option<file::Loaded>),
    Create(BuffersCreateParams),
    Describe(BufferParams),
    Read(BuffersReadParams),
    Show(BuffersShowParams),
    Apply(EditsApplyParams),
    Save(BuffersSaveParams, Option<file::Location>),
    Publish(crate::buffer::Save, file::Staged),
    Close(BuffersCloseParams),
    Commands(CommandsListParams),
    Invoke(CommandsInvokeParams),
    Bind(KeymapsBindParams),
    Unbind(KeymapsUnbindParams),
    Subscribe(EventsSubscribeParams),
    Unsubscribe(EventsUnsubscribeParams),
}
pub(crate) enum Reply {
    List(BuffersListResult),
    Descriptor(BufferDescriptor),
    Read(BuffersReadResult),
    Show(BuffersShowResult),
    Applied(EditsApplyResult),
    Load(file::Location),
    Stage(crate::buffer::Save, Ingress),
    Saved(BuffersSaveResult),
    Closed(BuffersCloseResult),
    Commands(CommandsListResult),
    Invoked(CommandsInvokeResult),
    Bound(KeymapsBindResult),
    Unbound(Empty),
    Subscribed(EventsSubscribeResult),
}
impl Reply {
    pub(super) fn value(self) -> serde_json::Value {
        match self {
            Self::List(v) => serde_json::to_value(v),
            Self::Descriptor(v) => serde_json::to_value(v),
            Self::Read(v) => serde_json::to_value(v),
            Self::Show(v) => serde_json::to_value(v),
            Self::Applied(v) => serde_json::to_value(v),
            Self::Saved(v) => serde_json::to_value(v),
            Self::Closed(v) => serde_json::to_value(v),
            Self::Commands(v) => serde_json::to_value(v),
            Self::Invoked(v) => serde_json::to_value(v),
            Self::Bound(v) => serde_json::to_value(v),
            Self::Unbound(v) => serde_json::to_value(v),
            Self::Subscribed(v) => serde_json::to_value(v),
            Self::Stage(_, _) | Self::Load(_) => {
                unreachable!("file preparation precedes wire encoding")
            }
        }
        .expect("finite typed editor result")
    }
}
pub(crate) struct Request {
    pub package: PackageId,
    pub generation: Generation,
    pub cause: Option<Origin>,
    pub(super) cancelled: Option<Arc<std::sync::atomic::AtomicBool>>,
    pub operation: Option<Operation>,
    pub(super) id: OpaqueId,
    pub(super) deadline: Instant,
    pub(super) connection: Arc<Connection>,
    pub(super) _permit: Permit,
    pub(super) _ingress: Option<Ingress>,
}
pub(super) struct Response {
    pub request: Request,
    pub result: Result<Reply, RpcError>,
}
impl Request {
    pub fn retain(&self, bytes: usize) -> Result<Ingress, RpcError> {
        if !reserve(&self._permit.budget.ingress, bytes, 16 * 1024 * 1024) {
            return Err(RpcError::application(ErrorKind::Busy));
        }
        Ok(Ingress {
            budget: self._permit.budget.clone(),
            bytes,
        })
    }
    pub fn live(&self) -> bool {
        !self.connection.stop.load(Ordering::Acquire)
            && self
                .cancelled
                .as_ref()
                .is_none_or(|flag| !flag.load(Ordering::Acquire))
            && Instant::now() < self.deadline
    }
    pub fn complete(self, result: Result<Reply, RpcError>) {
        let connection = self.connection.clone();
        // Combined permits cap requests and responses at sixteen. This response
        // owns one of them, so its bounded queue cannot be full.
        if !connection.stop.load(Ordering::Acquire) && Instant::now() < self.deadline {
            let result = if self.live() {
                result
            } else {
                Err(RpcError::application(ErrorKind::Cancelled))
            };
            match connection.responses.try_send(Response {
                request: self,
                result,
            }) {
                Ok(()) => connection.signal(),
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {}
                Err(std::sync::mpsc::TrySendError::Full(_)) => {
                    panic!("reserved editor response slot")
                }
            }
        }
    }
}
pub(super) fn decode(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    fn params<T: serde::de::DeserializeOwned + Validate>(
        value: &serde_json::Value,
    ) -> Result<T, RpcError> {
        let params: T = serde_json::from_value(value.clone())
            .map_err(|_| RpcError::application(ErrorKind::InvalidArguments))?;
        params.validate(&Limits::default())?;
        Ok(params)
    }
    Ok(match request.method.as_str() {
        BuffersList::NAME => Operation::List(params(&request.params)?),
        BuffersOpen::NAME => {
            let params: BuffersOpenParams = params(&request.params)?;
            if params.path.as_str().is_empty()
                || params.path.as_str().contains('\0')
                || params.path.as_str().contains("://")
            {
                return Err(RpcError::application(ErrorKind::InvalidArguments));
            }
            Operation::Open(
                file::resolve(std::path::Path::new(params.path.as_str())).map_err(io_error)?,
            )
        }
        BuffersCreate::NAME => Operation::Create(params(&request.params)?),
        BuffersDescribe::NAME => Operation::Describe(params(&request.params)?),
        BuffersRead::NAME => Operation::Read(params(&request.params)?),
        BuffersShow::NAME => Operation::Show(params(&request.params)?),
        EditsApply::NAME => Operation::Apply(params(&request.params)?),
        BuffersSave::NAME => {
            let params: BuffersSaveParams = params(&request.params)?;
            let target = params
                .path
                .as_ref()
                .map(|path| {
                    if path.as_str().is_empty()
                        || path.as_str().contains('\0')
                        || path.as_str().contains("://")
                    {
                        return Err(RpcError::application(ErrorKind::InvalidArguments));
                    }
                    file::resolve(std::path::Path::new(path.as_str())).map_err(io_error)
                })
                .transpose()?;
            Operation::Save(params, target)
        }
        BuffersClose::NAME => Operation::Close(params(&request.params)?),
        CommandsList::NAME => Operation::Commands(params(&request.params)?),
        CommandsInvoke::NAME => Operation::Invoke(params(&request.params)?),
        KeymapsBind::NAME => Operation::Bind(params(&request.params)?),
        KeymapsUnbind::NAME => Operation::Unbind(params(&request.params)?),
        EventsSubscribe::NAME => Operation::Subscribe(params(&request.params)?),
        EventsUnsubscribe::NAME => Operation::Unsubscribe(params(&request.params)?),
        _ => return Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    })
}
pub(super) fn io_error(error: std::io::Error) -> RpcError {
    RpcError::application(
        if error.kind() == std::io::ErrorKind::InvalidInput
            || error.kind() == std::io::ErrorKind::InvalidData
        {
            ErrorKind::UnsupportedResource
        } else {
            ErrorKind::InvalidArguments
        },
    )
}
/// Use native strict ingress, with a finite plugin admission ceiling. Open a
/// nonblocking descriptor and recheck its kind to reject substituted FIFOs.
pub(super) fn load(
    location: &file::Location,
    budget: &Arc<Budget>,
) -> Result<(Option<file::Loaded>, Option<Ingress>), RpcError> {
    use std::{io::Read, os::unix::fs::OpenOptionsExt};
    const MAX: u64 = 16 * 1024 * 1024;
    if !location.is_present() {
        return Ok((None, None));
    }
    let input = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
        .open(location.path())
        .map_err(io_error)?;
    let metadata = input.metadata().map_err(io_error)?;
    if !metadata.is_file() {
        return Err(RpcError::application(ErrorKind::UnsupportedResource));
    }
    if metadata.len() > MAX {
        return Err(RpcError::application(ErrorKind::LimitExceeded));
    }
    let bytes = metadata.len() as usize;
    let observed = file::Observation::from_metadata(metadata).map_err(io_error)?;
    if Some(&observed) != location.observation() {
        return Err(RpcError::application(ErrorKind::StaleRevision));
    }
    if !reserve(&budget.ingress, bytes, MAX as usize) {
        return Err(RpcError::application(ErrorKind::Busy));
    }
    let ingress = Ingress {
        budget: budget.clone(),
        bytes,
    };
    // Include the optional BOM in the reader ceiling; growth after metadata
    // cannot turn the bounded read into silently truncated source text.
    let loaded = file::Loaded::read((&input).take(MAX + 4)).map_err(io_error)?;
    if loaded.observed_text().len_bytes() as u64 > MAX {
        return Err(RpcError::application(ErrorKind::LimitExceeded));
    }
    // Refuse a source that changed while it was read; no native registry exists
    // on this thread, so canonical reuse is rechecked at publication as well.
    let after =
        file::Observation::from_metadata(input.metadata().map_err(io_error)?).map_err(io_error)?;
    if after != observed
        || file::observe(location.path()).map_err(io_error)?.as_ref() != location.observation()
    {
        return Err(RpcError::application(ErrorKind::StaleRevision));
    }
    Ok((Some(loaded), Some(ingress)))
}
