mod pulse;

use crate::config::{PairHighlight, Pairs};

/// Runtime overrides and remembered style belong to one view. The buffer only
/// supplies resolved startup defaults, including after a language change.
#[derive(Clone, Copy, Debug, Default)]
pub struct PairPresentation {
    pub(crate) pulse: Option<pulse::Pulse>,
    pub(crate) generation: u64,
    pub(crate) ticket: Option<crate::pairs::PresentationTicket>,
    pub(crate) pending: bool,
    style: Option<PairHighlight>,
    last_non_off: Option<PairHighlight>,
}

// Qualification tickets are disposable observations, like the native wrap
// cache. They cannot change the semantic View identity captured by a prompt.
impl PartialEq for PairPresentation {
    fn eq(&self, other: &Self) -> bool {
        self.style == other.style && self.last_non_off == other.last_non_off
    }
}
impl Eq for PairPresentation {}

impl PairPresentation {
    pub(crate) fn retire(&mut self) {
        self.ticket = None;
        self.pending = false;
    }

    pub fn style(self, defaults: Pairs) -> PairHighlight {
        self.style.unwrap_or(defaults.highlight)
    }

    pub fn last_non_off(self, defaults: Pairs) -> PairHighlight {
        self.last_non_off.unwrap_or(match defaults.highlight {
            PairHighlight::Off => PairHighlight::Depth,
            style => style,
        })
    }

    pub fn choose(&mut self, style: PairHighlight, defaults: Pairs) {
        self.cancel_pulse();
        let remembered = if style == PairHighlight::Off {
            self.style(defaults)
        } else {
            style
        };
        if remembered != PairHighlight::Off {
            self.last_non_off = Some(remembered);
        }
        self.style = Some(style);
    }

    pub fn toggle(&mut self, defaults: Pairs) {
        let style = self.style(defaults);
        if style == PairHighlight::Off {
            self.choose(self.last_non_off(defaults), defaults);
        } else {
            self.last_non_off = Some(style);
            self.cancel_pulse();
            self.style = Some(PairHighlight::Off);
        }
    }
}

impl super::View {
    pub(crate) fn pair_visible_span(
        &self,
        buffer: &crate::buffer::Buffer,
        area: crate::window::Area,
    ) -> Result<crate::buffer::Span, super::Error> {
        let text = self.gutter_geometry(buffer, area)?.text();
        if buffer.soft_wrap() {
            return self.wrapped_pair_span(buffer, text);
        }
        let mut range = None;
        for row in 0..text.height() {
            let Some(source) = self.source_row(buffer, usize::from(row)) else {
                continue;
            };
            let projection = super::Projection::horizontal(
                buffer,
                source,
                self.left,
                text.width(),
                self.compact_headings(),
            );
            if projection.source.row < buffer.len_lines() {
                include(buffer, projection, &mut range)?;
            }
        }
        let (start, end) = range.unwrap_or((self.point.position.0, self.point.position.0));
        Ok(buffer.span(start..end)?)
    }

    // Wrapped fragments already carry qualified scalar bounds. Rewalking their
    // glyphs spends the shared layout budget again and can evict a tall canvas.
    fn wrapped_pair_span(
        &self,
        buffer: &crate::buffer::Buffer,
        area: crate::window::Area,
    ) -> Result<crate::buffer::Span, super::Error> {
        let fragments = self.wrapped_rows(buffer, area.width(), area.height())?;
        let range = fragments.first().zip(fragments.last()).map_or(
            self.point.position.0..self.point.position.0,
            |(first, last)| first.source.start.0..last.source.end.0,
        );
        Ok(buffer.span(range)?)
    }
}

fn include(
    buffer: &crate::buffer::Buffer,
    projection: super::Projection,
    range: &mut Option<(usize, usize)>,
) -> Result<(), super::Error> {
    let coordinates = crate::coordinates::Coordinates::new(buffer)?;
    let right = projection
        .source
        .column
        .saturating_add(usize::from(projection.width));
    for glyph in coordinates.glyphs_from_column(projection.source)? {
        let glyph = glyph?;
        if glyph.columns.start >= right {
            break;
        }
        if glyph.columns.start >= projection.source.column && glyph.columns.end <= right {
            let previous = range.unwrap_or((glyph.source.start.0, glyph.source.end.0));
            *range = Some((
                previous.0.min(glyph.source.start.0),
                previous.1.max(glyph.source.end.0),
            ));
        }
    }
    Ok(())
}
