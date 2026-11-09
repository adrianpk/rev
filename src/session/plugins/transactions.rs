//! Explicit-target transaction adapter. Native atomic preparation/publication
//! owns all source/history/view state; this adapter admits only public handles.
use super::Session;
use crate::{
    buffer::{self, edit},
    plugins::lifecycle::editor::{Reply, Request},
    session::source_batch::{Atomic, AtomicFailure},
};
use rev_plugin_sdk::{
    identity::*,
    methods::*,
    rpc::{ErrorKind, RpcError},
};

fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
fn native(failure: edit::Error) -> RpcError {
    match failure {
        edit::Error::Source(source) => source_error(source),
        edit::Error::ReadOnly => error(ErrorKind::ReadOnly),
        edit::Error::Order | edit::Error::ChangedPositions => error(ErrorKind::EditConflict),
        edit::Error::Capacity => error(ErrorKind::LimitExceeded),
        _ => error(ErrorKind::InvalidArguments),
    }
}
fn source_error(source: buffer::Error) -> RpcError {
    use buffer::Error as Source;
    error(match source {
        Source::Missing(_) | Source::Foreign { .. } => ErrorKind::MissingBuffer,
        Source::Stale { .. } => ErrorKind::StaleRevision,
        Source::Bounds { .. } | Source::Boundary(_) | Source::Line { .. } => {
            ErrorKind::InvalidRange
        }
        Source::Exhausted => ErrorKind::LimitExceeded,
    })
}
// The atomic native owner accepts one source identity and its base replacements.
type NativeTarget = (buffer::BufferId, Vec<edit::Replacement>);
impl Session {
    pub(super) fn plugin_edits(
        &mut self,
        request: &Request,
        params: EditsApplyParams,
    ) -> Result<Reply, RpcError> {
        self.admit_plugin_edits(request, &params)?;
        let targets = self.plugin_edit_targets(&params)?;
        let selected_changed = targets
            .iter()
            .any(|(id, _)| *id == self.windows.selected().point.stamp.buffer);
        let batch = Atomic::prepare(&mut self.buffers, &self.windows, &targets).map_err(native)?;
        // Allocate the complete result before the first publication as well.
        let stamps = batch
            .stamps()
            .iter()
            .map(|stamp| Stamp {
                buffer: self.plugins.host.handle(format!("buffer-{}", stamp.buffer)),
                revision: Revision::new(stamp.revision.to_string()).expect("native revision"),
            })
            .collect();
        let result = EditsApplyResult {
            operation: params.operation,
            stamps: List::new(stamps)?,
        };
        let origin = self.plugin_edit_origin(request, &params.origin)?;
        self.charge_plugin_edits(request, &batch, &targets)?;
        let origin_guard = self.plugins.host.event_origin(origin);
        batch
            .publish(&mut self.buffers, &mut self.windows, || request.live())
            .map_err(|failure| match failure {
                AtomicFailure::Edit(error) => native(error),
                AtomicFailure::Cancelled => error(ErrorKind::Cancelled),
            })?;
        drop(origin_guard);
        self.plugin_macro_published();
        // Native observers already see a complete batch. Retire revision-bound
        // completion/resolve work without manufacturing recorded user commands.
        self.invalidate_completion();
        if selected_changed {
            self.cancel_pointer_capture();
            self.yank = None;
            self.kills.break_chain();
        }
        Ok(Reply::Applied(result))
    }
    fn admit_plugin_edits(
        &self,
        request: &Request,
        params: &EditsApplyParams,
    ) -> Result<(), RpcError> {
        if self.macros.active() && !self.plugin_macro_request(request) {
            return Err(error(ErrorKind::Busy));
        }
        if params
            .origin
            .package
            .as_ref()
            .is_some_and(|p| p != &request.package)
        {
            return Err(error(ErrorKind::NamespaceConflict));
        }
        Ok(())
    }
    fn plugin_edit_targets(
        &self,
        params: &EditsApplyParams,
    ) -> Result<Vec<NativeTarget>, RpcError> {
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(params.targets.len())
            .map_err(|_| error(ErrorKind::LimitExceeded))?;
        for target in params.targets.iter() {
            targets.push(self.plugin_edit_target(target)?);
        }
        Ok(targets)
    }
    fn plugin_edit_target(&self, target: &BufferEdits) -> Result<NativeTarget, RpcError> {
        let id = self.plugin_buffer(&target.stamp.buffer)?;
        let buffer = self
            .buffers
            .get(id)
            .map_err(|_| error(ErrorKind::MissingBuffer))?;
        if target.stamp.revision.as_str() != buffer.stamp().revision.to_string() {
            return Err(error(ErrorKind::StaleRevision));
        }
        if buffer.is_readonly() {
            return Err(error(ErrorKind::ReadOnly));
        }
        let edits = replacements(buffer, &target.replacements)?;
        Ok((id, edits))
    }
    fn plugin_edit_origin(&self, request: &Request, origin: &Origin) -> Result<Origin, RpcError> {
        let mut cause = request
            .cause
            .clone()
            .unwrap_or_else(|| self.plugins.host.event_root());
        if request.cause.is_some() {
            if cause.depth >= 8 {
                return Err(error(ErrorKind::LimitExceeded));
            }
            cause.depth += 1;
        } else if origin.depth != 0 {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        cause.package = Some(request.package.clone());
        Ok(cause)
    }
    fn charge_plugin_edits(
        &mut self,
        request: &Request,
        batch: &Atomic,
        targets: &[NativeTarget],
    ) -> Result<(), RpcError> {
        if let Some(run) = &self.macros.runner {
            batch.check(&self.buffers, &self.windows).map_err(native)?;
            if !request.live() {
                return Err(error(ErrorKind::Cancelled));
            }
            let bytes = targets
                .iter()
                .flat_map(|(_, edits)| edits)
                .map(|edit| edit.text.len())
                .sum();
            // Generated edits keep independent history units, while sharing the
            // automatic run's existing finite text budget across nested calls.
            if run.group.charge(bytes).is_err() {
                self.stop_macro(
                    crate::macros::StopReason::TextBudget,
                    "Keyboard macro generated text limit reached",
                );
                return Err(error(ErrorKind::LimitExceeded));
            }
        }
        Ok(())
    }
}

fn replacements(
    buffer: &buffer::Buffer,
    replacements: &List<Replacement, 256>,
) -> Result<Vec<edit::Replacement>, RpcError> {
    let mut edits = Vec::new();
    edits
        .try_reserve_exact(replacements.len())
        .map_err(|_| error(ErrorKind::LimitExceeded))?;
    for replacement in replacements.iter() {
        let start = usize::try_from(replacement.range.start())
            .map_err(|_| error(ErrorKind::InvalidRange))?;
        let end =
            usize::try_from(replacement.range.end()).map_err(|_| error(ErrorKind::InvalidRange))?;
        let source = buffer
            .span(start..end)
            .map_err(|_| error(ErrorKind::InvalidRange))?;
        edits.push(edit::Replacement {
            source,
            text: replacement.text.as_str().to_owned(),
        });
    }
    Ok(edits)
}
