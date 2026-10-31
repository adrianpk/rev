use rev::config::Palette;
use rev::coordinates::{Coordinates, LineByte};
use rev::display::{Frame, Position};
use rev::gutter::{Change, CurrentLine, Mode, NumberRow, Role, Settings};
use rev::view::{Size, View};

use super::{Buffers, surface};

pub(super) const SIZE: Size = Size {
    width: 30,
    height: 8,
};

pub(super) fn point(view: &mut View, buffer: &rev::buffer::Buffer, row: usize) {
    view.point.position = Coordinates::new(buffer)
        .unwrap()
        .from_line(LineByte { row, column: 0 })
        .unwrap();
}

pub(super) fn prefix(frame: &Frame, row: usize) -> String {
    frame.rows()[row].text().chars().take(6).collect()
}

// Both calculation modes honor each current-line form. The expected cells are
// explicit and include source text, current role/reset and unnumbered EOF rows.
#[test]
fn current_forms() {
    for (mode, form, expected) in [
        (
            Mode::Absolute,
            CurrentLine::Absolute,
            ["   1 │", "   2 │", "   3 │"],
        ),
        (
            Mode::Absolute,
            CurrentLine::Zero,
            ["   1 │", "   0 │", "   3 │"],
        ),
        (
            Mode::Absolute,
            CurrentLine::Hidden,
            ["   1 │", "     │", "   3 │"],
        ),
        (
            Mode::Relative,
            CurrentLine::Absolute,
            ["   1 │", "   2 │", "   1 │"],
        ),
        (
            Mode::Relative,
            CurrentLine::Zero,
            ["   1 │", "   0 │", "   1 │"],
        ),
        (
            Mode::Relative,
            CurrentLine::Hidden,
            ["   1 │", "     │", "   1 │"],
        ),
    ] {
        form_case(mode, form, expected);
    }
}

fn form_case(mode: Mode, form: CurrentLine, expected: [&str; 3]) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界\te\u{301}\r\nmid\n").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    point(&mut view, buffer, 1);
    view.gutter.apply(Change::Mode(mode));
    view.gutter.apply(Change::CurrentLine(form));
    let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
    for (row, expected) in expected.iter().enumerate() {
        assert_eq!(prefix(&frame, row), *expected, "{mode:?}/{form:?}/{row}");
    }
    assert_eq!(prefix(&frame, 3), "     │");
    assert!(
        frame.rows()[1]
            .text()
            .chars()
            .skip(6)
            .collect::<String>()
            .starts_with("mid")
    );
    assert_eq!(frame.cursor(), Some(Position { column: 6, row: 1 }));
    let cells = surface::draw(&frame);
    assert!(cells[&(1, 3)].bold);
    assert_eq!(
        cells[&(1, 3)].foreground,
        Some(Palette::default().gutter.current_line)
    );
    assert!(!cells[&(0, 3)].bold);
    assert!(!cells[&(1, 6)].bold);
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
    let mut bytes = Vec::new();
    rev::file::write(buffer, &mut bytes).unwrap();
    assert_eq!(bytes, "界\te\u{301}\r\nmid\n".as_bytes());
}

// A viewport can be far from point. Distances use logical rows without overflow
// at the largest representable source index; padding has no semantic row.
#[test]
fn distance_bounds() {
    let settings = Settings {
        mode: Mode::Relative,
        ..Settings::default()
    };
    for (name, row, count, current, label) in [
        ("before", 39, 100, 41, Some(2)),
        ("after", 43, 100, 41, Some(2)),
        ("current", 41, 100, 41, Some(42)),
        (
            "largest distance",
            0,
            usize::MAX,
            usize::MAX - 1,
            Some(usize::MAX - 1),
        ),
        (
            "largest current",
            usize::MAX - 1,
            usize::MAX,
            usize::MAX - 1,
            Some(usize::MAX),
        ),
        ("padding", 100, 100, 41, None),
        ("empty metadata", 0, 0, 0, None),
        ("invalid origin", 0, 1, 1, None),
    ] {
        let projected = NumberRow::new(row, count, current, settings);
        assert_eq!(projected.and_then(|r| r.label), label, "{name}");
        if let Some(projected) = projected {
            assert_eq!(projected.line, row);
            assert_eq!(
                projected.role(),
                if row == current {
                    Role::CurrentLine
                } else {
                    Role::Number
                }
            );
        }
    }
}

