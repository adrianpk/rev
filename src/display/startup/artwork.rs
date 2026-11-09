use crate::config::Rgb;
use std::ops::Range;
use std::sync::OnceLock;

pub(super) const WIDTH: u16 = 58;
pub(super) const HEIGHT: u16 = 8;

pub(super) struct Run {
    pub columns: Range<u16>,
    pub foreground: Rgb,
    pub background: Rgb,
}
pub(super) struct Row {
    pub text: String,
    pub runs: Vec<Run>,
}

/// The bundled asset admits only RGB SGR and reset. It is parsed once into 464
/// native cells; escape bytes never reach row text or a terminal write path.
pub(super) fn get() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        parse(include_str!("../../session/startup/rev-logo-small.ans"))
            .expect("bundled startup artwork must satisfy its admitted 58x8 RGB shape")
    })
}
#[derive(Clone, Copy)]
struct Ink {
    foreground: Rgb,
    background: Rgb,
}

impl Ink {
    fn control(&mut self, sequence: &str) -> Result<(), &'static str> {
        let values: Result<Vec<u8>, _> = sequence.split(';').map(str::parse).collect();
        match values.map_err(|_| "color value")?.as_slice() {
            [0] => {
                self.foreground = Rgb(0, 0, 0);
                self.background = Rgb(0, 0, 0);
            }
            [38, 2, r, g, b] => self.foreground = Rgb(*r, *g, *b),
            [48, 2, r, g, b] => self.background = Rgb(*r, *g, *b),
            _ => return Err("unsupported artwork control"),
        }
        Ok(())
    }
}

fn parse(asset: &str) -> Result<Vec<Row>, &'static str> {
    let mut ink = Ink {
        foreground: Rgb(0, 0, 0),
        background: Rgb(0, 0, 0),
    };
    let mut rows = Vec::new();
    for line in asset.lines() {
        if rows.len() >= usize::from(HEIGHT) {
            return Err("artwork row limit");
        }
        rows.push(parse_row(line, &mut ink)?);
    }
    if rows.len() != usize::from(HEIGHT) {
        return Err("artwork height");
    }
    Ok(rows)
}

fn parse_row(mut remaining: &str, ink: &mut Ink) -> Result<Row, &'static str> {
    let mut row = Row {
        text: String::new(),
        runs: Vec::new(),
    };
    let mut column = 0;
    while !remaining.is_empty() {
        if let Some(sequence) = remaining.strip_prefix("\x1b[") {
            let (sgr, tail) = sequence.split_once('m').ok_or("unfinished color")?;
            ink.control(sgr)?;
            remaining = tail;
        } else {
            let glyph = remaining.chars().next().ok_or("missing artwork cell")?;
            row.cell(column, glyph, *ink)?;
            column += 1;
            remaining = &remaining[glyph.len_utf8()..];
        }
    }
    if column != WIDTH {
        return Err("artwork width");
    }
    Ok(row)
}

impl Row {
    fn cell(&mut self, column: u16, glyph: char, ink: Ink) -> Result<(), &'static str> {
        if glyph.is_control()
            || unicode_width::UnicodeWidthChar::width(glyph) != Some(1)
            || column >= WIDTH
        {
            return Err("invalid artwork cell");
        }
        self.text.push(glyph);
        if let Some(run) = self.runs.last_mut()
            && run.foreground == ink.foreground
            && run.background == ink.background
        {
            run.columns.end += 1;
        } else {
            self.runs.push(Run {
                columns: column..column + 1,
                foreground: ink.foreground,
                background: ink.background,
            });
        }
        Ok(())
    }
}
