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
    State(crate::plugins::state::Operation),
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
            Self::Commands(v) => serde_json::to_value(v),
            Self::Invoked(v) => serde_json::to_value(v),
            Self::Bound(v) => serde_json::to_value(v),
            Self::Unbound(v) => serde_json::to_value(v),
            Self::Subscribed(v) => serde_json::to_value(v),
            buffer => buffer.buffer_value(),
        }
        .expect("finite typed editor result")
    }
    fn buffer_value(self) -> Result<serde_json::Value, serde_json::Error> {
        match self {
            Self::List(v) => serde_json::to_value(v),
            Self::Descriptor(v) => serde_json::to_value(v),
            Self::Read(v) => serde_json::to_value(v),
            Self::Show(v) => serde_json::to_value(v),
            Self::Applied(v) => serde_json::to_value(v),
            Self::Saved(v) => serde_json::to_value(v),
            Self::Closed(v) => serde_json::to_value(v),
            _ => unreachable!("prepared buffer reply precedes buffer wire encoding"),
        }
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
            budget: Arc::clone(&self._permit.budget),
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
        let connection = Arc::clone(&self.connection);
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
    match request.method.split('.').next() {
        Some("state") => decode_state(request),
        Some("buffers") => decode_buffers(request),
        Some("edits") if request.method == EditsApply::NAME => {
            params(&request.params).map(Operation::Apply)
        }
        Some("commands") => decode_commands(request),
        Some("keymaps") => decode_keymaps(request),
        Some("events") => decode_events(request),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    }
}
fn params<T: serde::de::DeserializeOwned + Validate>(
    value: &serde_json::Value,
) -> Result<T, RpcError> {
    let params: T = serde_json::from_value(value.clone())
        .map_err(|_| RpcError::application(ErrorKind::InvalidArguments))?;
    params.validate(&Limits::default())?;
    Ok(params)
}
fn decode_state(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    use crate::plugins::state::Operation as State;
    let operation = match request.method.as_str() {
        StateList::NAME => params(&request.params).map(State::List),
        StateGet::NAME => params(&request.params).map(State::Get),
        StatePut::NAME => params(&request.params).map(State::Put),
        StateDelete::NAME => params(&request.params).map(State::Delete),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    };
    operation.map(Operation::State)
}
fn decode_buffers(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    match request.method.as_str() {
        BuffersList::NAME => params(&request.params).map(Operation::List),
        BuffersDescribe::NAME => params(&request.params).map(Operation::Describe),
        BuffersRead::NAME => params(&request.params).map(Operation::Read),
        BuffersShow::NAME => params(&request.params).map(Operation::Show),
        _ => decode_files(request),
    }
}
fn decode_files(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    match request.method.as_str() {
        BuffersOpen::NAME => {
            let params: BuffersOpenParams = params(&request.params)?;
            location(params.path.as_str()).map(Operation::Open)
        }
        BuffersCreate::NAME => params(&request.params).map(Operation::Create),
        BuffersSave::NAME => {
            let params: BuffersSaveParams = params(&request.params)?;
            let target = params
                .path
                .as_ref()
                .map(|path| location(path.as_str()))
                .transpose()?;
            Ok(Operation::Save(params, target))
        }
        BuffersClose::NAME => params(&request.params).map(Operation::Close),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    }
}
fn location(path: &str) -> Result<file::Location, RpcError> {
    if path.is_empty() || path.contains('\0') || path.contains("://") {
        return Err(RpcError::application(ErrorKind::InvalidArguments));
    }
    file::resolve(std::path::Path::new(path)).map_err(|error| io_error(error.kind()))
}
fn decode_commands(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    match request.method.as_str() {
        CommandsList::NAME => params(&request.params).map(Operation::Commands),
        CommandsInvoke::NAME => params(&request.params).map(Operation::Invoke),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    }
}
fn decode_keymaps(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    match request.method.as_str() {
        KeymapsBind::NAME => params(&request.params).map(Operation::Bind),
        KeymapsUnbind::NAME => params(&request.params).map(Operation::Unbind),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    }
}
fn decode_events(request: &rev_plugin_sdk::rpc::Request) -> Result<Operation, RpcError> {
    match request.method.as_str() {
        EventsSubscribe::NAME => params(&request.params).map(Operation::Subscribe),
        EventsUnsubscribe::NAME => params(&request.params).map(Operation::Unsubscribe),
        _ => Err(RpcError::application(ErrorKind::UnsupportedMethod)),
    }
}
fn io_error(kind: std::io::ErrorKind) -> RpcError {
    RpcError::application(
        if kind == std::io::ErrorKind::InvalidInput || kind == std::io::ErrorKind::InvalidData {
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
    if !location.is_present() {
        return Ok((None, None));
    }
    let input = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(
            i32::try_from(rustix::fs::OFlags::NONBLOCK.bits())
                .expect("native nonblocking flag fits i32"),
        )
        .open(location.path())
        .map_err(|error| io_error(error.kind()))?;
    let observed = input_observation(&input, location)?;
    let bytes = usize::try_from(observed.length).expect("source length is at most 16 MiB");
    if !reserve(&budget.ingress, bytes, MAX_BYTES) {
        return Err(RpcError::application(ErrorKind::Busy));
    }
    let ingress = Ingress {
        budget: Arc::clone(budget),
        bytes,
    };
    // Include the optional BOM in the reader ceiling; growth after metadata
    // cannot turn the bounded read into silently truncated source text.
    let loaded =
        file::Loaded::read((&input).take(MAX + 4)).map_err(|error| io_error(error.kind()))?;
    check_loaded(&input, location, &observed, &loaded)?;
    Ok((Some(loaded), Some(ingress)))
}

const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX: u64 = 16 * 1024 * 1024;
fn input_observation(
    input: &std::fs::File,
    location: &file::Location,
) -> Result<file::Observation, RpcError> {
    let metadata = input.metadata().map_err(|error| io_error(error.kind()))?;
    if !metadata.is_file() {
        return Err(RpcError::application(ErrorKind::UnsupportedResource));
    }
    if metadata.len() > MAX {
        return Err(RpcError::application(ErrorKind::LimitExceeded));
    }
    let observed =
        file::Observation::from_metadata(metadata).map_err(|error| io_error(error.kind()))?;
    if Some(&observed) != location.observation() {
        return Err(RpcError::application(ErrorKind::StaleRevision));
    }
    Ok(observed)
}
fn check_loaded(
    input: &std::fs::File,
    location: &file::Location,
    observed: &file::Observation,
    loaded: &file::Loaded,
) -> Result<(), RpcError> {
    if loaded.observed_text().len_bytes() > MAX_BYTES {
        return Err(RpcError::application(ErrorKind::LimitExceeded));
    }
    // Refuse a source that changed while it was read; no native registry exists
    // on this thread, so canonical reuse is rechecked at publication as well.
    let after =
        file::Observation::from_metadata(input.metadata().map_err(|error| io_error(error.kind()))?)
            .map_err(|error| io_error(error.kind()))?;
    if after != *observed
        || file::observe(location.path())
            .map_err(|error| io_error(error.kind()))?
            .as_ref()
            != location.observation()
    {
        return Err(RpcError::application(ErrorKind::StaleRevision));
    }
    Ok(())
}
