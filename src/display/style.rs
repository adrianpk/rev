use std::io::{self, Write};
use std::ops::Range;

use crossterm::{
    QueueableCommand,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
};

use crate::config::{Palette, Rgb};
use crate::panel::Composition;
use crate::window::Area;

use super::{Position, Row};

/// Message severity is supplied by the semantic owner, never inferred from text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Message {
    #[default]
    Normal,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Tone {
    #[default]
    Document,
    Habit(crate::org::agenda::HabitState, bool),
    Status(bool),
    Border(bool),
    Tree,
    Gutter(crate::gutter::Role),
    Message(Message),
}

pub(super) struct Face {
    pub columns: Range<u16>,
    pub tone: Tone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Colors {
    foreground: Rgb,
    background: Rgb,
}

impl Colors {
    fn new(foreground: Rgb, background: Rgb) -> Self {
        Self {
            foreground,
            background,
        }
    }

    pub(super) fn write(
        self,
        writer: &mut impl Write,
        current: &mut Option<Self>,
    ) -> io::Result<()> {
        if current.is_none_or(|prior| prior.background != self.background) {
            writer.queue(SetBackgroundColor(color(self.background)))?;
        }
        if current.is_none_or(|prior| prior.foreground != self.foreground) {
            writer.queue(SetForegroundColor(color(self.foreground)))?;
        }
        *current = Some(self);
        Ok(())
    }
}

fn color(Rgb(r, g, b): Rgb) -> Color {
    Color::Rgb { r, g, b }
}

impl Tone {
    fn colors(self, palette: &Palette) -> Colors {
        match self {
            Self::Status(true) => Colors::new(palette.status_text, palette.status),
            Self::Status(false) => Colors::new(palette.status_text, palette.status_inactive),
            Self::Border(true) => Colors::new(palette.border_active, palette.background),
            Self::Border(false) => Colors::new(palette.border, palette.background),
            Self::Tree => Colors::new(palette.foreground, palette.tree),
            Self::Gutter(role) => Colors::new(
                gutter_color(role, &palette.gutter),
                palette.gutter.background,
            ),
            Self::Message(message) => Colors::new(message.color(palette), palette.background),
            Self::Document => Colors::new(palette.foreground, palette.background),
            Self::Habit(state, future) => {
                Colors::new(palette.foreground, habit_color(state, future, &palette.org))
            }
        }
    }
}

fn gutter_color(role: crate::gutter::Role, palette: &crate::config::GutterPalette) -> Rgb {
    match role {
        crate::gutter::Role::Number => palette.foreground,
        crate::gutter::Role::CurrentLine => palette.current_line,
        crate::gutter::Role::Separator => palette.separator,
        crate::gutter::Role::Bookmark => Rgb(0xe0, 0xaf, 0x68),
    }
}

impl Message {
    fn color(self, palette: &Palette) -> Rgb {
        match self {
            Self::Normal => palette.message,
            Self::Warning => palette.warning,
            Self::Error => palette.error,
        }
    }
}

impl Row {
    pub(super) fn tone(&self, column: usize) -> Tone {
        self.faces
            .iter()
            .rev()
            .find(|face| covers(&face.columns, column))
            .map_or(self.tone, |face| face.tone)
    }

    pub(super) fn colors(&self, column: usize, palette: &Palette) -> Colors {
        let tone = self.tone(column);
        if self
            .selection
            .as_ref()
            .is_some_and(|range| covers(range, column))
        {
            return match tone {
                Tone::Tree => Colors::new(palette.tree_selected_text, palette.tree_selected),
                _ => Colors::new(palette.selection_text, palette.selection),
            };
        }
        if self
            .search
            .as_ref()
            .is_some_and(|range| covers(range, column))
        {
            return Colors::new(palette.search_text, palette.search);
        }
        let mut colors = tone.colors(palette);
        if let Some(style) = self.style(column) {
            colors.foreground = super::syntax::foreground(style, palette);
        }
        colors
    }
}

fn covers(range: &Range<u16>, column: usize) -> bool {
    (usize::from(range.start)..usize::from(range.end)).contains(&column)
}

pub(super) fn reset(writer: &mut impl Write) -> io::Result<()> {
    writer.queue(ResetColor).map(|_| ())
}

pub(super) fn focused(composition: &Composition, point: Position) -> bool {
    composition
        .panels()
        .iter()
        .map(|(_, panel)| panel)
        .chain(composition.tree().as_ref())
        .any(|panel| panel.active() && adjacent(panel.outer(), point))
}

fn adjacent(area: Area, point: Position) -> bool {
    let vertical = (area.row()..area.row() + area.height()).contains(&point.row);
    let horizontal = (area.column()..area.column() + area.width()).contains(&point.column);
    let beside = point.column.checked_add(1) == Some(area.column())
        || point.column == area.column() + area.width();
    let above =
        point.row.checked_add(1) == Some(area.row()) || point.row == area.row() + area.height();
    (vertical && beside) || (horizontal && above)
}

pub(super) fn append(faces: &mut Vec<Face>, column: u16, tone: Tone) {
    if let Some(last) = faces.last_mut()
        && last.tone == tone
        && last.columns.end == column
    {
        last.columns.end += 1;
    } else {
        faces.push(Face {
            columns: column..column + 1,
            tone,
        });
    }
}

fn habit_color(
    state: crate::org::agenda::HabitState,
    future: bool,
    p: &crate::config::OrgPalette,
) -> Rgb {
    use crate::org::agenda::HabitState::*;
    match (state, future) {
        (Clear, false) => p.habit_clear,
        (Clear, true) => p.habit_clear_future,
        (Ready, false) => p.habit_ready,
        (Ready, true) => p.habit_ready_future,
        (Alert, false) => p.habit_alert,
        (Alert, true) => p.habit_alert_future,
        (Overdue, false) => p.habit_overdue,
        (Overdue, true) => p.habit_overdue_future,
    }
}
