mod capture;
mod preflight;
mod replay;
mod selection;
mod text;

use crate::buffer::{Stamp, edit::Point};
use crate::command::{Editing, ModalCommand};
use crate::kill::Shape;
use crate::window::ViewId;

const TEXT_BYTES: usize = 1024;

#[derive(Default)]
pub(super) struct State {
    last: Option<Record>,
    pending: Option<Pending>,
    text: bool,
    replaying: bool,
}

#[derive(Clone)]
struct Record {
    intent: Intent,
    text: String,
}

#[derive(Clone)]
enum Intent {
    Entry(ModalCommand),
    Edit(Editing, Option<selection::Geometry>),
    Paste { request: Editing, payload: Payload },
}

#[derive(Clone)]
enum Payload {
    Linear(Shape, String),
    Block(crate::rectangle::Rectangle),
}

struct Pending {
    view: ViewId,
    point: Point,
    first: Option<Stamp>,
    record: Record,
    valid: bool,
    changed: bool,
    trim: Trim,
}

pub(super) struct Observation {
    point: Point,
    record: Option<Record>,
    entry: bool,
    preserve: bool,
}

#[derive(Clone, Copy)]
enum Trim {
    None,
    Prefix,
    Suffix,
}
