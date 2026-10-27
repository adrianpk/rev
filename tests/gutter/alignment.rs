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
            origin_case(side, width, left, expected);
        }
    }
}

fn origin_case(side: Side, width: u16, left: usize, expected: &str) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\t界e\u{301}Z").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Side(side));
    view.point.position = Scalar(buffer.len_chars());
    let size = Size { width, height: 4 };
    view.reveal(buffer, size).unwrap();
    assert_eq!(view.left, left, "{side:?}/{width}");
    let frame = Frame::project(buffer, &view, size, "").unwrap();
    let geometry = view
        .gutter_geometry(buffer, super::area(0, 0, width, 2))
        .unwrap();
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
