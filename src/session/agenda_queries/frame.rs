use super::*;
impl Session {
    pub(in crate::session) fn query_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(state) = &self.query_agenda else {
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
            .result
            .sources
            .iter()
            .any(|source| match source.version {
                crate::org::agenda::SourceVersion::Buffer(stamp) => !self
                    .buffers
                    .get(stamp.buffer)
                    .is_ok_and(|b| b.stamp() == stamp),
                crate::org::agenda::SourceVersion::Disk(_) => false,
            });
        frame.org_query(
            crate::display::QueryDisplay {
                result: &state.result,
                top: state.top,
                selected: state.cursor,
                active: self.query_active() && self.input.focus() == crate::input::Focus::Editing,
                stale,
                columns: state.columns,
                marked: &self.agenda_marks.targets,
            },
            panel.content(),
        )
    }
}
