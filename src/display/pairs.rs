use crate::buffer::Buffer;
use crate::config::{PairHighlight, Palette, Rgb};
use crate::coordinates::{Coordinates, Scalar};
use crate::pairs::Delimiter;
use crate::view::{Projection, View};

use super::{Error, Paint, Row};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Foreground {
    Depth(usize),
    Match,
    Native,
    Plain,
}

/// A disposable cell overlay, never an independent source or qualification cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Mark {
    pub source: Scalar,
    pub active: bool,
    foreground: Foreground,
}

impl Mark {
    pub fn foreground(self, palette: &Palette, native: Rgb) -> Rgb {
        match self.foreground {
            Foreground::Depth(depth) => {
                palette.pairs.depth_colors[depth % palette.pairs.depth_colors.len()]
            }
            Foreground::Match => palette.pairs.match_text,
            Foreground::Native => native,
            Foreground::Plain => palette.foreground,
        }
    }
}

pub(super) fn active(buffer: &Buffer, view: &View) -> Option<Delimiter> {
    let facts = buffer.pair_presentation(view.pairs.ticket?)?;
    let entry = buffer.pair_entry(view.point.position);
    facts
        .spans
        .binary_search_by_key(&entry, |s| s.at)
        .ok()
        .map(|i| facts.spans[i])
}

pub(super) fn paint(
    buffer: &Buffer,
    view: &View,
    projection: Projection,
    row: &mut Row,
) -> Result<(), Error> {
    let style = view.pairs.style(buffer.pair_policy());
    if projection.source.row >= buffer.len_lines()
        || matches!(style, PairHighlight::Off | PairHighlight::Pulse)
    {
        return Ok(());
    }
    let Some(facts) = view.pairs.ticket.and_then(|t| buffer.pair_presentation(t)) else {
        return Ok(());
    };
    let coordinates = Coordinates::new(buffer)?;
    let selected = active(buffer, view);
    let right = projection
        .source
        .column
        .saturating_add(usize::from(projection.width));
    for glyph in coordinates.glyphs_from_column(projection.source)? {
        let glyph = glyph?;
        if glyph.columns.start >= right {
            break;
        }
        if !visible(&glyph.columns, projection.source.column..right) {
            continue;
        }
        let Ok(index) = facts
            .spans
            .binary_search_by_key(&glyph.source.start, |s| s.at)
        else {
            continue;
        };
        let endpoint = facts.spans[index];
        let active = selected.is_some_and(|s| endpoint.at == s.at || endpoint.at == s.mate);
        append(row, (endpoint, style, active), projection, glyph.columns);
    }
    Ok(())
}

fn append(
    row: &mut Row,
    pair: (Delimiter, PairHighlight, bool),
    projection: Projection,
    columns: std::ops::Range<usize>,
) {
    let (endpoint, style, active) = pair;
    if endpoint.kind.quote() && !active || style != PairHighlight::Depth && !active {
        return;
    }
    let start = projection
        .screen_column(columns.start)
        .expect("visible endpoint");
    let end = projection
        .screen_column(columns.end)
        .expect("visible endpoint");
    let native = row.style(start);
    let mark = Mark {
        source: endpoint.at,
        active,
        foreground: foreground(endpoint, style, active, native.is_some()),
    };
    let mut paint = Paint::pair(native, mark);
    paint.columns = u16::try_from(start).expect("native row width")
        ..u16::try_from(end).expect("native row width");
    row.syntax.push(paint);
}

fn foreground(endpoint: Delimiter, style: PairHighlight, active: bool, native: bool) -> Foreground {
    if endpoint.kind.quote() {
        return if native {
            Foreground::Native
        } else {
            Foreground::Plain
        };
    }
    if style == PairHighlight::Depth {
        Foreground::Depth(endpoint.depth)
    } else if active {
        Foreground::Match
    } else {
        Foreground::Native
    }
}

impl Row {
    pub(super) fn pair_underline(&self, column: usize) -> bool {
        let selected = self
            .selection
            .as_ref()
            .is_some_and(|r| (usize::from(r.start)..usize::from(r.end)).contains(&column));
        !selected
            && self
                .style(column)
                .is_some_and(|p| p.delimiter.is_some_and(|m| m.active))
    }
}

impl super::Frame {
    pub(super) fn pair_hint(&mut self, buffer: &Buffer, view: &View) -> Result<(), Error> {
        if matches!(
            view.pairs.style(buffer.pair_policy()),
            PairHighlight::Off | PairHighlight::Pulse
        ) {
            return Ok(());
        }
        let Some(endpoint) = active(buffer, view) else {
            return Ok(());
        };
        let Some(echo) = self.rows.last() else {
            return Ok(());
        };
        if echo.band != super::Band::Echo || !echo.text.trim().is_empty() {
            return Ok(());
        }
        let Ok(cell) = Coordinates::new(buffer)?.to_cell(endpoint.mate) else {
            return Ok(());
        };
        if self.source_rows.last().is_some_and(|source| {
            source.rows.iter().any(|p| {
                p.source.row == cell.row
                    && p.screen_column(cell.column)
                        .is_some_and(|column| column < usize::from(p.margin + p.width))
            })
        }) {
            return Ok(());
        }
        let message = format!(
            "Matching delimiter: line {}, column {}",
            cell.row + 1,
            cell.column + 1
        );
        let text = super::clip::line(
            crate::coordinates::Glyphs::for_text(&message)?,
            0,
            self.size.width,
        )?;
        self.rows.last_mut().expect("admitted echo row").text = text;
        Ok(())
    }
}

fn visible(glyph: &std::ops::Range<usize>, row: std::ops::Range<usize>) -> bool {
    glyph.start >= row.start && glyph.end <= row.end
}
