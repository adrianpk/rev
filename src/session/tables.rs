pub(super) mod interchange;
use super::{Action, Pending, Session, outline::Request};
use crate::{
    command::{Command, OrgCommand as C},
    input::Focus,
    org::table::{Output, Plan, Snapshot},
};
impl Session {
    pub(in crate::session) fn table_context(&self, command: C) -> bool {
        if command == C::CtrlCCtrlC && self.table_formula_candidate() {
            return true;
        }
        command.table()
            || matches!(
                command,
                C::Cycle
                    | C::GlobalCycle
                    | C::MetaReturn
                    | C::CtrlCCtrlC
                    | C::MetaLeft
                    | C::MetaRight
                    | C::MetaUp
                    | C::MetaDown
                    | C::IndentItemTree
                    | C::OutdentItemTree
            ) && self.table_candidate()
    }
    fn table_formula_candidate(&self) -> bool {
        let Ok(buffer) = self.buffer() else {
            return false;
        };
        let Ok(source) = buffer
            .span(0..buffer.len_chars())
            .and_then(|span| buffer.slice(&span))
        else {
            return false;
        };
        let row = source.char_to_line(self.view().point.position.0);
        let line: String = source.line(row).chars().take(4097).collect();
        line.trim_start()
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("#+TBLFM:"))
    }
    pub(in crate::session) fn table_candidate(&self) -> bool {
        let Ok(buffer) = self.buffer() else {
            return false;
        };
        let Ok(source) = buffer
            .span(0..buffer.len_chars())
            .and_then(|span| buffer.slice(&span))
        else {
            return false;
        };
        let row = source.char_to_line(self.view().point.position.0);
        source
            .line(row)
            .chars()
            .take(4097)
            .find(|c| !matches!(c, ' ' | '\t'))
            .is_some_and(|c| c == '|')
    }
    pub(in crate::session) fn table_return(&mut self, command: C) -> Option<Action> {
        (command == C::Return && !self.table_candidate())
            .then(|| self.edit_command(Command::Newline))
    }
    pub(in crate::session) fn start_table(&mut self, request: Request) -> Result<(), String> {
        if matches!(request.command, C::TableImport | C::TableExport) {
            return self.table_file_prompt(request);
        }
        self.run_table(request, None)
    }
    fn run_table(&mut self, request: Request, argument: Option<String>) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if request.command == C::TableCreate
            && buffer
                .region(request.origin.point)
                .map_err(|e| e.to_string())?
                .is_some()
        {
            return Err("Create an Org table without an active region".into());
        }
        let snapshot = Snapshot::new(
            buffer,
            request.origin.point,
            &self.buffers.snapshot().settings().org,
        )?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_table(snapshot, request.command, argument, seconds)?;
        self.table_request = Some(request);
        self.message("Checking Org table; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_table(&mut self, result: Result<Output, String>) {
        let Some(request) = self.table_request.take() else {
            return;
        };
        if let Ok(Output::Exported(outcome)) = result {
            self.table_export_result(outcome);
            return;
        }
        if !self.org_source_current(&request)
            || self.input.focus() != Focus::Editing
            || self.pending.is_some()
        {
            self.error("Table source or view changed; source unchanged");
            return;
        }
        if let Err(error) = result.and_then(|o| self.table_output(o, request)) {
            self.error(&error);
        }
    }
    fn table_output(&mut self, output: Output, request: Request) -> Result<(), String> {
        match output {
            Output::Prompt => {
                let prompt = if request.command == C::TableSort {
                    "Sort type (a/A alphabetical, n/N numeric): "
                } else {
                    "Table size (columns x rows, default 5x2): "
                };
                self.input.argument(prompt, "").map_err(|e| e.to_string())?;
                self.pending = Some(Pending::Table(Box::new(request)));
                self.message("Enter dimensions; C-g cancels");
                Ok(())
            }
            Output::Plan(plan) => self.publish_table(plan),
            Output::Fallback => self.table_fallback(request),
            Output::Imported(plan, source) => {
                self.table_import_current(&source)?;
                self.publish_table(plan)
            }
            Output::File(text, target) => self.table_export_review(text, target, request),
            Output::Exported(_) => unreachable!("file effects retain their actual outcome"),
        }
    }
    fn table_fallback(&mut self, request: Request) -> Result<(), String> {
        if matches!(request.command, C::Return | C::TableNextRow) {
            self.edit_command(Command::Newline);
            return Ok(());
        }
        let model = self
            .buffer()
            .map_err(|e| e.to_string())?
            .org_outline()
            .cloned()
            .ok_or("Table context observation is no longer current")?;
        self.resume_document_context(model, request);
        Ok(())
    }
    fn publish_table(&mut self, plan: Plan) -> Result<(), String> {
        if plan.edits.is_empty() {
            if let Some(position) = plan.cursor {
                let buffer = self.buffer().map_err(|e| e.to_string())?;
                let point = crate::buffer::edit::Point {
                    stamp: buffer.stamp(),
                    position,
                };
                self.place_point(point).map_err(|e| e.to_string())?;
            }
        } else {
            let buffer = self
                .buffers
                .get_mut(self.windows.selected().point.stamp.buffer)
                .map_err(|e| e.to_string())?;
            let points = self.windows.points(buffer).map_err(|e| e.to_string())?;
            let cursor = plan.cursor.map(|position| crate::buffer::edit::Cursor {
                index: points.initiator,
                position,
            });
            let mut prepared = buffer
                .prepare_retained(&plan.edits, &points.values, cursor, &plan.retained)
                .map_err(|e| e.to_string())?;
            prepared.set_origin(self.windows.selected_id());
            self.windows
                .publish(buffer, prepared, points)
                .map_err(|e| e.to_string())?;
            buffer.deactivate_region();
            self.windows.selected_mut().reset_navigation();
        }
        self.message("Org table");
        Ok(())
    }
    pub(in crate::session) fn table_argument(&mut self, request: Request, draft: &str) -> Action {
        if !self.org_source_current(&request) {
            return self.error("Table source or view changed; source unchanged");
        }
        if draft.len() > 32 {
            return self.error("Table dimensions exceed 32 bytes");
        }
        match self.run_table(request, Some(draft.into())) {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn cancel_table(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit {
            return None;
        }
        let prompt = matches!(
            self.pending,
            Some(Pending::Table(_) | Pending::TableFile(_))
        );
        if self.org.table_effect {
            self.org.cancel_table();
            return Some(
                self.message("Table export cancellation requested; awaiting file outcome"),
            );
        }
        if self.table_request.take().is_none() && !prompt {
            return None;
        }
        self.org.cancel_table();
        self.pending = None;
        self.input.cancel();
        Some(self.message("Org table cancelled; source unchanged"))
    }
}
