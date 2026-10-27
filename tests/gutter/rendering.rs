use rev::config::{Palette, Rgb};
use rev::coordinates::Scalar;
use rev::display::{Frame, Position};
use rev::gutter::{NumberRow, Separator};
use rev::view::{Size, View};
use unicode_width::UnicodeWidthStr;

use super::{Buffers, surface};

// The actual frame numbers only real logical rows, including the editable empty
// row. Tabs and Unicode/CRLF remain source text rather than synthetic rows.
#[test]
fn absolute_rows() {
    for (name, source, lines) in [
        ("empty", "", 1),
        ("crlf", "\r\n", 2),
        ("mixed", "界\te\u{301}\r\nlast\n", 3),
        ("no final", "a\nb", 2),
    ] {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get(id).unwrap();
        let view = View::new(buffer);
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 30,
                height: 7,
            },
            "notice",
        )
        .unwrap();
        for row in 0..5 {
            let label = if row < lines {
                format!("{:>4} │", row + 1)
            } else {
                "     │".into()
            };
            assert_eq!(
                frame.rows()[row].text().chars().take(6).collect::<String>(),
                label,
                "{name}/{row}"
            );
            assert_eq!(frame.rows()[row].text().width(), 30);
        }
        assert_eq!(frame.cursor(), Some(Position { column: 6, row: 0 }));
        assert!(frame.rows()[5].text().contains("-- RW L1 C1 Text"));
        assert!(frame.rows()[6].text().starts_with("notice"));
        let mut bytes = Vec::new();
        rev::file::write(buffer, &mut bytes).unwrap();
        assert_eq!(bytes, source.as_bytes());
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}

// Number records are checked integer metadata: no row object for padding and no
// overflow or truncation for the largest admitted one-based logical label.
#[test]
fn row_boundaries() {
    for (line, count, current, expected) in [
        (0, 1, 0, Some((1, true))),
        (1, 3, 0, Some((2, false))),
        (2, 3, 2, Some((3, true))),
        (1, 1, 0, None),
        (0, 0, 0, None),
        (
            usize::MAX - 1,
            usize::MAX,
            usize::MAX - 1,
            Some((usize::MAX, true)),
        ),
        (usize::MAX, usize::MAX, 0, None),
    ] {
        assert_eq!(
            NumberRow::absolute(line, count, current).map(|r| (r.label.unwrap(), r.current)),
            expected
        );
    }
}

// Actual SGR cells distinguish ordinary/current labels and separator, and bold
// emphasis is reset before document/status cells. ASCII fallback changes only
// separator output, preserving every cell and a literal document separator.
#[test]
fn semantic_cells() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("│a\nb").unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(3);
    let mut palette = Palette::default();
    palette.gutter.background = Rgb(1, 2, 3);
    palette.gutter.foreground = Rgb(4, 5, 6);
    palette.gutter.current_line = Rgb(7, 8, 9);
    palette.gutter.separator = Rgb(10, 11, 12);
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 30,
            height: 6,
        },
        "",
    )
    .unwrap()
    .with_palette(&palette);
    let primary = surface::draw(&frame);
    for (separator, character) in [(Separator::Unicode, '│'), (Separator::Ascii, '|')] {
        let cells = surface::draw_with_separator(&frame, separator);
        assert_eq!(separator.text().width(), 1);
        assert_eq!(cells[&(0, 3)].foreground, Some(palette.gutter.foreground));
        assert_eq!(cells[&(1, 3)].foreground, Some(palette.gutter.current_line));
        assert!(cells[&(1, 3)].bold);
        assert!(!cells[&(0, 3)].bold);
        assert_eq!(cells[&(0, 5)].character, character);
        assert_eq!(cells[&(0, 5)].foreground, Some(palette.gutter.separator));
        assert_eq!(cells[&(1, 3)].background, Some(palette.gutter.background));
        assert_eq!(cells[&(0, 6)].character, '│');
        assert_eq!(cells[&(0, 6)].foreground, Some(palette.foreground));
        assert_eq!(cells[&(1, 6)].background, Some(palette.background));
        assert!(!cells[&(1, 6)].bold);
        assert_eq!(cells.len(), primary.len());
        assert_eq!(frame.cursor(), Some(Position { column: 6, row: 1 }));
    }
}
