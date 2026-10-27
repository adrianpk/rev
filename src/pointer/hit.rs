use crate::buffer::{Buffers, edit::Point};
use crate::display::Position;
use crate::panel::Composition;
use crate::view::Error;
use crate::window::{ViewId, Windows};

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
        if composition
            .tree()
            .is_some_and(|panel| panel.outer().local(position).is_some())
        {
            return Ok(Some(Self::Tree));
        }
        let Some((id, panel)) = composition
            .panels()
            .iter()
            .find(|(_, panel)| panel.outer().local(position).is_some())
        else {
            return Ok(None);
        };
        let view = windows.get(*id)?;
        let buffer = buffers.get(view.point.stamp.buffer)?;
        let point = view.hit(buffer, panel.content(), position)?;
        Ok(Some(Self::Editing { view: *id, point }))
    }
}
