use super::*;
impl Session {
    pub(in crate::session) fn custom_calendar(&mut self, command: C) -> Action {
        match command {
            C::AgendaDay => self.change_custom(C::AgendaSpan, "day"),
            C::AgendaWeek => self.change_custom(C::AgendaSpan, "week"),
            C::AgendaDate => self.custom_prompt(C::AgendaDate, ""),
            _ => self.change_custom(command, ""),
        }
    }
    pub(super) fn change_custom(&mut self, command: C, draft: &str) -> Action {
        match self.prepare_custom(command, draft) {
            Ok(request) => match self.org.start_sources(
                self.view().point.stamp,
                super::super::agenda::Work::Custom(Box::new(request)),
            ) {
                Ok(()) => {
                    self.message("Agenda presentation pending; C-g cancels; sources retained")
                }
                Err(e) => self.error(&e),
            },
            Err(e) => {
                self.custom_prompt(command, draft);
                self.error(&e)
            }
        }
    }
    fn prepare_custom(&self, command: C, draft: &str) -> Result<Request, String> {
        let (mut definition, baseline, mut context, selection) = self.custom_definition()?;
        context = options::context(context, command, draft)?;
        if !matches!(
            command,
            C::AgendaNext | C::AgendaPrevious | C::AgendaToday | C::AgendaDate
        ) {
            options::apply(&mut definition, &baseline, command, draft)?;
        }
        let policy = self.buffers.snapshot().settings().org.clone();
        for block in &definition.blocks {
            crate::org::agenda::AgendaQuery::block(block, &policy.limits)?;
        }
        let sources = self
            .agenda_sources
            .as_ref()
            .ok_or("Refresh agenda sources first")?
            .clone();
        Ok(Request {
            scan: None,
            sources: Some(sources),
            definition,
            baseline,
            context,
            policy,
            selection,
            view: self.windows.selected_id(),
        })
    }
}
