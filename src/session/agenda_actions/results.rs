use super::*;
impl Session {
    pub(super) fn show_agenda_results(&mut self) -> Action {
        if self.agenda_results.is_empty() {
            return self.message("No completed agenda source action results");
        }
        self.agenda_results_view = Some((self.windows.selected_id(), 0));
        self.message("Agenda action results; q returns to agenda; source buffers remain canonical")
    }
    pub(super) fn agenda_results_command(&mut self, command: Command) -> Option<Action> {
        let (owner, top) = self.agenda_results_view?;
        if owner != self.windows.selected_id() {
            return None;
        }
        match command {
            Command::KeyboardQuit => {
                self.agenda_results_view = None;
                Some(self.message("Returned to agenda; action results retained"))
            }
            Command::Window(_) | Command::Quit | Command::ExecuteExtended => None,
            Command::Org(C::AgendaRefresh) => {
                self.agenda_results_view = None;
                None
            }
            Command::Scroll(scroll) => {
                let next = results_top(scroll, top, self.agenda_results.len());
                self.agenda_results_view = Some((owner, next));
                Some(Action::Redraw)
            }
            _ => Some(self.message("Agenda action results; q returns to agenda; g refreshes")),
        }
    }
    pub(in crate::session) fn agenda_results_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some((owner, top)) = self.agenda_results_view else {
            return Ok(());
        };
        let Some(composition) = composition else {
            return Ok(());
        };
        let Some((_, panel)) = composition.panels().iter().find(|(id, _)| *id == owner) else {
            return Ok(());
        };
        frame.org_action_results(
            &self.agenda_results,
            top,
            panel.content(),
            owner == self.windows.selected_id(),
        )?;
        Ok(())
    }
    pub(super) fn undo_agenda_action(&mut self) -> Action {
        let result = self.agenda_results.iter().rev().find(|r| {
            matches!(
                r.status,
                ActionStatus::Changed | ActionStatus::ChangedWithFailure
            )
        });
        let Some(stamp) = result.and_then(|r| r.after) else {
            return self.error("No unchanged agenda edit is available to undo");
        };
        let point = Point {
            stamp,
            position: crate::coordinates::Scalar(0),
        };
        let outcome = self
            .buffers
            .get(stamp.buffer)
            .map_err(|e| e.to_string())
            .and_then(|buffer| self.windows.source_action(buffer, point));
        if let Err(e) = outcome {
            return self.error(&e);
        }
        let action = self.history(crate::command::History::Undo);
        self.windows.restore_source_action();
        self.sync_input_context();
        action
    }
}

fn results_top(scroll: crate::command::Scroll, top: usize, count: usize) -> usize {
    match scroll {
        crate::command::Scroll::NextLine | crate::command::Scroll::PageForward => {
            top.saturating_add(1).min(count.saturating_sub(1))
        }
        _ => top.saturating_sub(1),
    }
}
