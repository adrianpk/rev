use std::io::{self, Write};

use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
    term::{
        TermMode,
        cell::{Cell, Flags},
    },
    vte::ansi::{Color, NamedColor},
};
use crossterm::{
    QueueableCommand,
    cursor::MoveToColumn,
    style::{Attribute, SetAttribute},
};

use super::{Band, Frame, Position, Tone};
use crate::{
    config::{Palette, Rgb},
    panel::Panel,
    term::Session,
};

pub(super) struct Run {
    column: u16,
    width: u16,
    cells: Vec<Cell>,
}
impl Run {
    pub(super) fn full_width(&self, width: u16) -> bool {
        self.column == 0 && self.width == width
    }
}

impl Frame {
    pub(crate) fn terminal_copy(
        &mut self,
        buffer: &crate::buffer::Buffer,
        view: &crate::view::View,
        panel: Panel,
        label: &str,
    ) -> Result<(), crate::view::Error> {
        self.panel(buffer, view, panel)?;
        if panel.active() {
            self.cursor_style = None;
        }
        if let Some(status) = panel.status() {
            let text = crate::coordinates::Glyphs::for_text(label)
                .and_then(|g| super::clip::line(g, 0, status.width()))?;
            self.terminal_blit(
                status,
                0,
                text,
                (Band::Status, Tone::Status(panel.active())),
            );
        }
        Ok(())
    }
    /// Project emulator cells, never child escape strings. Per-cell placement
    /// preserves the emulator's columns even for unusual combining sequences.
    pub(crate) fn terminal(
        &mut self,
        session: &Session,
        panel: Panel,
        label: &str,
        viewport: &crate::term::Viewport,
    ) {
        let area = panel.content();
        let grid = session.screen().grid();
        let size = session.geometry();
        let offset = viewport.offset.min(grid.history_size()) as i32;
        for row in 0..area.height() {
            let cells: Vec<_> = (0..area.width().min(size.columns))
                .filter(|_| row < size.rows)
                .map(|column| {
                    let mut cell = grid
                        [Point::new(Line(i32::from(row) - offset), Column(usize::from(column)))]
                    .clone();
                    cell.fg = resolved(cell.fg, session.screen().colors());
                    cell.bg = resolved(cell.bg, session.screen().colors());
                    select_cell(
                        &mut cell,
                        Point::new(Line(i32::from(row) - offset), Column(usize::from(column))),
                        viewport.selection,
                    );
                    if cell.flags.contains(Flags::WIDE_CHAR) && column + 1 == area.width() {
                        cell.c = ' ';
                        cell.extra = None;
                        cell.flags.remove(Flags::WIDE_CHAR);
                    }
                    cell
                })
                .collect();
            let mut text = projected(&cells);
            text.extend(std::iter::repeat_n(
                ' ',
                usize::from(area.width()) - cells.len(),
            ));
            self.terminal_blit(area, row, text, (Band::Document, Tone::Document));
            self.terminal_rows
                .entry(area.row() + row)
                .or_default()
                .push(Run {
                    column: area.column(),
                    width: area.width(),
                    cells,
                });
        }
        if let Some(status) = panel.status() {
            let text = crate::coordinates::Glyphs::for_text(label)
                .and_then(|g| super::clip::line(g, 0, status.width()))
                .unwrap_or_else(|_| " ".repeat(usize::from(status.width())));
            self.terminal_blit(
                status,
                0,
                text,
                (Band::Status, Tone::Status(panel.active())),
            );
        }
        if panel.active() {
            self.cursor_style = cursor_style(session.screen().cursor_style());
            self.cursor = None;
            let cursor = grid.cursor.point;
            if offset == 0
                && session.screen().mode().contains(TermMode::SHOW_CURSOR)
                && cursor.line.0 >= 0
            {
                self.cursor = panel.position(Position {
                    column: cursor.column.0 as u16,
                    row: cursor.line.0 as u16,
                });
            }
        }
    }

    fn terminal_blit(
        &mut self,
        area: crate::window::Area,
        row: u16,
        text: String,
        style: (Band, Tone),
    ) {
        self.blit(
            area,
            super::Row {
                row,
                text,
                band: style.0,
                tone: style.1,
                faces: Vec::new(),
                syntax: Vec::new(),
                search: None,
                selection: None,
            },
        )
        .expect("admitted terminal projection");
    }
}

fn select_cell(cell: &mut Cell, point: Point, selection: Option<crate::term::Selection>) {
    if selection.is_some_and(|s| s.contains(point)) {
        cell.flags.toggle(Flags::INVERSE);
    }
}

fn resolved(color: Color, colors: &alacritty_terminal::term::color::Colors) -> Color {
    let index = match color {
        Color::Indexed(index) => usize::from(index),
        Color::Named(name) => name as usize,
        Color::Spec(_) => return color,
    };
    colors[index].map(Color::Spec).unwrap_or(color)
}

