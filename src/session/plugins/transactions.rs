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
    use buffer::Error as Source;
    error(match failure {
        edit::Error::ReadOnly => ErrorKind::ReadOnly,
        edit::Error::Source(Source::Missing(_) | Source::Foreign { .. }) => {
            ErrorKind::MissingBuffer
        }
        edit::Error::Source(Source::Stale { .. }) => ErrorKind::StaleRevision,
        edit::Error::Source(Source::Bounds { .. } | Source::Boundary(_) | Source::Line { .. }) => {
            ErrorKind::InvalidRange
        }
        edit::Error::Order | edit::Error::ChangedPositions => ErrorKind::EditConflict,
        edit::Error::Capacity | edit::Error::Source(Source::Exhausted) => ErrorKind::LimitExceeded,
        _ => ErrorKind::InvalidArguments,
    })
}
impl Session {
    pub(super) fn plugin_edits(
        &mut self,
        request: &Request,
        params: EditsApplyParams,
    ) -> Result<Reply, RpcError> {
        if self.macros.active() {
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
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(params.targets.len())
            .map_err(|_| error(ErrorKind::LimitExceeded))?;
        for target in params.targets.iter() {
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
            let mut edits = Vec::new();
            edits
                .try_reserve_exact(target.replacements.len())
                .map_err(|_| error(ErrorKind::LimitExceeded))?;
            for replacement in target.replacements.iter() {
                let start = usize::try_from(replacement.range.start())
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                let end = usize::try_from(replacement.range.end())
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                let source = buffer
                    .span(start..end)
                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                edits.push(edit::Replacement {
                    source,
                    text: replacement.text.as_str().to_owned(),
                });
            }
            targets.push((id, edits));
        }
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
        batch
            .publish(&mut self.buffers, &mut self.windows, || request.live())
            .map_err(|failure| match failure {
                AtomicFailure::Edit(error) => native(error),
                AtomicFailure::Cancelled => error(ErrorKind::Cancelled),
            })?;
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
}
