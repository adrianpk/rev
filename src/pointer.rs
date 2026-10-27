use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::buffer::edit::Point;
use crate::display::Position;
use crate::window::ViewId;

/// Native operations admitted from the existing terminal decoder. Hover,
/// secondary buttons and horizontal scrolling have no operation in this contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Press,
    Drag,
    Release,
    ScrollUp,
    ScrollDown,
}

/// Zero-based terminal cells, distinct from a document point. The live composed
/// geometry and ordinary view owner must resolve any later text operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub kind: Kind,
    pub position: Position,
    pub modifiers: KeyModifiers,
}

impl Event {
    pub fn from_mouse(event: MouseEvent) -> Option<Self> {
        let kind = match event.kind {
            MouseEventKind::Down(MouseButton::Left) => Kind::Press,
            MouseEventKind::Drag(MouseButton::Left) => Kind::Drag,
            MouseEventKind::Up(MouseButton::Left) => Kind::Release,
            MouseEventKind::ScrollUp => Kind::ScrollUp,
            MouseEventKind::ScrollDown => Kind::ScrollDown,
            _ => return None,
        };
        Some(Self {
            kind,
            position: Position {
                column: event.column,
                row: event.row,
            },
            modifiers: event.modifiers,
        })
    }
}

/// One originating view and revision-qualified anchor for later linear capture.
/// It retains no rendered text, event queue, device state or timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capture {
    pub view: ViewId,
    pub anchor: Point,
}
