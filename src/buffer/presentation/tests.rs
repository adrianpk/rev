use super::*;
use crate::buffer::{Mode, edit::Replacement};
use crate::org::{Cancellation, ColorScan, Limits};
use std::ops::Range;

pub(super) fn observed(source: &str) -> Buffer {
    let mut buffer = Buffer::from_rope(Rope::from_str(source)).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    analyze(&mut buffer);
    buffer
}

pub(super) fn analyze(buffer: &mut Buffer) {
    let coloring = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    buffer.install_org_coloring(coloring).unwrap();
}

fn replacement(buffer: &Buffer, range: Range<usize>, text: &str) -> Replacement {
    Replacement {
        source: buffer.span(range).unwrap(),
        text: text.into(),
    }
}

pub(super) fn edit(buffer: &mut Buffer, range: Range<usize>, text: &str) {
    let edit = replacement(buffer, range, text);
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}

fn mapped_rows(buffer: &Buffer, base: &Rope) {
    let mapping = buffer.presentation.as_ref().unwrap();
    assert_eq!(mapping.current, buffer.stamp());
    assert_eq!(mapping.bytes, buffer.len_bytes());
    for row in 0..base.len_lines() {
        assert_eq!(
            mapping.base_byte(row, buffer.text.line_to_byte(row)),
            Some(base.line_to_byte(row)),
            "row {row}"
        );
    }
}

// Compose real canonical publications earlier/later on the same two rows.
// Unicode byte deltas must never be mistaken for scalar positions, and the
// fixed record must keep zero-net lexical dirtiness without retaining a log.
#[test]
fn composed_rows() {
    for (name, inserted) in [("ascii", "xyz"), ("wide", "界"), ("combining", "e\u{301}")] {
        let mut buffer = observed("* Root\nbody text\nmore text\n** Deep\n");
        let base = buffer.text.clone();
        for (row, column) in [(2, 3), (1, 4), (2, 0), (1, 0), (2, 5), (1, 6)] {
            let start = buffer.text.line_to_char(row) + column;
            edit(&mut buffer, start..start, inserted);
            mapped_rows(&buffer, &base);
        }
        let map = buffer.presentation.unwrap();
        assert_eq!(map.rows.iter().flatten().count(), 2, "{name}");
        assert_eq!(map.base.revision, 1, "{name}");
        assert!(buffer.org_outline().is_none(), "exact authority: {name}");
        let exact = &buffer.org_outline.as_ref().unwrap();
        let current = buffer.text.line_to_byte(3);
        assert_eq!(
            exact.heading_level_at(map.base_byte(3, current).unwrap()),
            Some(2),
            "{name}"
        );
        let final_source = buffer.text.clone();
        analyze(&mut buffer);
        assert_eq!(buffer.org_heading_level(3), Some(2), "{name}");
        assert_eq!(buffer.text, final_source, "{name}");
    }
}

// Replacing multibyte source repeatedly, including undoing its net size, must
// consume one row and leave a dirty row even when the cumulative delta is zero.
#[test]
fn repeated_replacements() {
    let mut buffer = observed("* Root\nwide 界 text\n** Deep\n");
    let base = buffer.text.clone();
    for _ in 0..40 {
        for (from, to) in [("界", "a"), ("a", "界")] {
            let start = buffer.text.to_string().find(from).unwrap();
            let start = buffer.text.byte_to_char(start);
            edit(&mut buffer, start..start + from.chars().count(), to);
            mapped_rows(&buffer, &base);
        }
    }
    let mapping = buffer.presentation.unwrap();
    let rows: Vec<_> = mapping.rows.iter().flatten().collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].delta, 0);
    assert_eq!(mapping.current.revision, 81);
}

// Every publication of a refused shape retires the chain, while the ordinary
// source transaction succeeds. A later harmless edit cannot bridge that gap.
#[test]
fn refused_shapes() {
    let cases = [
        ("insert lf", vec![(8..8, "\n")]),
        ("insert cr", vec![(8..8, "\r")]),
        ("remove lf", vec![(6..7, "")]),
        ("remove cr", vec![(17..18, "")]),
        ("two changes", vec![(8..8, "z"), (18..18, "z")]),
    ];
    for (name, changes) in cases {
        let mut buffer = observed("* Root\nbody text\rnext\n** Deep\n");
        edit(&mut buffer, 9..9, "a");
        assert!(buffer.presentation.is_some(), "{name}");
        let edits: Vec<_> = changes
            .into_iter()
            .map(|(range, text)| replacement(&buffer, range, text))
            .collect();
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        assert!(buffer.presentation.is_none(), "{name}");
        edit(&mut buffer, 8..8, "a");
        assert!(buffer.presentation.is_none(), "gap: {name}");
    }
}

