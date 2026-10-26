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

fn bounded(frame: &Frame, size: Size) {
    for row in frame.rows() {
        assert!(row.row() < size.height);
        assert_eq!(row.text().width(), usize::from(size.width));
        assert!(!row.text().chars().any(char::is_control));
    }
    if let Some(cursor) = frame.cursor() {
        assert!(cursor.column < size.width);
        assert!(cursor.row < size.text_rows());
    }
}

// Cell clipping never prints half a wide/emoji cluster or shifts following
// text. Tabs use logical stops even when their leading cells are offscreen.
#[test]
fn mixed_clipping() {
    let cases = [
        ("right wide", "ab界z", 0, 3, "ab "),
        ("left wide", "ab界z", 3, 3, " z "),
        ("whole wide", "ab界z", 2, 3, "界z"),
        ("both edges", "界界界", 1, 3, " 界"),
        ("tab fragment", "a\t界e\u{301}", 2, 4, "  界"),
        ("tab stops", "\t\tX", 3, 6, "     X"),
        ("combining", "ae\u{301}z", 1, 3, "e\u{301}z "),
        ("emoji edge", "a👩\u{200d}💻z", 2, 3, " z "),
        ("leading mark", "\u{301}x", 0, 3, "◌\u{301}x "),
        ("CRLF", "a\r\nb", 0, 3, "a  "),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, left, width, expected) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
        view.left = left;
        let size = Size { width, height: 3 };
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        assert_eq!(frame.rows()[0].text(), expected, "{name}");
        bounded(&frame, size);
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap()
                .to_string(),
            source,
            "{name}"
        );
        assert!(!buffer.is_modified(), "{name}");
    }
}

// Document and message controls use safe visible glyphs; the separate echo row
// can report an error without replacing the document or its mode indication.
#[test]
fn safe_echo() {
    let mut buffers = Buffers::new().unwrap();
    let source = "x\x1b[2J\u{9b}";
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let view = View::new(buffer);
    let size = Size {
        width: 40,
        height: 4,
    };
    let frame = Frame::project(buffer, &view, size, "Save failed: denied\x1b[2J\n").unwrap();
    assert!(frame.rows()[0].text().starts_with("x^[[2J<9B>"));
    assert!(frame.rows()[2].text().contains("-- RW L1 C1 Text"));
    assert!(
        frame.rows()[3]
            .text()
            .starts_with("Save failed: denied^[[2J^J")
    );
    bounded(&frame, size);
    assert_eq!(buffer.len_bytes(), source.len());
    assert!(buffer.history().is_empty());
}

// Every tiny shape, including zero dimensions, is a valid suspended frame.
// Resizing presentation cannot rewrite text, point, or its revision stamp.
#[test]
fn tiny_geometry() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界e\u{301}\r\nlast").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut original = View::new(buffer);
    original.point.position = Scalar(buffer.len_chars());
    original.top = 1;
    original.left = 2;
    for (width, height) in (0..6).flat_map(|width| (0..6).map(move |height| (width, height))) {
        let size = Size { width, height };
        let mut view = original.clone();
        view.reveal(buffer, size).unwrap();
        let frame = Frame::project(buffer, &view, size, "error").unwrap();
        bounded(&frame, size);
        assert_eq!(
            frame.rows().len(),
            if width == 0 { 0 } else { usize::from(height) }
        );
        assert_eq!(view.point, original.point);
        if !size.usable() {
            assert_eq!(view, original);
            assert_eq!(frame.cursor(), None);
            assert!(frame.rows().iter().all(|row| row.band() != Band::Status));
        }
    }
    assert_eq!(buffer.stamp(), original.point.stamp);
    assert!(!buffer.is_modified());
}

// Native dimension limits and excessive scroll offsets cannot overflow row or
// cell arithmetic. A usable reveal repairs scroll without touching the point.
#[test]
fn extreme_geometry() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("界\tend").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.top = usize::MAX;
    view.left = usize::MAX;
    for size in [
        Size {
            width: u16::MAX,
            height: 1,
        },
        Size {
            width: 1,
            height: u16::MAX,
        },
        Size {
            width: 3,
            height: 3,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "").unwrap();
        bounded(&frame, size);
        assert_eq!(frame.cursor(), None);
    }
    view.top = 0;
    let clipped = Frame::project(
        buffer,
        &view,
        Size {
            width: 3,
            height: 3,
        },
        "",
    )
    .unwrap();
    assert_eq!(clipped.rows()[0].text(), "   ");
    assert_eq!(clipped.cursor(), None);
    view.reveal(
        buffer,
        Size {
            width: 3,
            height: 3,
        },
    )
    .unwrap();
    assert_eq!((view.top, view.left), (0, 0));
    assert_eq!(view.point.position, Scalar(0));
}

