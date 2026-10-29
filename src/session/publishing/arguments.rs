use super::*;
impl Session {
    pub(in crate::session) fn publishing_argument(
        &mut self,
        prompt: Prompt,
        draft: &str,
    ) -> Action {
        let result = match prompt {
            Prompt::Select(request) => self.select_project(request, draft),
            Prompt::Replace(prepared) => self.replace_publication(*prepared, draft),
            Prompt::Upload(upload) => self.upload_project(upload, draft),
            Prompt::Result => return self.publishing_result(draft),
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => {
                self.end_publishing(&e);
                self.error(&e)
            }
        }
    }
    fn select_project(&mut self, request: Request, name: &str) -> Result<(), String> {
        let selection = Selection::configured(self.buffers.snapshot(), name)?;
        let time = self
            .org_clock
            .unwrap_or(super::super::outline::timestamps::capture)(
            &self.buffers.snapshot().settings().org.time,
        )?;
        let work = publish::Work::new(&self.buffers, selection, time)?;
        self.publishing_results.clear();
        self.publishing = Some(State {
            request,
            plan: None,
            inputs: Vec::new(),
            active: None,
            cancelled: false,
            evidence: Vec::new(),
        });
        self.launch_publishing(JobWork::Discover(work))
    }
    fn replace_publication(&mut self, prepared: Prepared, draft: &str) -> Result<(), String> {
        if !self.publishing_current() {
            return Err("Project source changed before replacement".into());
        }
        match draft {
            "yes" => self.write_publication(prepared),
            "no" => {
                self.publishing_results
                    .push(prepared.failed("Replacement declined"));
                self.publishing.as_mut().expect("selected project").active = None;
                self.next_publication()
            }
            _ => Err("Project replacement requires yes or no".into()),
        }
    }
    fn upload_project(&mut self, upload: publish::Upload, draft: &str) -> Result<(), String> {
        match draft {
            "yes" => {
                if !self.publishing_current() {
                    return Err("Project changed before upload confirmation".into());
                }
                self.launch_publishing(JobWork::Upload(upload))
            }
            "no" => {
                self.publishing = None;
                self.message("Project files published; upload declined");
                Ok(())
            }
            _ => Err("Upload requires yes or no".into()),
        }
    }
    pub(in crate::session) fn cancel_publishing(&mut self, command: Command) -> Option<Action> {
        if command != Command::KeyboardQuit {
            return None;
        }
        if self.publishing.is_none() && !matches!(self.pending, Some(Pending::Publish(_))) {
            return None;
        }
        if let Some(Pending::Publish(prompt)) = self.pending.take()
            && let Prompt::Replace(prepared) = *prompt
        {
            self.publishing_results
                .push(prepared.failed("Project cancelled before replacement"));
            if let Some(state) = &mut self.publishing {
                state.active = None;
            }
        }
        self.input.cancel();
        if self.org.publish_running {
            if let Some(state) = &mut self.publishing {
                state.cancelled = true;
            }
            self.org.cancel_publishing();
            return Some(
                self.message("Project cancellation requested; awaiting actual file outcome"),
            );
        }
        self.end_publishing("Project cancelled");
        Some(Action::Redraw)
    }
    pub(super) fn publishing_complete(&mut self) -> Result<(), String> {
        let state = self
            .publishing
            .as_ref()
            .ok_or("Project selection cancelled")?;
        let plan = state
            .plan
            .as_ref()
            .ok_or("Project discovery returned no plan")?;
        if !plan.selection.project.upload
            || self
                .publishing_results
                .iter()
                .any(|r| r.publication != Publication::Complete)
        {
            self.publishing = None;
            self.show_publishing_summary("Project publication finished");
            return Ok(());
        }
        let upload = publish::Upload::configured(self.buffers.snapshot(), &plan.selection.output)?;
        let label = format!(
            "Upload project {} with {}? yes/no: ",
            plan.selection.project.name,
            upload.label()
        );
        self.publishing_prompt(Prompt::Upload(upload), &label)
    }
}
