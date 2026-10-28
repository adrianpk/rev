use super::*;
impl Session {
    pub(in crate::session) fn agenda_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(state) = &self.calendar_agenda else {
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
        let stale = state
            .calendar
            .sources
            .iter()
            .any(|source| match source.version {
                crate::org::agenda::SourceVersion::Buffer(stamp) => !self
                    .buffers
                    .get(stamp.buffer)
                    .is_ok_and(|b| b.stamp() == stamp),
                crate::org::agenda::SourceVersion::Disk(_) => false,
            });
        frame.org_agenda(
            crate::display::AgendaDisplay {
                calendar: &state.calendar,
                top: state.top,
                selected: state.cursor,
                active: self.calendar_active(),
                stale,
            },
            panel.content(),
        )
    }
}
