use crate::buffer::Buffer;
use crate::coordinates::Cell;

/// One viewport row's source-cell window and visible virtual prefix. Painting,
/// highlights, cursor anchors and horizontal hits share this translation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Projection {
    pub source: Cell,
    pub margin: u16,
    pub width: u16,
    concealed: usize,
}

impl Projection {
    pub(crate) fn horizontal(
        buffer: &Buffer,
        row: usize,
        left: usize,
        width: u16,
        compact: bool,
    ) -> Self {
        let margins = Margins::new(buffer, row, width, compact);
        let margin = margins.at(0);
        let padding = margin.saturating_sub(left).min(usize::from(width)) as u16;
        Self {
            source: Cell {
                row,
                column: left
                    .saturating_sub(margin)
                    .saturating_add(margins.concealed),
            },
            margin: padding,
            width: width - padding,
            concealed: margins.concealed,
        }
    }

    pub(crate) fn wrapped(fragment: &super::Fragment, width: u16) -> Self {
        Self {
            source: Cell {
                row: fragment.line,
                column: fragment.columns.start.max(fragment.margins.concealed),
            },
            margin: fragment.margin,
            width: width.saturating_sub(fragment.margin),
            concealed: fragment.margins.concealed,
        }
    }

    pub(crate) fn screen_column(self, source: usize) -> Option<usize> {
        source
            .max(self.concealed)
            .checked_sub(self.source.column)?
            .checked_add(usize::from(self.margin))
    }

    pub(crate) fn source_column(self, screen: usize) -> usize {
        self.source
            .column
            .saturating_add(screen.saturating_sub(usize::from(self.margin)))
    }

    pub(crate) fn shift(self, range: std::ops::Range<u16>) -> std::ops::Range<u16> {
        range.start + self.margin..range.end + self.margin
    }
}

/// Virtual columns never change the logical columns used for source tabs.
/// Clamp before allocating text, leaving room for a two-cell grapheme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Margins {
    first: usize,
    continuation: usize,
    enabled: bool,
    observed: bool,
    pub(super) compact: bool,
    pub(super) concealed: usize,
}

impl Margins {
    pub(super) fn new(buffer: &Buffer, row: usize, width: u16, compact: bool) -> Self {
        if !buffer.is_org() {
            return Self::default();
        }
        let enabled = buffer.org_indentation();
        if !enabled && !compact {
            return Self::default();
        }
        let Some(observation) = buffer.org_line_indentation(row) else {
            return Self {
                enabled,
                compact,
                ..Self::default()
            };
        };
        let cap = usize::from(width.saturating_sub(2));
        let body = observation.level.saturating_mul(2).min(cap);
        Self {
            first: if !enabled {
                0
            } else if observation.heading {
                observation.level.saturating_sub(1).min(cap)
            } else {
                body
            },
            continuation: if enabled { body } else { 0 },
            enabled,
            observed: true,
            compact,
            concealed: if compact && !enabled && observation.heading {
                observation.level.saturating_sub(1)
            } else {
                0
            },
        }
    }

