use super::*;
use crate::org::{
    agenda::{CalendarQuery, SourceScan, SourceSelection},
    columns,
};
impl Session {
    pub(in crate::session) fn start_columns(
        &mut self,
        model: &Outline,
        request: Request,
        update: bool,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if update && buffer.is_readonly() {
            return Err("Column report source is read-only".into());
        }
        let policy = &self.buffers.snapshot().settings().org;
        let plan = model.column_plan(buffer, request.origin.point.position, policy, update)?;
        let operation = self.org_clock.unwrap_or(super::super::timestamps::capture)(&policy.time)?;
        let context = CalendarQuery::new(operation.clone(), policy)?;
        let selection = SourceSelection {
            buffers: vec![buffer.id()],
            paths: Vec::new(),
        };
        let scan = columns::Scan {
            sources: SourceScan::new(&self.buffers, selection, &policy.limits)?,
            clocks: plan.clocks,
            format: plan.format,
            context,
            policy: policy.clone(),
            maxlevel: plan.maxlevel,
            update: plan.update,
        };
        self.org
            .start_report(request, Work::Columns(Box::new(scan)), operation)?;
        self.message("Org column report pending; C-g cancels");
        Ok(())
    }
    pub(super) fn publish_columns(
        &mut self,
        output: columns::Output,
        view: crate::window::ViewId,
    ) -> Result<(), String> {
        match output {
            columns::Output::View(result) => {
                let buffer = self.buffer().map_err(|e| e.to_string())?;
                let selection = SourceSelection {
                    buffers: vec![buffer.id()],
                    paths: Vec::new(),
                };
                self.install_query(*result, selection, view);
                self.query_agenda
                    .as_mut()
                    .expect("installed live view")
                    .columns = true;
                self.message("Org property columns; Enter visits source; g refreshes; q closes");
            }
            columns::Output::Update(edit) => {
                let buffer = self.buffer().map_err(|e| e.to_string())?;
                if buffer.slice(&edit.source).map_err(|e| e.to_string())? == edit.text {
                    self.message("Org columnview unchanged");
                    return Ok(());
                }
                self.publish_org_edits(&[edit]).map_err(|e| e.to_string())?;
                self.message("Org columnview updated");
            }
        }
        Ok(())
    }
}
