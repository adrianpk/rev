use super::*;
use crate::{
    buffer::{Buffer, BufferId},
    session::source_batch::PreparedSource,
};
struct PreparedRefile {
    visit: crate::buffer::Visit,
    insertion: PreparedSource,
    removal: Option<PreparedSource>,
    source: BufferId,
    target: BufferId,
    copy: bool,
}
impl Session {
    pub(in crate::session) fn publish_refile(&mut self, plan: Plan) -> Result<(), String> {
        let PreparedRefile {
            mut visit,
            insertion,
            removal,
            source,
            target,
            copy,
        } = self.prepare_refile(&plan)?;
        // All histories and live points are admitted. Exclusive session ownership
        // permits no callbacks or fallible work between canonical publications.
        let buffer = visit
            .buffer_mut(&mut self.buffers)
            .expect("admitted destination owner");
        insertion.publish(buffer, &mut self.windows);
        self.buffers.publish_visit(visit);
        if let Some(removal) = removal {
            let buffer = self
                .buffers
                .get_mut(source)
                .expect("admitted refile source");
            removal.publish(buffer, &mut self.windows);
        }
        self.windows.selected_mut().reset_navigation();
        self.record_refile(source, target, copy);
        Ok(())
    }
    fn prepare_refile(&mut self, plan: &Plan) -> Result<PreparedRefile, String> {
        let mut visit = plan.visit(&self.buffers)?;
        let (edits, source, target) = self.refile_edits(plan, &visit)?;
        let removal = self.prepare_refile_removal(plan, source, target, &edits)?;
        let changes = for_buffer(&edits, target);
        let buffer = visit
            .buffer_mut(&mut self.buffers)
            .map_err(|e| e.to_string())?;
        let insertion = PreparedSource::prepare(buffer, &self.windows, &changes)?;
        Ok(PreparedRefile {
            visit,
            insertion,
            removal,
            source,
            target,
            copy: plan.copy,
        })
    }
    fn refile_edits(
        &self,
        plan: &Plan,
        visit: &crate::buffer::Visit,
    ) -> Result<(Vec<crate::buffer::edit::Replacement>, BufferId, BufferId), String> {
        let from = self
            .buffers
            .get(plan.subtree.stamp().buffer)
            .map_err(|e| e.to_string())?;
        let destination = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        verify_file(from, !plan.copy)?;
        verify_file(destination, true)?;
        Ok((plan.edits(from, destination)?, from.id(), destination.id()))
    }
    fn prepare_refile_removal(
        &mut self,
        plan: &Plan,
        source: BufferId,
        target: BufferId,
        edits: &[crate::buffer::edit::Replacement],
    ) -> Result<Option<PreparedSource>, String> {
        if source == target || plan.copy {
            return Ok(None);
        }
        let changes = for_buffer(edits, source);
        let buffer = self.buffers.get_mut(source).map_err(|e| e.to_string())?;
        PreparedSource::prepare(buffer, &self.windows, &changes).map(Some)
    }
}

fn for_buffer(
    edits: &[crate::buffer::edit::Replacement],
    id: BufferId,
) -> Vec<crate::buffer::edit::Replacement> {
    edits
        .iter()
        .filter(|e| e.source.stamp.buffer == id)
        .cloned()
        .collect()
}
fn verify_file(buffer: &Buffer, writing: bool) -> Result<(), String> {
    if writing && buffer.is_readonly() {
        return Err("Move participant is read-only".into());
    }
    let Some(file) = buffer.file() else {
        return Ok(());
    };
    let current = crate::file::resolve(file.path()).map_err(|e| e.to_string())?;
    if current.observation() != file.observation() {
        return Err("Move file was replaced or changed externally; nothing moved".into());
    }
    if writing && current.observation().is_some_and(|o| o.mode & 0o222 == 0) {
        return Err("Move file has no write permission; nothing moved".into());
    }
    Ok(())
}
