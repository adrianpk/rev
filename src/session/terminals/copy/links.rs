use super::snapshot::Snapshot;
use crate::term::Session;
use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
    term::cell::Flags,
};
use std::ops::Range;

const LIMIT: usize = 64 * 1024;
const SPANS: usize = 1024;
struct Link {
    range: Range<usize>,
    uri: String,
}
pub(super) struct Links {
    spans: Vec<Link>,
    charge: usize,
}
impl Default for Links {
    fn default() -> Self {
        Self {
            spans: Vec::with_capacity(SPANS),
            charge: SPANS * size_of::<Link>(),
        }
    }
}
impl Links {
    pub fn capture(session: &Session, snapshot: &Snapshot) -> Self {
        let mut links = Self::default();
        let grid = session.screen().grid();
        for (row, range) in snapshot.rows.iter().enumerate() {
            let line = Line(row as i32 - snapshot.history as i32);
            let mut start = range.start;
            for column in 0..grid.columns() {
                let cell = &grid[Point::new(line, Column(column))];
                let end = start + super::snapshot::cell(cell).chars().count();
                if !cell.flags.contains(Flags::HIDDEN)
                    && let Some(link) = cell.hyperlink()
                {
                    links.record(start..end, link.uri());
                }
                start = end;
            }
        }
        links
    }
    fn record(&mut self, range: Range<usize>, uri: &str) {
        if range.is_empty() {
            return;
        }
        if let Some(last) = self.spans.last_mut()
            && last.range.end == range.start
            && last.uri == uri
        {
            last.range.end = range.end;
            return;
        }
        let bytes = uri.len();
        if self.spans.len() >= SPANS || self.charge + bytes > LIMIT {
            return;
        }
        self.spans.push(Link {
            range,
            uri: uri.into(),
        });
        self.charge += bytes;
    }
    pub fn at(&self, range: Range<usize>) -> Option<&str> {
        self.spans
            .iter()
            .find(|link| link.range.start <= range.start && link.range.end >= range.end)
            .map(|link| link.uri.as_str())
    }
}
