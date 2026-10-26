#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Band, Frame, Position, Size};
use rev::view::View;
use unicode_width::UnicodeWidthStr;

// The initial frame borrows the canonical empty buffer, reserves distinct
// document/status/echo rows, and places the active cursor at the first cell.
#[test]
fn initial_frame() {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let view = View::new(buffer);
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 80,
            height: 24,
        },
        "C-x C-c: exit",
    )
    .unwrap();
    assert_eq!(frame.stamp(), buffer.stamp());
    assert_eq!(frame.cursor(), Some(Position { column: 0, row: 0 }));
    assert_eq!(frame.rows().len(), 24);
    assert!(
        frame.rows()[..22]
            .iter()
            .all(|row| row.band() == Band::Document && row.text().trim().is_empty())
    );
    assert_eq!(frame.rows()[22].band(), Band::Status);
    assert!(
        frame.rows()[22]
            .text()
            .starts_with("> *scratch* -- RW L1 C1 Text")
    );
    assert_eq!(frame.rows()[23].band(), Band::Echo);
    assert!(frame.rows()[23].text().starts_with("C-x C-c: exit"));
    for (index, row) in frame.rows().iter().enumerate() {
        assert_eq!(usize::from(row.row()), index);
        assert_eq!(row.text().width(), 80);
    }
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}

// Both scroll axes belong to the view. Revealing a point projects only its
// visible logical rows and uses terminal-cell rather than scalar columns.
#[test]
fn viewport_reveal() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .create("first\nsecond\na\t界e\u{301}\nlast")
        .unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(18);
    let size = Size {
        width: 4,
        height: 4,
    };
    view.reveal(buffer, size).unwrap();
    assert_eq!((view.top, view.left), (1, 4));
    let frame = Frame::project(buffer, &view, size, "").unwrap();
    assert_eq!(frame.rows()[0].text(), "nd  ");
    assert_eq!(frame.rows()[1].text(), "界e\u{301} ");
    assert_eq!(frame.cursor(), Some(Position { column: 3, row: 1 }));
    let mut independent = View::new(buffer);
    independent.reveal(buffer, size).unwrap();
    assert_eq!((independent.top, independent.left), (0, 0));
    assert_eq!(buffer.stamp(), view.point.stamp);
}
