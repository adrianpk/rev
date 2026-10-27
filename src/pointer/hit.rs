use crate::buffer::{Buffers, edit::Point};
use crate::display::Position;
use crate::panel::Composition;
use crate::view::Error;
use crate::window::{ViewId, Windows};

use super::Target;

/// Disposable destination from the current projection, never a cached hit map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Tree,
    Editing { view: ViewId, point: Option<Point> },
}

impl Hit {
    pub fn resolve(
        composition: &Composition,
        windows: &Windows,
        buffers: &Buffers,
        position: Position,
    ) -> Result<Option<Self>, Error> {
        let (id, panel) = match Target::resolve(composition, position) {
            Some(Target::Tree(_)) => return Ok(Some(Self::Tree)),
            Some(Target::Editing { view, panel }) => (view, panel),
            None => return Ok(None),
        };
        let view = windows.get(id)?;
        let buffer = buffers.get(view.point.stamp.buffer)?;
        let point = view.hit(buffer, panel.content(), position)?;
        Ok(Some(Self::Editing { view: id, point }))
    }
}
