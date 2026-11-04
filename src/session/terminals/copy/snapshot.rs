use crate::{
    buffer::Buffer,
    coordinates::Scalar,
    term::{Session, Viewport},
};
use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
    term::cell::{Cell, Flags},
};

pub(super) const TEXT_BYTES: usize = 1024 * 1024;
pub(super) const EDITOR_BYTES: usize = 8 * 1024 * 1024;
pub(super) const STATES: usize = 32;

pub(super) struct Snapshot {
    pub text: String,
    pub rows: Vec<std::ops::Range<usize>>,
    pub history: usize,
}

impl Snapshot {
    pub fn capture(session: &Session) -> Result<Self, String> {
        let grid = session.screen().grid();
        let history = grid.history_size();
        let mut snapshot = Self {
            text: String::new(),
            rows: Vec::new(),
            history,
        };
        let mut scalars = 0;
        for line in -(history as i32)..i32::from(session.geometry().rows) {
            let row = row(session, Line(line))?;
            let start = scalars;
            scalars += row.chars().count();
            snapshot.rows.push(start..scalars);
            // Physical rows remain addressable by native row/column selection.
            let separator = usize::from(line + 1 < i32::from(session.geometry().rows));
            if snapshot.text.len() + row.len() + separator > TEXT_BYTES {
                return Err("Terminal copy projection exceeds 1 MiB; live state retained".into());
            }
            snapshot.text.push_str(&row);
            if separator > 0 {
                snapshot.text.push('\n');
                scalars += 1;
            }
        }
        Ok(snapshot)
    }
    pub fn charge(&self) -> usize {
        2 * self.text.len() + self.rows.len() * size_of::<std::ops::Range<usize>>() + 64 * 1024
    }
    pub fn point(&self, session: &Session, point: Point) -> Scalar {
        let index = (point.line.0 + self.history as i32).max(0) as usize;
        let index = index.min(self.rows.len() - 1);
        let range = &self.rows[index];
        let grid = session.screen().grid();
        let line = Line(index as i32 - self.history as i32);
        let column = point.column.0.min(usize::from(session.geometry().columns));
        let before: usize = (0..column)
            .map(|column| {
                cell(&grid[Point::new(line, Column(column))])
                    .chars()
                    .count()
            })
            .sum();
        Scalar((range.start + before).min(range.end))
    }
    pub fn top(&self, viewport: Viewport, buffer: &Buffer) -> Result<usize, String> {
        let index = self
            .history
            .saturating_sub(viewport.offset.min(self.history));
        crate::coordinates::Coordinates::new(buffer)
            .map_err(|e| e.to_string())?
            .to_cell(Scalar(self.rows[index].start))
            .map(|c| c.row)
            .map_err(|e| e.to_string())
    }
}

fn row(session: &Session, line: Line) -> Result<String, String> {
    let mut text = String::new();
    for column in 0..usize::from(session.geometry().columns) {
        let incoming = cell(&session.screen().grid()[Point::new(line, Column(column))]);
        if text.len() + incoming.len() > TEXT_BYTES {
            return Err("Terminal copy row exceeds 1 MiB; live state retained".into());
        }
        text.push_str(&incoming);
    }
    Ok(text)
}

fn cell(cell: &Cell) -> String {
    if cell
        .flags
        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER)
    {
        return String::new();
    }
    if cell.flags.contains(Flags::HIDDEN) {
        return " ".repeat(if cell.flags.contains(Flags::WIDE_CHAR) {
            2
        } else {
            1
        });
    }
    std::iter::once(cell.c)
        .chain(cell.zerowidth().unwrap_or_default().iter().copied())
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