// Reprojection reads both current rope text and current metadata. A read-only
// change needs no text revision to update status, and an edit updates it once.
#[test]
fn fresh_projection() {
    use rev::buffer::edit::Replacement;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut view = View::new(buffer);
    let size = Size {
        width: 48,
        height: 4,
    };
    let before = Frame::project(buffer, &view, size, "").unwrap();
    let edits = [Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "a\t界e\u{301}".into(),
    }];
    let prepared = buffer
        .prepare(&edits, std::slice::from_ref(&view.point), None)
        .unwrap();
    buffer
        .publish(prepared, std::slice::from_mut(&mut view.point))
        .unwrap();
    let edited = Frame::project(buffer, &view, size, "Save failed: denied").unwrap();
    assert_ne!(before.stamp(), edited.stamp());
    assert_eq!(edited.stamp(), buffer.stamp());
    assert!(edited.rows()[0].text().starts_with("a   界e\u{301}"));
    assert!(
        edited.rows()[2]
            .text()
            .starts_with("> *scratch* ** RW L1 C8 Text")
    );
    buffer.set_read_only(true);
    let readonly = Frame::project(buffer, &view, size, "Save failed: denied").unwrap();
    assert_eq!(readonly.stamp(), edited.stamp());
    assert!(
        readonly.rows()[2]
            .text()
            .starts_with("> *scratch* ** RO L1 C8 Text")
    );
    assert_eq!(readonly.rows()[0].text(), edited.rows()[0].text());
    assert!(readonly.rows()[3].text().starts_with("Save failed: denied"));
    assert_eq!(buffer.history().len(), 1);
    bounded(&readonly, size);
}

// Projection rejects foreign/stale points and invalid scalar/grapheme
// boundaries instead of displaying a cursor detached from its source revision.
#[test]
fn qualified_views() {
    use rev::buffer::{Error as Source, Stamp};
    use rev::coordinates::Error as Coordinate;
    use rev::view::Error;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}界").unwrap();
    let other = buffers.initial();
    let buffer = buffers.get(id).unwrap();
    let stamp = buffer.stamp();
    let cases = [
        (
            "foreign",
            Stamp {
                buffer: other,
                ..stamp
            },
            0,
            Error::Buffer(Source::Foreign {
                expected: id,
                actual: other,
            }),
        ),
        (
            "stale",
            Stamp {
                revision: 0,
                ..stamp
            },
            0,
            Error::Buffer(Source::Stale {
                expected: 1,
                actual: 0,
            }),
        ),
        (
            "outside",
            stamp,
            4,
            Error::Buffer(Source::Bounds {
                start: 4,
                end: 4,
                len: 3,
            }),
        ),
        (
            "inside cluster",
            stamp,
            1,
            Error::Coordinates(Coordinate::Cluster(Scalar(1))),
        ),
    ];
    for (name, stamp, position, expected) in cases {
        let mut view = View::new(buffer);
        view.point.stamp = stamp;
        view.point.position = Scalar(position);
        assert_eq!(
            Frame::project(
                buffer,
                &view,
                Size {
                    width: 8,
                    height: 3
                },
                ""
            )
            .err(),
            Some(expected),
            "{name}"
        );
    }
    assert!(!buffer.is_modified());
}

#[derive(Default)]
struct Output {
    bytes: Vec<u8>,
    flushes: usize,
}

impl std::io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flushes += 1;
        Ok(())
    }
}

// The actual output adapter writes admitted row moves with safe full-width
// payloads and one deliberate flush; zero geometry emits only cursor hiding.
#[test]
fn draw_boundary() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a\x1b[2J").unwrap();
    let buffer = buffers.get(id).unwrap();
    let view = View::new(buffer);
    for size in [
        Size {
            width: 8,
            height: 3,
        },
        Size {
            width: 0,
            height: 0,
        },
        Size {
            width: 2,
            height: 2,
        },
    ] {
        let frame = Frame::project(buffer, &view, size, "\x1b[2J").unwrap();
        bounded(&frame, size);
        let mut output = Output::default();
        frame.write(&mut output).unwrap();
        assert_eq!(output.flushes, 1);
        let wire = String::from_utf8(output.bytes).unwrap();
        assert!(wire.starts_with("\x1b[?25l"));
        assert_eq!(wire.ends_with("\x1b[?25h"), frame.cursor().is_some());
        assert!(!wire.contains("\x1b[2J"));
        for row in frame.rows() {
            assert!(wire.contains(&format!("\x1b[{};1H{}", row.row() + 1, row.text())));
        }
        if size.width == 0 {
            assert_eq!(wire, "\x1b[?25l");
        }
    }
}
