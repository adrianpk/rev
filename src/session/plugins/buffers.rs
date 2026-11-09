//! Native text/view operations. Wire decoding, encoding and file reads belong
//! to the supervisor; only this serialized owner can publish buffers or panes.
use super::Session;
use crate::{
    buffer::{Buffer, BufferId},
    plugins::lifecycle::{
        Service,
        editor::{Operation, Reply, Request},
    },
};
use rev_plugin_sdk::{
    identity::*,
    limits::Limits,
    methods::*,
    rpc::{ErrorKind, RpcError},
};

fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
fn native(_: impl std::fmt::Display) -> RpcError {
    error(ErrorKind::UnsupportedResource)
}
fn text(buffer: &Buffer) -> Result<(), RpcError> {
    if buffer.is_transient() || buffer.terminal().is_some() {
        Err(error(ErrorKind::UnsupportedResource))
    } else {
        Ok(())
    }
}
impl Session {
    pub(super) fn serve_plugin_buffers(&mut self, service: &mut Service) {
        while let Some(mut request) = self.plugins.host.request(service) {
            // Retirement and deadline are checked immediately before the native
            // operation. A queued result never reacquires a retired generation.
            if !request.live() {
                continue;
            }
            let operation = request.operation.take().expect("admitted editor operation");
            if let Operation::Invoke(params) = operation {
                self.plugin_invoke(request, params);
                continue;
            }
            let origin = if matches!(
                &operation,
                Operation::Install(_, _) | Operation::Create(_) | Operation::Close(_)
            ) {
                let mut origin = self.plugins.host.event_root();
                origin.package = Some(request.package.clone());
                Some(self.plugins.host.event_origin(origin))
            } else {
                None
            };
            let result = self.plugin_buffer_operation(&request, operation);
            drop(origin);
            request.complete(result);
        }
    }
    pub(super) fn plugin_buffer(&self, handle: &Handle) -> Result<BufferId, RpcError> {
        if !self.plugins.host.owns(handle) {
            return Err(error(ErrorKind::MissingBuffer));
        }
        let buffer = self
            .buffers
            .iter()
            .find(|b| handle.id.as_str() == format!("buffer-{}", b.id()))
            .ok_or_else(|| error(ErrorKind::MissingBuffer))?;
        text(buffer)?;
        Ok(buffer.id())
    }
    pub(super) fn plugin_descriptor(&self, buffer: &Buffer) -> Result<BufferDescriptor, RpcError> {
        text(buffer)?;
        let expected_disk = match buffer.file().and_then(|f| f.observation()) {
            None => DiskObservation::Absent,
            Some(o) => DiskObservation::Observed {
                token: OpaqueId::new(format!(
                    "disk-{:x}.{:x}.{:x}.{:x}.{:x}.{:x}.{:x}.{:x}",
                    o.device,
                    o.inode,
                    o.length,
                    o.modified.0 as u64,
                    o.modified.1 as u32,
                    o.changed.0 as u64,
                    o.changed.1 as u32,
                    o.mode,
                ))
                .expect("finite lossless native disk observation"),
            },
        };
        Ok(BufferDescriptor {
            stamp: Stamp {
                buffer: self.plugins.host.handle(format!("buffer-{}", buffer.id())),
                revision: Revision::new(buffer.stamp().revision.to_string())
                    .expect("native revision"),
            },
            name: Label::new(buffer.name())?,
            path: buffer
                .file()
                .map(|f| {
                    f.path()
                        .to_str()
                        .ok_or_else(|| error(ErrorKind::UnsupportedResource))
                        .and_then(Label::new)
                })
                .transpose()?,
            kind: ResourceKind::Text,
            read_only: buffer.is_readonly(),
            modified: buffer.is_modified(),
            expected_disk,
            owner: buffer.plugin_owner().cloned(),
            scalar_len: buffer.len_chars() as u64,
        })
    }
    fn plugin_buffer_operation(
        &mut self,
        request: &Request,
        operation: Operation,
    ) -> Result<Reply, RpcError> {
        if matches!(
            &operation,
            Operation::Open(_)
                | Operation::Install(_, _)
                | Operation::Create(_)
                | Operation::Apply(_)
                | Operation::Save(_, _)
                | Operation::Publish(_, _)
                | Operation::Close(_)
        ) && (self.context_storage.locked()
            || self.context_prepared.is_some()
            || self.quit_pending())
        {
            return Err(error(ErrorKind::Busy));
        }
        match operation {
            Operation::Subscribe(params) => {
                for handle in params.buffers.iter() {
                    self.plugin_buffer(handle)?;
                }
                self.plugins
                    .host
                    .subscribe(request, params)
                    .map(Reply::Subscribed)
            }
            Operation::Unsubscribe(params) => {
                self.plugins
                    .host
                    .unsubscribe(request, &params.subscription)?;
                Ok(Reply::Unbound(Empty {}))
            }
            Operation::List(params) => {
                if params.kind.is_some_and(|k| k != ResourceKind::Text) {
                    return Err(error(ErrorKind::UnsupportedResource));
                }
                // Continuations carry the host session and last native ID.
                let prefix = format!(
                    "page-{}-",
                    self.plugins.host.handle("cursor".into()).session
                );
                let after = params
                    .page
                    .cursor
                    .as_ref()
                    .map(|c| {
                        c.as_str()
                            .strip_prefix(&prefix)
                            .and_then(|s| s.parse::<u64>().ok())
                            .ok_or_else(|| error(ErrorKind::InvalidArguments))
                    })
                    .transpose()?;
                let mut buffers = Vec::new();
                let mut last = None;
                let mut next = None;
                for buffer in self.buffers.iter() {
                    if buffer.is_transient() || buffer.terminal().is_some() {
                        continue;
                    }
                    let number: u64 = buffer.id().to_string().parse().expect("native identity");
                    if after.is_some_and(|a| number <= a) {
                        continue;
                    }
                    if buffers.len() == usize::from(params.page.limit) {
                        next = Some(
                            OpaqueId::new(format!("{prefix}{}", last.expect("nonempty page")))
                                .expect("finite cursor"),
                        );
                        break;
                    }
                    buffers.push(self.plugin_descriptor(buffer)?);
                    last = Some(number);
                }
                Ok(Reply::List(BuffersListResult {
                    buffers: List::new(buffers)?,
                    next,
                }))
            }
            Operation::Open(location) => {
                if let Some(id) = self.buffers.resolved_buffer(&location) {
                    return Ok(Reply::Descriptor(
                        self.plugin_descriptor(self.buffers.get(id).map_err(native)?)?,
                    ));
                }
                Ok(Reply::Load(location))
            }
            Operation::Install(location, loaded) => {
                let visit = self
                    .buffers
                    .prepare_resolved_visit(location, loaded)
                    .map_err(native)?;
                let descriptor =
                    self.plugin_descriptor(visit.buffer(&self.buffers).map_err(native)?)?;
                if !request.live() {
                    return Err(error(ErrorKind::Cancelled));
                }
                self.buffers.publish_visit(visit);
                Ok(Reply::Descriptor(descriptor))
            }
            Operation::Create(params) => {
                if params.name.as_str().is_empty()
                    || params.name.as_str().len() > 4000
                    || params.name.as_str().chars().any(char::is_control)
                {
                    return Err(error(ErrorKind::InvalidArguments));
                }
                if params.text.as_str().contains('\0') {
                    return Err(error(ErrorKind::UnsupportedResource));
                }
                let buffer = self
                    .buffers
                    .prepare_plugin_create(
                        request.package.clone(),
                        params.name.as_str(),
                        params.text.as_str(),
                        params.read_only,
                    )
                    .map_err(native)?;
                let descriptor = self.plugin_descriptor(&buffer)?;
                if !request.live() {
                    return Err(error(ErrorKind::Cancelled));
                }
                self.buffers.insert(buffer);
                Ok(Reply::Descriptor(descriptor))
            }
            Operation::Describe(params) => {
                let id = self.plugin_buffer(&params.buffer)?;
                Ok(Reply::Descriptor(self.plugin_descriptor(
                    self.buffers.get(id).map_err(native)?,
                )?))
            }
            Operation::Read(params) => {
                let id = self.plugin_buffer(&params.buffer)?;
                let buffer = self.buffers.get(id).map_err(native)?;
                let descriptor = self.plugin_descriptor(buffer)?;
                if params
                    .expected_revision
                    .is_some_and(|r| r != descriptor.stamp.revision)
                {
                    return Err(error(ErrorKind::StaleRevision));
                }
                let start = usize::try_from(params.range.start())
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                let end = usize::try_from(params.range.end())
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                let span = buffer
                    .span(start..end)
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                let slice = buffer.slice(&span).map_err(native)?;
                if slice.len_bytes() > Limits::default().text_bytes as usize {
                    return Err(error(ErrorKind::LimitExceeded));
                }
                Ok(Reply::Read(BuffersReadResult {
                    stamp: descriptor.stamp,
                    range: params.range,
                    text: SourceText::new(slice.to_string())?,
                }))
            }
            Operation::Show(params) => self.plugin_show(request, params),
            Operation::Apply(params) => self.plugin_edits(request, params),
            Operation::Save(params, target) => self.plugin_save(request, params, target),
            Operation::Publish(save, staged) => self.plugin_publish_save(request, save, staged),
            Operation::Close(params) => self.plugin_close(request, params),
            Operation::Commands(params) => self.plugin_commands(params),
            Operation::Bind(params) => self.plugin_bind(request, params),
            Operation::Unbind(params) => self.plugin_unbind(request, params),
            Operation::Invoke(_) => {
                unreachable!("owned invocation handled before borrowed operations")
            }
        }
    }
    fn plugin_show(
        &mut self,
        request: &Request,
        params: BuffersShowParams,
    ) -> Result<Reply, RpcError> {
        let id = self.plugin_buffer(&params.buffer)?;
        let pane = if let Some(handle) = params.pane {
            if !self.plugins.host.owns(&handle) {
                return Err(error(ErrorKind::UnsupportedInvocation));
            }
            self.windows
                .iter()
                .find(|(id, _)| handle.id.as_str() == format!("pane-{id}"))
                .map(|(id, _)| id)
                .ok_or_else(|| error(ErrorKind::UnsupportedInvocation))?
        } else {
            self.windows.selected_id()
        };
        if self.input.focus() != crate::input::Focus::Editing
            || self.pending.is_some()
            || self.tree.focused()
            || self.terminals.focused
            || self.capture_blocks_input()
            || self.capture_selected()
            || self.source_edit_selected()
            || self.windows.retained_view_active()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
            || self.macros.active()
        {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        let mut view = self.windows.get(pane).map_err(native)?.clone();
        if view.point.stamp.buffer != id {
            view.rebind(self.buffers.get(id).map_err(native)?);
        }
        let split = match params.placement {
            Placement::Current => None,
            Placement::SplitHorizontal => Some(crate::window::Split::Below),
            Placement::SplitVertical => Some(crate::window::Split::Right),
        };
        self.stop_typing().map_err(native)?;
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        let pane = self
            .windows
            .show(pane, view, split, self.editing_size())
            .map_err(|_| error(ErrorKind::UnsupportedInvocation))?;
        self.touch_completion_source(id);
        self.sync_input_context();
        Ok(Reply::Show(BuffersShowResult {
            pane: self.plugins.host.handle(format!("pane-{pane}")),
        }))
    }
}
