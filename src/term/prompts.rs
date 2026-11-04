use alacritty_terminal::{
    Term,
    event::EventListener,
    grid::Dimensions,
    index::{Column, Line, Point},
    term::TermMode,
};
use std::collections::VecDeque;
use unicode_width::UnicodeWidthChar;

const LIMIT: usize = 1024;
struct Prompt {
    point: Point,
    printed: bool,
}

/// Actual OSC 133 A reports, bounded independently of history length. Ordinary
/// full-grid scrolling preserves their coordinates. Unproven grid transforms
/// invalidate evidence instead of treating shell-looking text as a prompt.
pub(super) struct Prompts {
    entries: VecDeque<Prompt>,
    history: usize,
    full: bool,
}
impl Prompts {
    pub fn new(history: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            history,
            full: true,
        }
    }
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.entries.iter().map(|p| p.point)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn reset(&mut self) {
        self.clear();
        self.full = true;
    }
    pub fn observe(&mut self, bytes: &[u8], term: &Term<impl EventListener>) {
        if !self.full
            || term.mode().contains(TermMode::ALT_SCREEN)
            || term.grid().cursor.input_needs_wrap
        {
            return;
        }
        let Some(suffix) = bytes.strip_prefix(b"\x1b]133;A") else {
            return;
        };
        if !valid_suffix(suffix) {
            return;
        }
        let point = term.grid().cursor.point;
        self.entries.retain(|entry| entry.point != point);
        if self.entries.len() == LIMIT {
            self.entries.pop_front();
        }
        self.entries.push_back(Prompt {
            point,
            printed: false,
        });
    }
    pub fn region(&mut self, top: usize, bottom: Option<usize>, rows: usize) {
        let bottom = bottom.unwrap_or(rows);
        if top >= bottom {
            return;
        }
        self.clear();
        self.full = top == 1 && bottom >= rows;
    }
    pub fn scroll(&mut self, lines: usize) {
        for entry in &mut self.entries {
            entry.point.line.0 -= lines as i32;
        }
        self.entries
            .retain(|entry| entry.point.line.0 >= -(self.history as i32));
    }
    pub fn linefeed(&mut self, term: &Term<impl EventListener>) {
        for entry in &mut self.entries {
            if entry.point.line == term.grid().cursor.point.line {
                entry.printed = true;
            }
        }
        if self.full && term.grid().cursor.point.line.0 + 1 == term.grid().screen_lines() as i32 {
            self.scroll(1);
        }
    }
    pub fn erase_row(&mut self, line: Line) {
        self.entries.retain(|entry| entry.point.line != line);
    }
    pub fn input(&mut self, term: &Term<impl EventListener>, character: char) {
        let Some(width @ 1..=2) = character.width() else {
            return;
        };
        let mut point = term.grid().cursor.point;
        if wraps(term, width) {
            self.linefeed(term);
            point.line.0 = (point.line.0 + 1).min(term.grid().screen_lines() as i32 - 1);
            point.column = Column(0);
        }
        if term.mode().contains(TermMode::INSERT) {
            self.erase_row(point.line);
        }
        self.overwrite(point, width);
    }
    fn overwrite(&mut self, point: Point, width: usize) {
        self.entries.retain_mut(|entry| {
            if entry.point.line != point.line {
                return true;
            }
            if !(point.column.0..point.column.0 + width).contains(&entry.point.column.0) {
                return true;
            }
            let keep = !entry.printed;
            entry.printed = true;
            keep
        });
    }
}
fn wraps(term: &Term<impl EventListener>, width: usize) -> bool {
    let cursor = &term.grid().cursor;
    term.mode().contains(TermMode::LINE_WRAP)
        && (cursor.input_needs_wrap
            || (width == 2 && cursor.point.column.0 + 1 >= term.grid().columns()))
}
fn valid_suffix(suffix: &[u8]) -> bool {
    let Some(body) = suffix
        .strip_suffix(b"\x07")
        .or_else(|| suffix.strip_suffix(b"\x1b\\"))
    else {
        return false;
    };
    body.is_empty() || body.starts_with(b";")
}
