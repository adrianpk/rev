use std::io::{self, Write};

use alacritty_terminal::{
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

impl Frame {
    /// Project emulator cells, never child escape strings. Per-cell placement
    /// preserves the emulator's columns even for unusual combining sequences.
    pub(crate) fn terminal(&mut self, session: &Session, panel: Panel, label: &str) {
        let area = panel.content();
        let grid = session.screen().grid();
        let size = session.geometry();
        for row in 0..area.height() {
            let cells: Vec<_> = (0..area.width().min(size.columns))
                .filter(|_| row < size.rows)
                .map(|column| {
                    let mut cell =
                        grid[Point::new(Line(i32::from(row)), Column(usize::from(column)))].clone();
                    if cell.flags.contains(Flags::WIDE_CHAR) && column + 1 == area.width() {
                        cell.c = ' ';
                        cell.extra = None;
                        cell.flags.remove(Flags::WIDE_CHAR);
                    }
                    cell
                })
                .collect();
            let native = &mut self.rows[usize::from(area.row() + row)];
            native.text = projected(&cells);
            native.text.extend(std::iter::repeat_n(
                ' ',
                usize::from(area.width()) - cells.len(),
            ));
            native.band = Band::Document;
            self.terminal_rows.insert(native.row, cells);
        }
        if let Some(status) = panel.status() {
            let row = &mut self.rows[usize::from(status.row())];
            row.text = crate::coordinates::Glyphs::for_text(label)
                .and_then(|g| super::clip::line(g, 0, status.width()))
                .unwrap_or_else(|_| " ".repeat(usize::from(status.width())));
            row.band = Band::Status;
            row.tone = Tone::Status(panel.active());
            row.faces.clear();
        }
        if panel.active() {
            self.cursor = None;
            let cursor = grid.cursor.point;
            if session.screen().mode().contains(TermMode::SHOW_CURSOR) && cursor.line.0 >= 0 {
                self.cursor = panel.position(Position {
                    column: cursor.column.0 as u16,
                    row: cursor.line.0 as u16,
                });
            }
        }
    }
}

pub(super) fn write(
    writer: &mut impl Write,
    cells: &[Cell],
    palette: &Palette,
    width: u16,
) -> io::Result<()> {
    for (column, cell) in cells.iter().enumerate() {
        if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
            continue;
        }
        writer.queue(MoveToColumn(column as u16))?;
        attributes(writer, cell.flags)?;
        let fg = color(cell.fg, palette);
        let bg = color(cell.bg, palette);
        super::style::Colors::new(fg, bg).write(writer, &mut None)?;
        write_cell(writer, cell)?;
    }
    writer.queue(SetAttribute(Attribute::Reset))?;
    padding(writer, cells.len(), width, palette)?;
    super::style::reset(writer)
}

fn padding(writer: &mut impl Write, cells: usize, width: u16, palette: &Palette) -> io::Result<()> {
    if cells >= usize::from(width) {
        return Ok(());
    }
    super::style::Colors::new(palette.foreground, palette.background).write(writer, &mut None)?;
    writer.queue(MoveToColumn(cells as u16))?;
    for _ in cells..usize::from(width) {
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
