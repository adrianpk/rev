use rev::coordinates::Scalar;
use rev::display::{Frame, Position};
use rev::gutter::{Change, Side};
use rev::view::{Size, View};
use unicode_width::UnicodeWidthStr;

use super::Buffers;

// Reveal uses final text width; tab stops and whole Unicode clusters remain
// logical even when the number field is moved to the other side or hidden.
#[test]
fn text_origin() {
    for side in [Side::Left, Side::Right] {
        for (width, left, expected) in [
            (9, 6, "e\u{301}Z"),
            (12, 3, " 界e\u{301}Z"),
            (80, 0, "a   界e\u{301}Z"),
        ] {
            for wrap in [true, false] {
                origin_case(side, width, left, expected, wrap);
            }
        }
    }
}

fn origin_case(side: Side, width: u16, left: usize, expected: &str, wrap: bool) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}Z").unwrap();
    if !wrap {
        buffers.get_mut(id).unwrap().toggle_soft_wrap();
    }
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.gutter.apply(Change::Side(side));
    view.point.position = Scalar(buffer.len_chars());
    let size = Size { width, height: 4 };
    view.reveal(buffer, size).unwrap();
    assert_eq!(view.left, if wrap { 0 } else { left }, "{side:?}/{width}");
    let frame = Frame::project(buffer, &view, size, "").unwrap();
    let geometry = view
        .gutter_geometry(buffer, super::area(0, 0, width, 2))
        .unwrap();
    if wrap {
        let (column, row, text, label) = match width {
            9 => (1, 1, "界e\u{301}", "     │"),
            12 => (4, 1, "a", "   1 │"),
            _ => (8, 0, "a   界e\u{301}Z", "   1 │"),
        };
        assert_eq!(
            frame.cursor(),
            Some(Position {
                column: geometry.text().column() + column,
                row
            })
        );
        let rendered = frame.rows()[0].text();
        let (prefix, suffix) = match side {
            Side::Left => (label, ""),
            Side::Right => ("", if width == 9 { "│     " } else { "│   1 " }),
        };
        assert_eq!(
            rendered
                .strip_prefix(prefix)
                .unwrap()
                .strip_suffix(suffix)
                .unwrap()
                .trim_end(),
            text
        );
        assert_eq!(rendered.width(), usize::from(width));
    } else {
        let expected_cursor = geometry.text().column() + u16::try_from(8 - left).unwrap();
        assert_eq!(
            frame.cursor(),
            Some(Position {
                column: expected_cursor,
                row: 0
            })
        );
        let row = frame.rows()[0].text();
        assert_eq!(row.width(), usize::from(width));
        let text = match side {
            Side::Left => row.strip_prefix("   1 │").unwrap(),
            Side::Right => row.strip_suffix("│   1 ").unwrap(),
        };
        assert_eq!(text.trim_end(), expected);
    }
    view.gutter.apply(Change::Visible(false));
    view.left = 0;
    view.reveal(buffer, size).unwrap();
    let hidden = Frame::project(buffer, &view, size, "").unwrap();
    assert_eq!(view.left, 0);
    assert_eq!(hidden.cursor(), Some(Position { column: 8, row: 0 }));
    assert!(hidden.rows()[0].text().starts_with("a   界e\u{301}Z"));
    assert!(buffer.history().is_empty());
    assert!(!buffer.is_modified());
}
