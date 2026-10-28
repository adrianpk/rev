use super::*;
use crate::org::{Outline, agenda::ClockQuery, time::ZonedOperation};
use crate::session::Pending;

pub(in crate::session) struct Prompt {
    request: Request,
    model: Arc<Outline>,
    operation: ZonedOperation,
}
impl Session {
    pub(super) fn start_clock_report(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let policy = &self.buffers.snapshot().settings().org.time;
        let operation = self.org_clock.unwrap_or(super::timestamps::capture)(policy)?;
        self.input
            .argument("Clock scope [start-date end-date]: ", "")
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::ClockReport(Box::new(Prompt {
            request,
            model,
            operation,
        })));
        self.message("task, subtree or file; optional YYYY-MM-DD bounds, end exclusive; empty subtree; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn clock_report_argument(
        &mut self,
        prompt: Prompt,
        draft: &str,
    ) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.message("Clock report source or view changed; request cancelled");
        }
        match self.clock_report_feedback(&prompt, draft) {
            Ok(feedback) => self.message(&feedback),
            Err(error) => self.error(&error),
        }
    }
    fn clock_report_feedback(&self, prompt: &Prompt, draft: &str) -> Result<String, String> {
        prompt
            .model
            .clock_summary(
                self.buffer().map_err(|e| e.to_string())?,
                prompt.request.origin.point.position,
                ClockQuery::parse(draft)?,
                (
                    &self.buffers.snapshot().settings().org,
                    prompt.operation.zone(),
                ),
            )?
            .feedback()
    }
}

impl Session {
    pub(super) fn publish_clock_table(
        &mut self,
        model: &Outline,
        request: &Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.is_readonly() {
            return Err(crate::buffer::edit::Error::ReadOnly.to_string());
        }
        if request.command == OrgCommand::UpdateDynamicBlock
            && !model.clock_table_at(buffer, request.origin.point.position)
        {
            return Err("No dynamic clocktable at point".into());
        }
        let policy = &self.buffers.snapshot().settings().org;
        let operation = self.org_clock.unwrap_or(super::timestamps::capture)(&policy.time)?;
        let edit =
            model.clock_table_edit(buffer, request.origin.point.position, &operation, policy)?;
        if buffer.slice(&edit.source).map_err(|e| e.to_string())? == edit.text {
            self.message("Org clocktable unchanged");
            return Ok(());
        }
        self.publish_org_edits(&[edit]).map_err(|e| e.to_string())?;
        self.message("Org clocktable updated");
        Ok(())
    }
}
