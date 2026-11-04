use super::{Composition, Session, SessionId};
use crate::{display::Frame, panel::Panel};

impl Session {
    pub(in crate::session) fn terminal_frame(
        &self,
        frame: &mut Frame,
        composition: Option<&Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(composition) = composition else {
            return Ok(());
        };
        for (view, panel) in composition.panels() {
            let terminal = self
                .windows
                .get(*view)
                .ok()
                .and_then(|v| self.buffers.get(v.point.stamp.buffer).ok())
                .and_then(|b| b.terminal());
            if let Some(id) = terminal {
                let active = self.terminals.focused && self.terminals.view == Some(*view);
                let viewport = self
                    .windows
                    .get(*view)
                    .expect("live terminal view")
                    .terminal;
                let panel = Panel::new(panel.outer(), true, active);
                if !self.paint_copy(frame, super::copy::Key::View(*view), panel)? {
                    self.paint_terminal(frame, id, panel, &viewport);
                }
            }
        }
        self.terminal_panel_frame(frame, composition)
    }

    fn terminal_panel_frame(
        &self,
        frame: &mut Frame,
        composition: &Composition,
    ) -> Result<(), crate::view::Error> {
        if let Some(panel) = composition.terminal()
            && let Some(id) = self.terminals.selected
            && !self.paint_copy(frame, super::copy::Key::Panel, panel)?
        {
            self.paint_terminal(frame, id, panel, &self.terminals.viewport);
        }
        Ok(())
    }

    fn paint_copy(
        &self,
        frame: &mut Frame,
        key: super::copy::Key,
        panel: Panel,
    ) -> Result<bool, crate::view::Error> {
        let Some(copy) = self.terminals.copies.get(&key) else {
            return Ok(false);
        };
        let state = self.terminals.sessions.get(copy.id).map_or_else(
            || "Unavailable".into(),
            |session| session.state().to_string(),
        );
        let label = if copy.mode {
            format!(" Terminal [COPY] {state}  Return: live")
        } else {
            format!(" Terminal retained output; {state}  C-c C-t: copy")
        };
        copy.frame(frame, panel, &label)?;
        Ok(true)
    }

    fn paint_terminal(
        &self,
        frame: &mut Frame,
        id: SessionId,
        panel: Panel,
        viewport: &crate::term::Viewport,
    ) {
        let Some(session) = self.terminals.sessions.get(id) else {
            return;
        };
        let name = self
            .buffers
            .iter()
            .find(|b| b.terminal() == Some(id))
            .map_or("Terminal", |b| b.name());
        frame.terminal(
            session,
            panel,
            &format!(" {name}  {}  C-g: editor", session.state()),
            viewport,
        );
    }
}
