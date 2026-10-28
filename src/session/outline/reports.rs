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