// Eight rows are accepted in arbitrary order; the ninth retires mapping before
// publishing the next source revision. Repeated edits do not consume new slots.
#[test]
fn row_ceiling() {
    let mut buffer = observed(&format!("* Root\n{}** Deep\n", "body\n".repeat(9)));
    let base = buffer.text.clone();
    for row in [8, 1, 7, 2, 6, 3, 5, 4, 1, 8] {
        let start = buffer.text.line_to_char(row);
        edit(&mut buffer, start..start, "界");
        mapped_rows(&buffer, &base);
    }
    assert_eq!(
        buffer.presentation.unwrap().rows.iter().flatten().count(),
        8
    );
    let start = buffer.text.line_to_char(9);
    edit(&mut buffer, start..start, "z");
    assert!(buffer.presentation.is_none());
}

// Before+after lines and both payload passes share the admission budget.
// An oversized line/payload must leave source editing available without a copy.
#[test]
fn scan_ceiling() {
    for (name, length, inserted, admitted) in [
        ("last byte", SCAN_BYTES / 2 - 2, "z", true),
        ("over line", SCAN_BYTES / 2 - 1, "z", false),
        ("short line", 1, "x", true),
    ] {
        let mut buffer = observed(&format!("* Root\n{}\n", "a".repeat(length)));
        let before = buffer.text.clone();
        let stamp = buffer.stamp();
        edit(&mut buffer, 7..7, inserted);
        let mut remaining = SCAN_BYTES;
        let shape = Mapping::exact(stamp, before.len_bytes()).prepare(
            &before,
            &buffer.text,
            buffer.history().back().unwrap(),
            &mut remaining,
        );
        assert_eq!(shape.is_some(), admitted, "{name}");
    }
    let mut buffer = observed("* Root\na\n");
    edit(&mut buffer, 7..7, &"x".repeat(SCAN_BYTES + 1));
    assert!(buffer.presentation.is_none());
}

// Stale preparation never publishes any state, and reserving an edit prepares
// all mapping before the infallible installation used by atomic native batches.
#[test]
fn reserved_publication() {
    let mut buffer = observed("* Root\nbody\n** Deep\n");
    let base = buffer.text.clone();
    let change = replacement(&buffer, 8..8, "界");
    let stale = buffer
        .prepare(std::slice::from_ref(&change), &[], None)
        .unwrap();
    let prepared = buffer.prepare(&[change], &[], None).unwrap();
    let reserved = buffer.reserve_edit(prepared, &[]).unwrap();
    assert!(buffer.presentation.is_none());
    assert_eq!(buffer.text, base);
    buffer.check_reserved_edit(&reserved, &[]).unwrap();
    buffer.commit_reserved_edit(reserved, &mut []);
    mapped_rows(&buffer, &base);
    let installed = buffer.presentation;
    assert!(buffer.publish(stale, &mut []).is_err());
    assert_eq!(buffer.presentation, installed);
    assert_eq!(buffer.history().len(), 1);
}

// Both history directions and reload retire mapping even for a same-line diff;
// neither may turn a source replay into an admitted ordinary carry publication.
#[test]
fn history_and_reload() {
    for name in ["undo", "redo", "reload"] {
        let mut buffer = observed("* Root\nbody\n");
        edit(&mut buffer, 8..8, "z");
        match name {
            "undo" => buffer
                .navigate(crate::buffer::edit::History::Undo, &mut [], None)
                .unwrap(),
            "redo" => {
                buffer
                    .navigate(crate::buffer::edit::History::Undo, &mut [], None)
                    .unwrap();
                analyze(&mut buffer);
                buffer
                    .navigate(crate::buffer::edit::History::Redo, &mut [], None)
                    .unwrap();
            }
            _ => {
                let prepared = buffer
                    .prepare_reload_text(&Rope::from_str("* Root\nnext\n"), false, &[])
                    .unwrap();
                buffer.publish(prepared, &mut []).unwrap();
            }
        }
        assert!(buffer.presentation.is_none(), "{name}");
    }
}

