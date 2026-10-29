use super::*;
impl Session {
    pub(super) fn attachment_prompt(
        &mut self,
        stage: Stage,
        label: &'static str,
        feedback: &str,
    ) -> Result<(), String> {
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.attachment
            .as_mut()
            .ok_or("No attachment interaction")?
            .stage = stage;
        self.pending = Some(Pending::Attachment);
        self.message(feedback);
        Ok(())
    }
    pub(in crate::session) fn attachment_argument(&mut self, draft: &str) -> Action {
        let Some(state) = self.attachment.as_ref() else {
            return self.error("No attachment interaction");
        };
        if !self.org_source_current(&state.request) {
            return self.reject_attachment("Heading source or view changed");
        }
        let stage = std::mem::replace(
            &mut self.attachment.as_mut().expect("owned state").stage,
            Stage::Dispatcher,
        );
        let result = match stage {
            Stage::Dispatcher => self.attachment_dispatch(draft),
            Stage::Source(method) => self.prepare_attachment(draft, method),
            Stage::Directory => self.attachment_directory(draft),
            Stage::Choose(listing, native) => self.open_attachment(listing, native, draft),
            Stage::Review(plan) => self.confirm_attachment(*plan, draft),
            _ => Err("Attachment worker still owns its operation".into()),
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => self.reject_attachment(&e),
        }
    }
    fn attachment_dispatch(&mut self, draft: &str) -> Result<(), String> {
        let command = match draft {
            "a" | "c" => C::AttachCopy,
            "m" => C::AttachMove,
            "l" => C::AttachLink,
            "o" => C::AttachOpen,
            "O" => C::AttachOpenNative,
            "f" => C::AttachList,
            "s" => C::AttachDirectory,
            _ => return self.attachment_command(C::Attach),
        };
        self.attachment_command(command)
    }
    fn prepare_attachment(&mut self, draft: &str, method: Method) -> Result<(), String> {
        let state = self.attachment.as_mut().ok_or("No attachment source")?;
        self.org.start_attachment_plan(
            state.request.origin.point.stamp,
            state.context.clone(),
            draft.to_owned(),
            method,
        )?;
        state.stage = Stage::Running;
        self.message("Checking attachment paths and collisions; source unchanged");
        Ok(())
    }
    fn confirm_attachment(&mut self, plan: Plan, draft: &str) -> Result<(), String> {
        match draft {
            "no" => {
                self.attachment = None;
                self.message("Attachment declined; files and heading unchanged");
                Ok(())
            }
            "yes" => self.launch_attachment(plan),
            _ => self.attachment_prompt(
                Stage::Review(Box::new(plan)),
                "Confirm attachment yes/no: ",
                "Enter yes or no; C-g cancels",
            ),
        }
    }
    fn launch_attachment(&mut self, plan: Plan) -> Result<(), String> {
        let _ = self.attachment_edits(&plan)?;
        let state = self.attachment.as_mut().ok_or("No attachment source")?;
        self.org.start_attachment_effect(
            state.request.origin.point.stamp,
            plan,
            state.context.policy.limits.job_seconds,
        )?;
        state.stage = Stage::Running;
        self.message("Attachment file operation running; C-g cancels");
        Ok(())
    }
    fn attachment_directory(&mut self, draft: &str) -> Result<(), String> {
        let state = self.attachment.as_ref().ok_or("No attachment source")?;
        let root = crate::org::attachment::location(
            Some(draft),
            None,
            &state.context.base,
            &state.context.policy,
        )?;
        let b = self.buffer().map_err(|e| e.to_string())?;
        let entry = state
            .model
            .property_target(b, state.request.origin.point.position)?;
        let edits = state.model.property_edit(
            b,
            entry,
            "DIR",
            Some(root.to_str().ok_or("Attachment DIR is not UTF-8")?),
        )?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        self.attachment = None;
        self.message("Attachment DIR changed; no directory created; source undo available");
        Ok(())
    }
    fn open_attachment(
        &mut self,
        listing: Listing,
        native: bool,
        draft: &str,
    ) -> Result<(), String> {
        let name = draft
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| listing.names.get(n))
            .or_else(|| listing.names.iter().find(|s| s.as_str() == draft))
            .ok_or("Select an exact attachment name or displayed number")?
            .clone();
        let state = self.attachment.take().ok_or("No attachment source")?;
        let target = state.context.selected(&name, native)?;
        self.follow_attachment(target, state.request, name, native)
    }
}
