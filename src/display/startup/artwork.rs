use crate::config::Rgb;
use std::ops::Range;
use std::sync::OnceLock;

pub(super) struct Run {
    pub columns: Range<u16>,
    pub foreground: Rgb,
    pub background: Rgb,
}
pub(super) struct Row {
    pub text: String,
    pub runs: Vec<Run>,
}

/// The bundled asset admits only RGB SGR and reset. It is parsed once into 1,300
/// native cells; escape bytes never reach row text or a terminal write path.
pub(super) fn get() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        parse(include_str!("../../session/startup/rev-logo.ans"))
            .expect("bundled startup artwork must satisfy its admitted 100x13 RGB shape")
    })
}
fn parse(asset: &str) -> Result<Vec<Row>, &'static str> {
    let mut foreground = Rgb(0, 0, 0);
    let mut background = Rgb(0, 0, 0);
    let mut rows = Vec::new();
    for line in asset.lines() {
        if rows.len() >= 13 {
            return Err("artwork row limit");
        }
        let mut remaining = line;
        let mut text = String::new();
        let mut runs: Vec<Run> = Vec::new();
        let mut column = 0;
        while !remaining.is_empty() {
            if let Some(sequence) = remaining.strip_prefix("\x1b[") {
                let (sgr, tail) = sequence.split_once('m').ok_or("unfinished color")?;
                let values: Result<Vec<u8>, _> = sgr.split(';').map(str::parse).collect();
                match values.map_err(|_| "color value")?.as_slice() {
                    [0] => {
                        foreground = Rgb(0, 0, 0);
                        background = Rgb(0, 0, 0);
                    }
                    [38, 2, r, g, b] => foreground = Rgb(*r, *g, *b),
                    [48, 2, r, g, b] => background = Rgb(*r, *g, *b),
                    _ => return Err("unsupported artwork control"),
                }
                remaining = tail;
            } else {
                let glyph = remaining.chars().next().ok_or("missing artwork cell")?;
                if glyph.is_control()
                    || unicode_width::UnicodeWidthChar::width(glyph) != Some(1)
                    || column >= 100
                {
                    return Err("invalid artwork cell");
                }
                text.push(glyph);
                if let Some(run) = runs.last_mut()
                    && run.foreground == foreground
                    && run.background == background
                {
                    run.columns.end += 1;
                } else {
                    runs.push(Run {
                        columns: column..column + 1,
                        foreground,
                        background,
                    });
                }
                column += 1;
                remaining = &remaining[glyph.len_utf8()..];
            }
        }
        if column != 100 {
            return Err("artwork width");
        }
        rows.push(Row { text, runs });
    }
    if rows.len() != 13 {
        return Err("artwork height");
    }
    Ok(rows)
}
