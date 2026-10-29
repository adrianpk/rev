use super::*;
#[derive(Default)]
pub(in crate::session) struct Marks {
    pub owner: Option<ViewId>,
    pub targets: Vec<ActionTarget>,
}
impl Session {
    pub(in crate::session) fn clear_agenda_marks(&mut self) {
        self.agenda_marks = Marks::default();
    }
    pub(super) fn agenda_bulk_command(&mut self, command: C) -> Action {
        if self.org.busy() {
            return self.message("Org observation pending; C-g cancels");
        }
        match command {
            C::AgendaMark => self.mark_agenda_entry(),
            C::AgendaUnmark => self.unmark_agenda_entry(),
            C::AgendaUnmarkAll => {
                self.clear_agenda_marks();
                self.message("Agenda marks cleared")
            }
            C::AgendaMarkAll => self.mark_all_agenda_entries(),
            C::AgendaBulk => self.prompt_agenda_bulk(super::BulkPrompt::Action, ""),
            C::AgendaResults => self.show_agenda_results(),
            C::AgendaUndo => self.undo_agenda_action(),
            _ => unreachable!("bulk command"),
        }
    }
    fn mark_agenda_entry(&mut self) -> Action {
        let target = match self.selected_agenda_target() {
            Ok(t) => t,
            Err(e) => return self.error(&e),
        };
        if self.agenda_marks.owner != Some(self.windows.selected_id()) {
            self.clear_agenda_marks();
        }
        if self.agenda_marks.targets.iter().any(|t| t.same(&target)) {
            return self.message("Agenda target already marked");
        }
        if self.agenda_marks.targets.len() == 256 {
            return self.error("Agenda selection exceeds 256 distinct targets");
        }
        self.agenda_marks.owner = Some(self.windows.selected_id());
        self.agenda_marks.targets.push(target);
        self.execute(Command::Scroll(crate::command::Scroll::NextLine));
        self.message(&format!(
            "{} source targets marked",
            self.agenda_marks.targets.len()
        ))
    }
    fn unmark_agenda_entry(&mut self) -> Action {
        let target = match self.selected_agenda_target() {
            Ok(t) => t,
            Err(e) => return self.error(&e),
        };
        self.agenda_marks.targets.retain(|t| !t.same(&target));
        self.message("Agenda target unmarked")
    }
    fn mark_all_agenda_entries(&mut self) -> Action {
        let mut targets = Vec::new();
        for target in self.visible_agenda_targets() {
            if !targets.iter().any(|t: &ActionTarget| t.same(&target)) {
                targets.push(target);
            }
            if targets.len() > 256 {
                break;
            }
        }
        if targets.len() > 256 {
            return self
                .error("Agenda selection exceeds 256 distinct targets; existing marks retained");
        }
        self.agenda_marks = Marks {
            owner: Some(self.windows.selected_id()),
            targets,
        };
        self.message(&format!(
            "{} visible source targets marked",
            self.agenda_marks.targets.len()
        ))
    }
    fn visible_agenda_targets(&self) -> impl Iterator<Item = ActionTarget> + '_ {
        let query = self
            .query_agenda
            .as_ref()
            .filter(|_| self.query_active())
            .into_iter()
            .flat_map(|s| {
                s.result.rows.iter().map(|r| ActionTarget {
                    source: s.result.sources[r.source].clone(),
                    byte: r.byte,
                    title: r.title.clone(),
                })
            });
        let calendar = self
            .calendar_agenda
            .as_ref()
            .filter(|_| self.calendar_active())
            .into_iter()
            .flat_map(|s| {
                s.calendar.rows.iter().map(|r| ActionTarget {
                    source: s.calendar.sources[r.source].clone(),
                    byte: r.byte,
                    title: r.title.clone(),
                })
            });
        query.chain(calendar)
    }
}
