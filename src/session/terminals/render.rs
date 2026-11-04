use super::{Composition, Session, SessionId};
use crate::{display::Frame, panel::Panel};

impl Session {
    pub(in crate::session) fn terminal_frame(
        &self,
        frame: &mut Frame,
        composition: Option<&Composition>,
    ) {
        let Some(composition) = composition else {
            return;
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
                self.paint_terminal(frame, id, Panel::new(panel.outer(), true, active));
            }
        }
        if let Some(panel) = composition.terminal()
            && let Some(id) = self.terminals.selected
        {
            self.paint_terminal(frame, id, panel);
        }
    }

    fn paint_terminal(&self, frame: &mut Frame, id: SessionId, panel: Panel) {
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
            &format!(" {name}  {:?}  C-g: editor", session.state()),
        );
    }
}
