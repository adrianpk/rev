use crate::input::Focus;
use crate::pointer::{Event, Hit, Kind};
use crate::view::Error;

use super::{Action, Session};

mod scroll;
mod selection;
pub(super) use selection::Selection;

impl Session {
    pub(super) fn pointer(&mut self, event: Event) -> Action {
        if self.input.focus() != Focus::Editing {
            return Action::Wait;
        }
        match event.kind {
            Kind::Press => self.press(event.position),
            Kind::Drag => self.drag_pointer(event.position),
            Kind::Release => self.release_pointer(event.position),
            Kind::ScrollUp | Kind::ScrollDown => self.pointer_scroll(event),
        }
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