fn cursor_style(
    style: alacritty_terminal::vte::ansi::CursorStyle,
) -> Option<crossterm::cursor::SetCursorStyle> {
    use alacritty_terminal::vte::ansi::CursorShape;
    use crossterm::cursor::SetCursorStyle;
    Some(match (style.shape, style.blinking) {
        (CursorShape::Block | CursorShape::HollowBlock, true) => SetCursorStyle::BlinkingBlock,
        (CursorShape::Block | CursorShape::HollowBlock, false) => SetCursorStyle::SteadyBlock,
        (CursorShape::Beam, true) => SetCursorStyle::BlinkingBar,
        (CursorShape::Beam, false) => SetCursorStyle::SteadyBar,
        (CursorShape::Underline, true) => SetCursorStyle::BlinkingUnderScore,
        (CursorShape::Underline, false) => SetCursorStyle::SteadyUnderScore,
        (CursorShape::Hidden, _) => return None,
    })
}

pub(super) fn write(writer: &mut impl Write, run: &Run, palette: &Palette) -> io::Result<()> {
    for (column, cell) in run.cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }
        writer.queue(MoveToColumn(run.column + column as u16))?;
        attributes(writer, cell.flags)?;
        let fg = color(cell.fg, palette);
        let bg = color(cell.bg, palette);
        super::style::Colors::new(fg, bg).write(writer, &mut None)?;
        write_cell(writer, cell)?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    padding(writer, run, palette)?;
    super::style::reset(writer)
}

fn padding(writer: &mut impl Write, run: &Run, palette: &Palette) -> io::Result<()> {
    let cells = run.cells.len();
    if cells >= usize::from(run.width) {
        return Ok(());
    }
    super::style::Colors::new(palette.foreground, palette.background).write(writer, &mut None)?;
    writer.queue(MoveToColumn(run.column + cells as u16))?;
    for _ in cells..usize::from(run.width) {
        writer.write_all(b" ")?;
    }
    Ok(())
}

fn projected(cells: &[Cell]) -> String {
    let mut text = String::new();
    for cell in cells
        .iter()
        .filter(|c| !c.flags.contains(Flags::WIDE_CHAR_SPACER))
    {
        if cell.flags.contains(Flags::HIDDEN) {
            text.push(' ');
            if cell.flags.contains(Flags::WIDE_CHAR) {
                text.push(' ');
            }
        } else {
            text.push(printable(cell.c));
            text.extend(
                cell.zerowidth()
                    .unwrap_or_default()
                    .iter()
                    .map(|c| printable(*c)),
            );
        }
    }
    text
}

fn write_cell(writer: &mut impl Write, cell: &Cell) -> io::Result<()> {
    if cell.flags.contains(Flags::HIDDEN) {
        return writer.write_all(if cell.flags.contains(Flags::WIDE_CHAR) {
            b"  "
        } else {
            b" "
        });
    }
    write!(writer, "{}", printable(cell.c))?;
    for c in cell.zerowidth().unwrap_or_default() {
        write!(writer, "{}", printable(*c))?;
    }
    Ok(())
}

fn printable(c: char) -> char {
    if c.is_control() { ' ' } else { c }
}

fn attributes(writer: &mut impl Write, flags: Flags) -> io::Result<()> {
    writer.queue(SetAttribute(Attribute::Reset))?;
    for (flag, attribute) in [
        (Flags::BOLD, Attribute::Bold),
        (Flags::DIM, Attribute::Dim),
        (Flags::ITALIC, Attribute::Italic),
        (Flags::ALL_UNDERLINES, Attribute::Underlined),
        (Flags::STRIKEOUT, Attribute::CrossedOut),
        (Flags::INVERSE, Attribute::Reverse),
    ] {
        if flags.intersects(flag) {
            writer.queue(SetAttribute(attribute))?;
        }
    }
    Ok(())
}

fn color(color: Color, palette: &Palette) -> Rgb {
    match color {
        Color::Spec(rgb) => Rgb(rgb.r, rgb.g, rgb.b),
        Color::Indexed(index) => indexed(index),
        Color::Named(NamedColor::Background) => palette.background,
        Color::Named(NamedColor::DimForeground) => dim(palette.foreground),
        Color::Named(name) if (name as usize) < 16 => indexed(name as u8),
        Color::Named(name) if (259..267).contains(&(name as usize)) => dim(indexed(
            (name as usize - NamedColor::DimBlack as usize) as u8,
        )),
        Color::Named(_) => palette.foreground,
    }
}

fn dim(Rgb(r, g, b): Rgb) -> Rgb {
    Rgb(r / 2, g / 2, b / 2)
}

fn indexed(index: u8) -> Rgb {
    const ANSI: [Rgb; 16] = [
        Rgb(0, 0, 0),
        Rgb(205, 0, 0),
        Rgb(0, 205, 0),
        Rgb(205, 205, 0),
        Rgb(0, 0, 238),
        Rgb(205, 0, 205),
        Rgb(0, 205, 205),
        Rgb(229, 229, 229),
        Rgb(127, 127, 127),
        Rgb(255, 0, 0),
        Rgb(0, 255, 0),
        Rgb(255, 255, 0),
        Rgb(92, 92, 255),
        Rgb(255, 0, 255),
        Rgb(0, 255, 255),
        Rgb(255, 255, 255),
    ];
    if index < 16 {
        return ANSI[usize::from(index)];
    }
    if index >= 232 {
        let gray = 8 + (index - 232) * 10;
        return Rgb(gray, gray, gray);
    }
    let value = index - 16;
    let component = |n| if n == 0 { 0 } else { 55 + n * 40 };
    Rgb(
        component(value / 36),
        component(value / 6 % 6),
        component(value % 6),
    )
}
