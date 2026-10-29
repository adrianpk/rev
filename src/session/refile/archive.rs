use super::*;
use crate::org::archive::Archive;
impl Session {
    pub(in crate::session) fn start_archive(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        // Reuse source, capture, read-only and pending interaction admission.
        let _ = self.refile_subtree(&model, &request)?;
        if request.command == C::ToggleArchiveTag {
            return self.toggle_archive(&model, &request);
        }
        let work = self.archive_work(&model, &request)?;
        let subtree = self.refile_subtree(&model, &request)?;
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org.start_archive(work, seconds)?;
        self.refile = Some(State {
            request,
            subtree,
            copy: false,
            targets: None,
        });
        self.message("Archive validation pending; source unchanged; C-g cancels");
        Ok(())
    }
    fn toggle_archive(&mut self, model: &Outline, request: &Request) -> Result<(), String> {
        let archiving = !model.archive_tagged(
            self.buffer().map_err(|e| e.to_string())?,
            request.origin.point.position,
        )?;
        let edits = model.archive_tag(
            self.buffer().map_err(|e| e.to_string())?,
            request.origin.point.position,
        )?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        if archiving {
            self.org_command(C::HideSubtree);
        }
        self.message("Org archive tag toggled; source retained; agenda policy applies");
        Ok(())
    }
    fn archive_work(&self, model: &Outline, request: &Request) -> Result<Archive, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let policy = &self.buffers.snapshot().settings().org;
        let mut context = model.archive_context(buffer, request.origin.point.position, policy)?;
        if request.command == C::ArchiveSibling {
            let target = model.archive_sibling(buffer, request.origin.point.position)?;
            return context.sibling(&self.buffers, policy, target);
        }
        let now =
            self.org_clock
                .unwrap_or(crate::session::outline::timestamps::capture)(&policy.time)?
            .now();
        let body = context.body(buffer, model, now.format_timestamp(false))?;
        context.external(&self.buffers, policy, now.date, body)
    }
}
