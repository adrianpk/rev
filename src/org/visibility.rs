mod insertion;
mod model;
mod navigation;
mod state;
mod subtree;
pub(crate) use insertion::Insertion;
pub(crate) use subtree::SubtreeEdit;
pub(super) use subtree::movement_pieces;

use std::ops::Range;
use std::sync::Arc;

use crate::buffer::Stamp;
use crate::command::OrgCommand;

use super::{Cancellation, Coverage, Structure};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Heading {
    start: usize,
    body: usize,
    end: usize,
    section: usize,
    parent: Option<usize>,
    level: usize,
    title: Range<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Drawer {
    start: usize,
    body: Range<usize>,
    end: usize,
}

/// A bounded native fold model prepared by the existing observation worker.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Outline {
    pub(crate) stamp: Stamp,
    coverage: Coverage,
    headings: Vec<Heading>,
    drawers: Vec<Drawer>,
    blocked: Vec<Range<usize>>,
    blocked_eof: bool,
    malformed: Vec<Range<usize>>,
    pub(super) lists: super::list::Lists,
    pub(super) properties: super::property::Properties,
}

impl Outline {
    pub(crate) fn observe(
        structure: &Structure,
        text: &ropey::Rope,
        cancel: &Cancellation,
    ) -> Self {
        let mut model = model::observe(structure, cancel);
        model.lists = super::list::Lists::observe(structure, text, cancel);
        model.properties = super::property::Properties::observe(structure, text, cancel);
        model
    }
    fn heading(&self, byte: usize) -> Option<usize> {
        self.headings
            .partition_point(|h| h.start <= byte)
            .checked_sub(1)
            .filter(|i| byte <= self.headings[*i].end)
    }
    fn drawer(&self, byte: usize) -> Option<usize> {
        self.drawers
            .iter()
            .position(|d| d.start <= byte && byte < d.end)
    }

    pub(super) fn complete(&self) -> Result<(), &'static str> {
        if self.coverage != Coverage::Complete {
            return Err("Org structural observation is incomplete");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Level {
    Full,
    Closed,
    Children,
}
impl Level {
    fn next(self) -> Self {
        match self {
            Self::Full => Self::Closed,
            Self::Closed => Self::Children,
            Self::Children => Self::Full,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Visibility {
    pub(crate) model: Arc<Outline>,
    levels: Vec<Level>,
    drawers: Vec<bool>,
    global: u8,
}

pub(crate) struct Hidden {
    pub(crate) bytes: Range<usize>,
    pub(crate) anchor: usize,
}

impl Visibility {
    pub(crate) fn new(model: Arc<Outline>) -> Self {
        Self {
            levels: vec![Level::Full; model.headings.len()],
            drawers: vec![false; model.drawers.len()],
            model,
            global: 0,
        }
    }
    pub(crate) fn change(&self, command: OrgCommand, byte: usize) -> Result<Self, &'static str> {
        if self.model.coverage != Coverage::Complete {
            return Err("Org visibility observation is incomplete");
        }
        let mut next = self.clone();
        next.dispatch(command, byte)?;
        Ok(next)
    }
    pub(crate) fn spans(&self) -> Vec<Hidden> {
        let headings = self
            .model
            .headings
            .iter()
            .zip(&self.levels)
            .filter_map(|(h, level)| {
                let end = match level {
                    Level::Full => h.body,
                    Level::Closed => h.end,
                    Level::Children => h.section,
                };
                (h.body < end).then_some(Hidden {
                    bytes: h.body..end,
                    anchor: h.start,
                })
            });
        let drawers = self
            .model
            .drawers
            .iter()
            .zip(&self.drawers)
            .filter(|(d, hidden)| **hidden && !d.body.is_empty())
            .map(|(d, _)| Hidden {
                bytes: d.body.clone(),
                anchor: d.start,
            });
        let mut spans: Vec<_> = headings.chain(drawers).collect();
        spans.sort_by_key(|s| s.bytes.start);
        spans
    }
    pub(crate) fn reveal(&self, byte: usize) -> Self {
        let mut next = self.clone();
        if let Some(index) = self.model.heading(byte) {
            next.reveal_heading(index, byte);
        }
        if let Some(index) = self.model.drawer(byte) {
            next.drawers[index] = false;
        }
        next
    }
    fn reveal_heading(&mut self, index: usize, byte: usize) {
        if byte >= self.model.headings[index].body {
            self.levels[index] = Level::Full;
        }
        let mut parent = self.model.headings[index].parent;
        while let Some(index) = parent {
            if self.levels[index] == Level::Closed {
                self.levels[index] = Level::Children;
            }
            parent = self.model.headings[index].parent;
        }
    }
}
