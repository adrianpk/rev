mod work;
use super::*;
use crate::org::{Outline, agenda::ClockQuery, time::ZonedOperation};
use crate::session::Pending;
pub(in crate::session) use work::{Output, Work};
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
        let result = self.prepare_clock_display(&prompt, draft).and_then(|work| {
            self.org
                .start_report(prompt.request, work, prompt.operation)
        });
        match result {
            Ok(()) => self.message("Org clock report pending; C-g cancels"),
            Err(error) => self.error(&error),
        }
    }
    fn prepare_clock_display(&self, prompt: &Prompt, draft: &str) -> Result<Work, String> {
        let scan = prompt.model.clock_scan(
            self.buffer().map_err(|e| e.to_string())?,
            prompt.request.origin.point.position,
            ClockQuery::parse(draft)?,
            &self.buffers.snapshot().settings().org,
        )?;
        Ok(Work::Display(scan))
    }
    pub(super) fn publish_clock_table(
        &mut self,
        model: &Outline,
        request: Request,
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
        let update =
            model.clock_table_update(buffer, request.origin.point.position, &operation, policy)?;
        self.org
            .start_report(request, Work::Update(update), operation)?;
        self.message("Org clock report pending; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn finish_clock_report(&mut self, result: Result<Output, String>) {
        let Some(request) = self.org.report.take() else {
            return;
        };
        if !self.org_request_current(&request) {
            self.message("Clock report source or view changed; result discarded");
            return;
        }
        if let Err(error) = self.publish_clock_output(result) {
            self.error(&error);
        }
    }
    fn publish_clock_output(&mut self, result: Result<Output, String>) -> Result<(), String> {
        match result? {
            Output::Display(feedback) => {
                self.message(&feedback);
            }
            Output::Update(edit) => {
                if self
                    .buffer()
                    .map_err(|e| e.to_string())?
                    .slice(&edit.source)
                    .map_err(|e| e.to_string())?
                    == edit.text
                {
                    self.message("Org clocktable unchanged");
                    return Ok(());
                }
                self.publish_org_edits(&[edit]).map_err(|e| e.to_string())?;
                self.message("Org clocktable updated");
            }
        }
        Ok(())
    }
}
