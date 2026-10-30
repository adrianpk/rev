//! Special edits retain a transient buffer until checked canonical reinsertion.
mod lifecycle;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    buffer::{Buffer, BufferId},
    command::{Command, OrgCommand as C},
    coordinates::Coordinates,
    org::block::{Definition, Scan},
};
pub(super) struct Draft {
    request: Request,
    definition: Definition,
    buffer: BufferId,
}
pub(super) struct HeaderPrompt {
    request: Request,
    definition: Definition,
}
impl Session {
    pub fn source_edit_draft(&self) -> Option<&Buffer> {
        self.buffers.get(self.source_edit.as_ref()?.buffer).ok()
    }
    pub(super) fn source_edit_selected(&self) -> bool {
        self.source_edit_draft()
            .is_some_and(|b| b.id() == self.view().point.stamp.buffer)
    }
    pub(in crate::session) fn begin_block(&mut self, request: Request) -> Result<(), String> {
        if self.windows.retained_view_active() {
            return Err(
                "Another interaction retains this view; finish it before source editing".into(),
            );
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if !matches!(request.command, C::ViewSrcInfo | C::ExecuteSrc) && buffer.is_readonly() {
            return Err("Source buffer is read-only".into());
        }
        let byte = Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_byte(request.origin.point.position)
            .map_err(|e| e.to_string())?;
        let scan = Scan::new(buffer, byte)?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org.start_block(buffer.stamp(), scan, seconds)?;
        self.block_request = Some(request);
        self.message("Source block observation pending; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_block(&mut self, result: Result<Definition, String>) {
        let Some(request) = self.block_request.take() else {
            return;
        };
        if !self.org_source_current(&request) {
            self.error("Source block or view changed during observation");
            return;
        }
        if let Err(e) = result.and_then(|definition| self.block_result(request, definition)) {
            self.error(&e);
        }
    }
    fn block_result(&mut self, request: Request, definition: Definition) -> Result<(), String> {
        match request.command {
            C::ExecuteSrc => self.prepare_execution(request, definition),
            C::ViewSrcInfo => {
                self.message(&definition.info());
                Ok(())
            }
            C::EditSpecial => self.install_source_edit(request, definition),
            C::InsertHeaderArg => {
                self.input
                    .argument("Source header argument (:key value): ", "")
                    .map_err(|e| e.to_string())?;
                self.pending = Some(Pending::SourceHeader(Box::new(HeaderPrompt {
                    request,
                    definition,
                })));
                self.message("Enter one source header argument; C-g cancels");
                Ok(())
            }
            _ => unreachable!("source observation command"),
        }
    }
    fn install_source_edit(
        &mut self,
        request: Request,
        definition: Definition,
    ) -> Result<(), String> {
        let id = self
            .buffers
            .source_draft(&definition.draft, &definition.language)
            .map_err(|e| e.to_string())?;
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let point = crate::view::View::new(buffer).point;
        if let Err(e) = self.windows.retain_view(buffer, point) {
            self.discard_capture_buffer(id)?;
            return Err(e);
        }
        self.source_edit = Some(Draft {
            request,
            definition,
            buffer: id,
        });
        self.sync_input_context();
        self.message("Source draft: C-c ' or C-c C-c returns; C-c C-k aborts");
        Ok(())
    }
    pub(in crate::session) fn source_header_argument(
        &mut self,
        prompt: HeaderPrompt,
        draft: &str,
    ) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.error("Source header context changed; source unchanged");
        }
        let result = self
            .buffer()
            .map_err(|e| e.to_string())
            .and_then(|b| prompt.definition.header_replacement(b, draft))
            .and_then(|e| self.publish_org_edits(&[e]).map_err(|e| e.to_string()));
        match result {
            Ok(()) => self.message("Source header argument inserted"),
            Err(e) => self.error(&e),
        }
    }
}
