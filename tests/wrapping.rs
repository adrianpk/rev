#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::coordinates::Scalar;
use rev::display::{Frame, Position, Size};
use rev::gutter::Change;
use rev::view::View;

// Word breaks preserve every source scalar; tokens and oversized graphemes
// advance finitely without emitting half a cluster or altering canonical text.
#[test]
fn source_fragments() {
    let cases = [
        ("words", "one two three", 8, vec!["one two ", "three   "]),
        ("token", "abcdefghij", 4, vec!["abcd", "efgh", "ij  "]),
        (
            "combining",
            "e\u{301}e\u{301}e\u{301}",
            2,
            vec!["e\u{301}e\u{301}", "e\u{301} "],
        ),
        ("wide", "界界x", 3, vec!["界 ", "界x", "   "]),
        ("tiny", "界x", 1, vec![" ", "x", " "]),
        ("tabs", "a\tb", 4, vec!["a   ", "b   "]),
        ("empty", "", 4, vec!["    "]),
    ];
    for (name, source, width, expected) in cases {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let view = View::new(buffer);
        let fragments = view.wrapped_rows(buffer, width, 20).unwrap();
        assert_eq!(
            fragments
                .iter()
                .map(|f| f.text.as_str())
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
        let mut end = Scalar(0);
        for fragment in &fragments {
            assert_eq!(fragment.line, 0, "{name}");
            assert_eq!(fragment.source.start, end, "{name}");
            assert!(
                rev::coordinates::Coordinates::new(buffer)
                    .unwrap()
                    .is_boundary(fragment.source.end)
                    .unwrap(),
                "{name}"
            );
            end = fragment.source.end;
        }
        assert_eq!(end, Scalar(source.chars().count()), "{name}");
        assert!(!buffer.is_modified(), "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert!(
            view.wrapped_rows(buffer, 0, 20).unwrap().is_empty(),
            "{name}"
        );
    }
}

// Actual rendering and reveal use pane-local fragments over one buffer. A
// continuation cursor remains visible after resize without changing its point.
#[test]
fn pane_widths() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("one two three four five six").unwrap();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let mut view = View::new(buffer);
    view.gutter.apply(Change::Visible(false));
    view.point.position = Scalar(buffer.len_chars());
    for width in [12, 8, 3, 15] {
        let size = Size { width, height: 5 };
        view.reveal(buffer, size).unwrap();
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        let cursor = frame.cursor().unwrap();
        assert!(cursor.column < width && cursor.row < size.text_rows());
        assert_eq!(view.point.position, Scalar(buffer.len_chars()));
        assert_eq!(frame.stamp(), stamp);
        assert_eq!(view.left, 0);
    }
    let mut other = View::new(buffer);
    other.gutter.apply(Change::Visible(false));
    let frame = Frame::project(
        buffer,
        &other,
        Size {
            width: 8,
            height: 6,
        },
        "",
    )
    .unwrap();
    assert_eq!(frame.rows()[0].text(), "one two ");
    assert_eq!(frame.rows()[1].text(), "three   ");
    assert_eq!(frame.cursor(), Some(Position { column: 0, row: 0 }));
    assert_eq!(buffer.stamp(), stamp);
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
}
