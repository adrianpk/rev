use super::*;
impl Session {
    pub(in crate::session) fn source_edit_command(&mut self, command: Command) -> Option<Action> {
        if self.source_edit.is_none() {
            return self.source_edit_idle(command);
        }
        match command {
            Command::Org(C::EditSpecial | C::EditSrcExit) => Some(self.finish_source_edit()),
            Command::Org(C::EditSrcAbort) => Some(self.abort_source_edit()),
            Command::KeyboardQuit => None,
            _ if protected(command) => {
                Some(self.message("Source draft retained; return with C-c ' or abort with C-c C-k"))
            }
            _ => None,
        }
    }
    fn source_edit_idle(&mut self, command: Command) -> Option<Action> {
        if matches!(command, Command::Org(C::EditSrcExit | C::EditSrcAbort)) {
            return Some(self.message("No source edit draft is active"));
        }
        if command != Command::KeyboardQuit {
            return None;
        }
        let prompt = matches!(self.pending, Some(Pending::SourceHeader(_)));
        if self.block_request.take().is_none() && !prompt {
            return None;
        }
        self.org.cancel_block();
        self.input.cancel();
        self.pending = None;
        Some(self.message("Source block operation cancelled; source unchanged"))
    }
    fn finish_source_edit(&mut self) -> Action {
        match self.publish_source_edit() {
            Ok(()) => self.leave_source_edit("Source block updated; source remains unsaved"),
            Err(e) => self.error(&e),
        }
    }
    fn publish_source_edit(&mut self) -> Result<(), String> {
        self.stop_typing().map_err(|e| e.to_string())?;
        let state = self.source_edit.as_ref().ok_or("No source draft")?;
        if !self.source_edit_selected() {
            return Err("Select the retained source draft before returning".into());
        }
        let text = state.text(&self.buffers)?;
        let source = self
            .buffers
            .get_mut(state.definition.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let edit = state.definition.replacement(source, &text)?;
        if text == state.definition.draft {
            return Ok(());
        }
        let prepared = super::super::source_batch::PreparedSource::prepare_retained(
            source,
            &self.windows,
            &[edit],
        )?;
        prepared.publish(source, &mut self.windows);
        Ok(())
    }
    fn abort_source_edit(&mut self) -> Action {
        self.leave_source_edit("Source edit aborted; source unchanged")
    }
    fn leave_source_edit(&mut self, message: &str) -> Action {
        let Some(state) = self.source_edit.take() else {
            return self.message("No source draft");
        };
        self.windows.restore_view();
        if let Err(e) = self.discard_capture_buffer(state.buffer) {
            return self.error(&e);
        }
        if let Err(e) = self.windows.select(state.request.view) {
            return self.error(&e.to_string());
        }
        self.input.cancel();
        self.sync_input_context();
        self.message(message)
    }
}
fn protected(command: Command) -> bool {
    matches!(
        command,
        Command::SaveBuffer
            | Command::WriteFile
            | Command::FindFile
            | Command::Quit
            | Command::Buffer(_)
            | Command::Window(_)
            | Command::Split(_)
            | Command::Tree(_)
            | Command::Mode(_)
            | Command::Org(_)
    )
}

impl Draft {
    fn text(&self, buffers: &crate::buffer::Buffers) -> Result<String, String> {
        let draft = buffers.get(self.buffer).map_err(|e| e.to_string())?;
        let span = draft
            .span(0..draft.len_chars())
            .map_err(|e| e.to_string())?;
        draft
            .slice(&span)
            .map(|s| s.to_string())
            .map_err(|e| e.to_string())
    }
}