    pub(super) fn at(self, source_column: usize) -> usize {
        if source_column == 0 {
            self.first
        } else {
            self.continuation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{
        Buffers, Mode,
        edit::{History, Replacement},
    };
    use crate::coordinates::{Coordinates, Scalar};
    use crate::org::{Cancellation, ColorScan, Coverage, Limits};
    use crate::view::{View, wrap::Line};

    fn observe(buffer: &mut Buffer, limits: Limits) {
        let coloring = ColorScan::new(buffer, limits)
            .unwrap()
            .run(&Cancellation::default());
        buffer.install_org_coloring(coloring).unwrap();
    }

    fn create(buffers: &mut Buffers, source: &str) -> crate::buffer::BufferId {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.select_mode(Mode::Org).unwrap();
        buffer.org_indent = true;
        observe(buffer, Limits::default());
        id
    }

    // Native ownership follows the latest heading, including blanks and a
    // shallower return. Closed literal blocks cannot introduce outline levels.
    #[test]
    fn structural_margins() {
        let mut buffers = Buffers::new().unwrap();
        for (name, source, margins) in [
            (
                "outline",
                "preamble\n* Parent\nbody\n*** Child\n\nchild body\n** Sibling\nsibling body\n* Return\nbody",
                vec![0, 0, 2, 2, 6, 6, 1, 4, 0, 2],
            ),
            (
                "source block",
                "** Parent\n#+begin_src text\n* literal\n#+end_src\nbody",
                vec![1, 4, 4, 4, 4],
            ),
            (
                "example block",
                "** Parent\n#+begin_example\n*** literal\n#+end_example\nbody",
                vec![1, 4, 4, 4, 4],
            ),
            (
                "nested delimiters",
                "** Parent\n#+begin_src text\n#+begin_example\n* literal\n#+end_example\n#+end_src\nbody",
                vec![1, 4, 4, 4, 4, 4, 4],
            ),
            (
                "source layout",
                "* Parent\n| one | two |\n  - item\n:PROPERTIES:\n:NAME: value\n:END:\n  code",
                vec![0, 2, 2, 2, 2, 2, 2],
            ),
        ] {
            let id = create(&mut buffers, source);
            let b = buffers.get(id).unwrap();
            let stamp = b.stamp();
            let state = b.state();
            let view = View::new(b);
            let rows = view.wrapped_rows(b, 80, 100).unwrap();
            assert_eq!(
                rows.iter().map(|f| f.margin).collect::<Vec<_>>(),
                margins,
                "{name}"
            );
            for f in rows {
                let source = b
                    .slice(&b.span(f.source.start.0..f.source.end.0).unwrap())
                    .unwrap()
                    .to_string();
                assert!(
                    f.text
                        .starts_with(&format!("{}{source}", " ".repeat(usize::from(f.margin)))),
                    "{name}"
                );
            }
            assert_eq!(
                b.slice(&b.span(0..b.len_chars()).unwrap())
                    .unwrap()
                    .to_string(),
                source
            );
            assert_eq!(b.stamp(), stamp);
            assert_eq!(b.state(), state);
            assert!(b.history().is_empty());
            assert!(!b.is_modified());
        }
    }

    // Every fully painted grapheme, including each source star, keeps its real
    // scalar. Virtual cells resolve to the fragment start without adding text.
    #[test]
    fn fragment_round_trips() {
        let mut buffers = Buffers::new().unwrap();
        let id = create(
            &mut buffers,
            "** Heading words\na\t界e\u{301}👩‍💻 next words\n",
        );
        let b = buffers.get(id).unwrap();
        let view = View::new(b);
        let coordinates = Coordinates::new(b).unwrap();
        for width in [6, 9, 16, 80] {
            let rows = view.wrapped_rows(b, width, 100).unwrap();
            for f in rows {
                assert_eq!(
                    unicode_width::UnicodeWidthStr::width(f.text.as_str()),
                    usize::from(width)
                );
                for column in f.columns.clone() {
                    let screen = f.screen_column(column).unwrap();
                    assert_eq!(f.source_column(screen), column);
                }
                for column in 0..f.margin {
                    assert_eq!(f.hit(b, width, column).unwrap(), Some(f.source.start));
                }
                for glyph in coordinates.glyphs(f.line).unwrap() {
                    let glyph = glyph.unwrap();
                    if glyph.source.start < f.source.start || glyph.source.end > f.source.end {
                        continue;
                    }
                    let start = f.screen_column(glyph.columns.start).unwrap();
                    let end = f.screen_column(glyph.columns.end).unwrap();
                    if end > usize::from(width) {
                        continue;
                    }
                    for column in start..end {
                        assert_eq!(
                            f.hit(b, width, column as u16).unwrap(),
                            Some(glyph.source.start),
                            "width={width}, {f:?}"
                        );
                    }
                }
                let actual = Line::resume(
                    b,
                    f.line,
                    width,
                    f.source.start,
                    f.columns.start,
                    f.margins.compact,
                )
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
                assert_eq!(actual, f, "restart at width {width}");
            }
        }
    }

    // Heading continuations align with the title; tabs remain expanded at their
    // source columns, independently of first-row and continuation margins.
    #[test]
    fn continuation_columns() {
        let mut buffers = Buffers::new().unwrap();
        for (name, source, expected) in [
            (
                "heading",
                "** One two three",
                vec![" ** One   ", "    two   ", "    three "],
            ),
            (
                "body",
                "** H\na\tb cdef",
                vec![" ** H     ", "    a   b ", "    cdef  "],
            ),
        ] {
            let id = create(&mut buffers, source);
            let b = buffers.get(id).unwrap();
            let actual = View::new(b).wrapped_rows(b, 10, 20).unwrap();
            assert_eq!(
                actual.iter().map(|f| f.text.as_str()).collect::<Vec<_>>(),
                expected,
                "{name}"
            );
        }
    }

    // Missing, incomplete and old structural evidence must retire cached
    // margins. Re-observation, policy changes, edits and undo rebuild geometry.
    #[test]
    fn evidence_invalidation() {
        let mut buffers = Buffers::new().unwrap();
        let id = create(&mut buffers, "** H\nbody");
        let b = buffers.get_mut(id).unwrap();
        let mut view = View::new(b);
        let margins = |view: &View, b: &Buffer| {
            view.wrapped_rows(b, 20, 10)
                .unwrap()
                .iter()
                .map(|f| f.margin)
                .collect::<Vec<_>>()
        };
        assert_eq!(margins(&view, b), [1, 4]);
        let before_policy = view.wrapped_rows(b, 20, 10).unwrap().pop().unwrap();
        b.org_indent = false;
        assert!(before_policy.hit(b, 20, 0).is_err());
        assert_eq!(margins(&view, b), [0, 0]);
        b.org_indent = true;
        observe(b, Limits::new(2, 100, 20).unwrap());
        assert_eq!(margins(&view, b), [0, 0]);
        let incomplete = view.wrapped_rows(b, 20, 10).unwrap().pop().unwrap();
        observe(b, Limits::default());
        assert!(incomplete.hit(b, 20, 0).is_err());
        assert_eq!(margins(&view, b), [1, 4]);
        assert!(
            view.wrapped_rows(b, 20, 10).unwrap()[0]
                .hit(b, 19, 0)
                .is_err()
        );
        let old = view.wrapped_rows(b, 20, 10).unwrap().pop().unwrap();
        let mut points = [view.point];
        let edit = Replacement {
            source: b.span(0..1).unwrap(),
            text: String::new(),
        };
        let prepared = b.prepare(&[edit], &points, None).unwrap();
        b.publish(prepared, &mut points).unwrap();
        view.point = points[0];
        assert!(old.hit(b, 20, 0).is_err());
        assert_eq!(margins(&view, b), [0, 0]);
        observe(b, Limits::default());
        assert_eq!(margins(&view, b), [0, 2]);
        b.navigate(History::Undo, &mut points, None).unwrap();
        view.point = points[0];
        assert_eq!(margins(&view, b), [0, 0]);
        observe(b, Limits::default());
        assert_eq!(margins(&view, b), [1, 4]);
        b.select_mode(Mode::Text).unwrap();
        assert_eq!(margins(&view, b), [0, 0]);
    }

    // Horizontal clipping uses the same capped outer margin. Hits on a partly
    // clipped wide glyph stay unavailable, and tab expansion remains literal.
    #[test]
    fn horizontal_hits() {
        use crate::display::Position;
        use crate::window::Area;
        let mut buffers = Buffers::new().unwrap();
        let id = create(&mut buffers, "** H\n界ab");
        buffers.get_mut(id).unwrap().toggle_soft_wrap();
        let b = buffers.get(id).unwrap();
        let mut view = View::new(b);
        view.gutter.apply(crate::gutter::Change::Visible(false));
        view.bookmark_markers = false;
        let area = Area::new(0, 0, 10, 2).unwrap();
        for (left, column, expected) in [
            (0, 0, Some(5)),
            (0, 3, Some(5)),
            (0, 4, Some(5)),
            (0, 5, Some(5)),
            (0, 6, Some(6)),
            (5, 0, None),
            (5, 1, Some(6)),
            (6, 0, Some(6)),
        ] {
            view.left = left;
            assert_eq!(
                view.hit(b, area, Position { row: 1, column })
                    .unwrap()
                    .map(|p| p.position),
                expected.map(Scalar),
                "{left}/{column}"
            );
        }
    }

    // Deep headings and long tokens make source progress at every pane width;
    // the bounded prefix is allocated only after the width cap is applied.
    #[test]
    fn narrow_progress() {
        let mut buffers = Buffers::new().unwrap();
        let source = format!("{} H\n{}", "*".repeat(200), "界a".repeat(100));
        let id = create(&mut buffers, &source);
        let b = buffers.get(id).unwrap();
        for width in [1, 2, 3, 8, 20] {
            let mut last = Scalar(0);
            let rows = View::new(b).wrapped_rows(b, width, 1000).unwrap();
            assert!(rows.len() < b.len_chars() + b.len_lines() + 1);
            for f in rows {
                assert!(f.margin <= width.saturating_sub(2));
                assert!(f.source.end >= last);
                assert!(!f.source.is_empty() || f.end_of_line);
                assert_eq!(
                    unicode_width::UnicodeWidthStr::width(f.text.as_str()),
                    usize::from(width)
                );
                last = f.source.end;
            }
            assert_eq!(last, Scalar(b.len_chars()));
        }
        assert_eq!(Coverage::Complete, b.org_highlights().unwrap().coverage());
    }
}
