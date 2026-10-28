use super::*;
impl Session {
    pub(in crate::session) fn habit_frame(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        let Some(state) = &self.habit_agenda else {
            return Ok(());
        };
        let Some(composition) = composition else {
            return Ok(());
        };
        let Some((_, panel)) = composition
            .panels()
            .iter()
            .find(|(id, _)| *id == state.view)
        else {
            return Ok(());
        };
        let stale = !self
            .buffers
            .get(state.agenda.source.stamp.buffer)
            .is_ok_and(|b| b.stamp() == state.agenda.source.stamp);
        frame.habit_agenda(
            crate::display::HabitDisplay {
                agenda: &state.agenda,
                policy: &state.policy,
                date: state.date,
                shown: state.shown,
                top: state.top,
                stale,
            },
            panel.content(),
        )
    }
}