// Checked identities and every mapping arithmetic boundary reject rather than
// wrap. Synthetic extremes exercise states too large for an allocated rope.
#[test]
fn checked_arithmetic() {
    let buffer = observed("* Root\nbody\n");
    let initial = Mapping::exact(buffer.stamp(), 12);
    for (name, row, start, delta) in [
        ("negative length", 0, 0, -13),
        ("positive length overflow", 0, 0, isize::MAX),
        ("negative mapping overflow", 1, 0, isize::MIN),
    ] {
        let mut mapping = initial;
        if name == "positive length overflow" {
            mapping.bytes = usize::MAX;
        }
        let next = Stamp {
            revision: 2,
            ..buffer.stamp()
        };
        let mutation = Mutation { row, start, delta };
        assert!(mapping.advance(&mutation, next).is_none(), "{name}");
    }
    let mut mapping = initial;
    mapping.rows[0] = Some(Row {
        index: 0,
        base_start: 0,
        delta: 3,
    });
    assert_eq!(mapping.base_byte(1, 2), None);
    assert_eq!(mapping.base_byte(0, 0), Some(0));
    let foreign = Buffer::from_rope(Rope::new()).unwrap().stamp();
    for next in [
        foreign,
        Stamp {
            revision: 3,
            ..buffer.stamp()
        },
    ] {
        assert!(
            initial
                .advance(
                    &Mutation {
                        row: 0,
                        start: 0,
                        delta: 1
                    },
                    next
                )
                .is_none()
        );
    }
}

// No current chain may be seeded by a missing, partial or canceled base. A
// fresh complete result starts a new base instead of reusing a retired gap.
#[test]
fn base_admission() {
    for (name, limits, cancelled) in [
        ("bytes", Limits::new(4, 200, 20).unwrap(), false),
        ("nodes", Limits::new(4096, 1, 20).unwrap(), false),
        ("cancelled", Limits::default(), true),
    ] {
        let mut buffer = observed("* Root\nbody\n** Deep\n");
        let cancel = Cancellation::default();
        if cancelled {
            cancel.cancel();
        }
        let coloring = ColorScan::new(&buffer, limits).unwrap().run(&cancel);
        buffer.install_org_coloring(coloring).unwrap();
        edit(&mut buffer, 8..8, "z");
        assert!(buffer.presentation.is_none(), "{name}");
    }
    let mut buffer = Buffer::from_rope(Rope::from_str("* Root\nbody\n")).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    edit(&mut buffer, 8..8, "z");
    assert!(buffer.presentation.is_none());
    analyze(&mut buffer);
    let base = buffer.stamp();
    edit(&mut buffer, 8..8, "z");
    assert_eq!(buffer.presentation.unwrap().base, base);
}

// History grouping must not replace the atomic change used by the mapper.
// Five keys share one undo unit while their five source revisions still map.
#[test]
fn typing_publications() {
    use crate::buffer::edit::{Cursor, Point};
    use crate::coordinates::Scalar;
    let mut buffer = observed("* Root\nbody\n** Deep\n");
    let base = buffer.text.clone();
    let mut points = [Point {
        stamp: buffer.stamp(),
        position: Scalar(8),
    }];
    for key in ["x", "界", "y", "z", "a"] {
        let start = points[0].position.0;
        let change = replacement(&buffer, start..start, key);
        let cursor = Cursor {
            index: 0,
            position: Scalar(start + 1),
        };
        let prepared = buffer.prepare_typing(&change, &points, cursor).unwrap();
        buffer.publish(prepared, &mut points).unwrap();
        mapped_rows(&buffer, &base);
    }
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(
        buffer.presentation.unwrap().rows.iter().flatten().count(),
        1
    );
}

// Representation overflow is rejected both within one dirty row and across
// rows, and the byte-to-base mapping cannot overflow in either direction.
#[test]
fn cumulative_overflow() {
    let buffer = observed("* Root\nbody\n");
    let initial = Mapping::exact(buffer.stamp(), 12);
    for row in [0, 1] {
        let mut mapping = initial;
        mapping.rows[0] = Some(Row {
            index: 0,
            base_start: 0,
            delta: isize::MAX,
        });
        let start = if row == 0 { 0 } else { usize::MAX };
        let mutation = Mutation {
            row,
            start,
            delta: 1,
        };
        let next = Stamp {
            revision: 2,
            ..buffer.stamp()
        };
        assert!(mapping.advance(&mutation, next).is_none());
    }
    let mut mapping = initial;
    mapping.rows[0] = Some(Row {
        index: 0,
        base_start: 0,
        delta: -1,
    });
    assert_eq!(mapping.base_byte(1, usize::MAX), None);
    mapping.current.revision = u64::MAX;
    let next = Stamp {
        revision: 0,
        ..buffer.stamp()
    };
    assert!(
        mapping
            .advance(
                &Mutation {
                    row: 0,
                    start: 0,
                    delta: 0
                },
                next
            )
            .is_none()
    );
}
