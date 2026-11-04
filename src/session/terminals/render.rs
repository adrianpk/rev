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
                if let Some(copy) = self.terminals.copies.get(&super::copy::Key::View(*view)) {
                    copy.frame(
                        frame,
                        Panel::new(panel.outer(), true, active),
                        if copy.mode {
                            " Terminal [COPY]  Return: live"
                        } else {
                            " Terminal retained output; C-c C-t: copy"
                        },
                    )?;
                    continue;
                }
                self.paint_terminal(
                    frame,
                    id,
                    Panel::new(panel.outer(), true, active),
                    &viewport,
                );
            }
        }
        if let Some(panel) = composition.terminal()
            && let Some(id) = self.terminals.selected
        {
            if let Some(copy) = self.terminals.copies.get(&super::copy::Key::Panel) {
                copy.frame(
                    frame,
                    panel,
                    if copy.mode {
                        " Terminal [COPY]  Return: live"
                    } else {
                        " Terminal retained output; C-c C-t: copy"
                    },
                )?;
                return Ok(());
            }
            self.paint_terminal(frame, id, panel, &self.terminals.viewport);
        }
        Ok(())
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
            &format!(" {name}  {:?}  C-g: editor", session.state()),
            viewport,
        );
    }
}
