use crate::input::Focus;
use crate::pointer::{Event, Hit, Kind};
use crate::view::Error;

use super::{Action, Session};

mod autoscroll;
mod scroll;
mod selection;
pub(super) use selection::Selection;

impl Session {
    /// Effective session override; the admitted startup snapshot stays immutable.
    pub fn mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    pub(super) fn toggle_mouse(&mut self) -> Action {
        self.cancel_pointer_capture();
        self.mouse_enabled = !self.mouse_enabled;
        self.message(if self.mouse_enabled {
            "Mouse mode enabled"
        } else {
            "Mouse mode disabled"
        })
    }

    fn mouse_scroll_lines(&self) -> usize {
        self.buffers.snapshot().settings().mouse.scroll_lines
    }

    pub(super) fn pointer(&mut self, event: Event, now: std::time::Instant) -> Action {
        if let Some(action) = self.layout_pointer(event, now) {
            return action;
        }
        self.pointer_now(event, now)
    }

    pub(super) fn pointer_now(&mut self, event: Event, now: std::time::Instant) -> Action {
        if !self.mouse_enabled || self.input.focus() != Focus::Editing {
            return Action::Wait;
        }
        match event.kind {
            Kind::Press => self.press(event.position),
            Kind::Drag => self.drag_pointer(event.position, now),
            Kind::Release => self.release_pointer(event.position, now),
            Kind::ScrollUp | Kind::ScrollDown => self.pointer_scroll(event),
        }
    }

    pub(super) fn prepare_pointer(&self, event: Event) -> Result<(), Error> {
        if matches!(event.kind, Kind::ScrollUp | Kind::ScrollDown) && self.selection.is_some() {
            return Ok(());
        }
        let Some(composition) = self.composition() else {
            return Ok(());
        };
        if matches!(
            crate::pointer::Target::resolve(&composition, event.position),
            Some(crate::pointer::Target::Tree(_))
        ) {
            return Ok(());
        }
        let mut windows = self.windows.clone();
        windows.reveal(&self.buffers, composition.editing_size())?;
        match event.kind {
            Kind::Press => {
                Hit::resolve(&composition, &windows, &self.buffers, event.position)?;
            }
            Kind::Drag | Kind::Release => {
                if let Some(selection) = &self.selection
                    && let Some(panel) = self.capture_panel(selection.capture)
                {
                    let view = windows.selected();
                    view.hit(self.buffer()?, panel.content(), event.position)?;
                }
            }
            Kind::ScrollUp | Kind::ScrollDown if self.selection.is_none() => {
                if let Some(crate::pointer::Target::Editing { view, panel }) =
                    crate::pointer::Target::resolve(&composition, event.position)
                {
                    self.scrolling_editing(view, panel, event.kind == Kind::ScrollUp)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn press(&mut self, position: crate::display::Position) -> Action {
        self.selection = None;
        match self.pointer_press(position) {
            Ok(Some(hit)) => self.focus_hit(hit),
            Ok(None) => Action::Wait,
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn pointer_press(&mut self, position: crate::display::Position) -> Result<Option<Hit>, Error> {
        let composition = self.composition();
        self.reveal_workspace(composition.as_ref())?;
        composition
            .as_ref()
            .map(|composition| Hit::resolve(composition, &self.windows, &self.buffers, position))
            .transpose()
            .map(Option::flatten)
    }

    fn focus_hit(&mut self, hit: Hit) -> Action {
        let result = match hit {
            Hit::Tree => self.focus_tree(),
            Hit::Editing { view, point } => self.focus_editing(view, point),
        };
        match result {
            Ok(()) => self.message(""),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn focus_tree(&mut self) -> Result<(), Error> {
        self.stop_typing()?;
        self.tree
            .focus(self.size)
            .map_err(|error| Error::WorkingDirectory(error.kind()))?;
        self.yank = None;
        self.kills.break_chain();
        self.input.cancel();
        Ok(())
    }

    fn focus_editing(
        &mut self,
        view: crate::window::ViewId,
        point: Option<crate::buffer::edit::Point>,
    ) -> Result<(), Error> {
        self.select_editing_view(view)?;
        self.tree.edit();
        self.yank = None;
        self.input.cancel();
        if let Some(point) = point {
            self.place_point(point)?;
            self.selection = Some(Selection::new(crate::pointer::Capture {
                view,
                anchor: point,
            }));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Runtime mode changes cannot mutate any admitted startup leaf or acquire
    // document history, even though effective admission changes immediately.
    #[test]
    fn frozen_startup() {
        let mut session = Session::empty(crate::view::Size {
            width: 80,
            height: 24,
        })
        .unwrap();
        let snapshot = session.buffers.snapshot().clone();
        for expected in [false, true, false] {
            session.execute(crate::command::Command::MouseMode);
            assert_eq!(session.mouse_enabled(), expected);
            assert_eq!(session.buffers.snapshot(), &snapshot);
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}