// Moving point and viewport changes distances, never the width derived from the
// complete source. This also exercises an offscreen relative origin.
#[test]
fn stable_origin() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"\n".repeat(9_999)).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.gutter.apply(Change::Mode(Mode::Relative));
    for (top, current, first) in [
        (39, 41, "    2 │"),
        (39, 0, "   39 │"),
        (9_994, 9_999, "    5 │"),
    ] {
        point(&mut view, buffer, current);
        view.top = top;
        let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
        assert_eq!(
            frame.rows()[0].text().chars().take(7).collect::<String>(),
            first
        );
        assert_eq!(frame.rows().len(), usize::from(SIZE.height));
        assert_eq!(
            view.gutter_geometry(buffer, super::area(0, 0, 30, 6))
                .unwrap()
                .digits(),
            5
        );
        if let Some(cursor) = frame.cursor() {
            assert_eq!(cursor.column, 7);
        }
    }
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}

// Relative numbering follows canonical logical rows at EOF. An empty source,
// CRLF, Unicode clusters and a final editable empty row never create padding labels.
#[test]
fn logical_rows() {
    for (name, source, labels) in [
        ("empty", "", vec!["   1 │"]),
        ("one Unicode", "界\te\u{301}", vec!["   1 │"]),
        ("CRLF", "\r\n", vec!["   1 │", "   2 │"]),
        ("no final", "a\nb", vec!["   1 │", "   2 │"]),
        (
            "mixed final",
            "界\r\nlast\n",
            vec!["   2 │", "   1 │", "   3 │"],
        ),
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.bookmark_markers = false;
        view.gutter.apply(Change::Mode(Mode::Relative));
        point(&mut view, buffer, labels.len() - 1);
        let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
        for (row, expected) in labels.iter().enumerate() {
            assert_eq!(prefix(&frame, row), *expected, "{name}/{row}");
        }
        assert_eq!(prefix(&frame, labels.len()), "     │", "{name}");
        assert_eq!(
            frame.cursor().unwrap().row,
            u16::try_from(labels.len() - 1).unwrap()
        );
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}

// Large source metadata changes field digits, while materialization remains the
// same six visible text rows. Hiding/restoring retains relative mode and point.
#[test]
fn visible_bound() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"\n".repeat(99_999)).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    point(&mut view, buffer, 99_999);
    view.top = 99_997;
    view.gutter.apply(Change::Mode(Mode::Relative));
    let original = view.clone();
    let frame = Frame::project(buffer, &view, SIZE, "").unwrap();
    for (row, expected) in [
        "     2 │",
        "     1 │",
        "100000 │",
        "       │",
        "       │",
        "       │",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(
            frame.rows()[row].text().chars().take(8).collect::<String>(),
            *expected
        );
    }
    assert_eq!(frame.rows().len(), usize::from(SIZE.height));
    assert_eq!(frame.cursor(), Some(Position { column: 8, row: 2 }));
    view.gutter.apply(Change::Visible(false));
    let hidden = Frame::project(buffer, &view, SIZE, "").unwrap();
    assert_eq!(hidden.cursor(), Some(Position { column: 0, row: 2 }));
    assert!(hidden.rows()[0].text().chars().all(|c| c == ' '));
    assert_eq!(view.gutter.effective().mode, Mode::Relative);
    view.gutter.apply(Change::Visible(true));
    let restored = Frame::project(buffer, &view, SIZE, "").unwrap();
    assert_eq!(restored.rows()[2].text(), frame.rows()[2].text());
    assert_eq!(
        (view.point, view.top, view.left),
        (original.point, original.top, original.left)
    );
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}
