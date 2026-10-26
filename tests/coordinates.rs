#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::coordinates::{Byte, Cell, Coordinates, Error, LineByte, Scalar};
use unicode_segmentation::UnicodeSegmentation;

// Non-newline controls must remain visible text within their logical line.
#[test]
fn control_lines() {
    let mut buffers = Buffers::new().unwrap();
    for (name, text) in [
        ("vertical tab", "a\u{b}b"),
        ("form feed", "a\u{c}b"),
        ("carriage return", "a\rb"),
    ] {
        let id = buffers.create(text).unwrap();
        assert_eq!(buffers.get(id).unwrap().len_lines(), 1, "{name}");
    }
}

// Parsing bytes and terminal cells must round-trip the same qualified boundaries.
#[test]
fn coordinate_units() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}").unwrap();
    let buffer = buffers.get(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    assert_eq!(coordinates.stamp(), buffer.stamp());
    let cases = [
        (
            "start",
            0,
            0,
            LineByte { row: 0, column: 0 },
            Some(Cell { row: 0, column: 0 }),
        ),
        (
            "wide start",
            1,
            1,
            LineByte { row: 0, column: 1 },
            Some(Cell { row: 0, column: 1 }),
        ),
        (
            "crlf start",
            2,
            4,
            LineByte { row: 0, column: 4 },
            Some(Cell { row: 0, column: 3 }),
        ),
        ("crlf interior", 3, 5, LineByte { row: 0, column: 5 }, None),
        (
            "second line",
            4,
            6,
            LineByte { row: 1, column: 0 },
            Some(Cell { row: 1, column: 0 }),
        ),
        (
            "combining interior",
            5,
            7,
            LineByte { row: 1, column: 1 },
            None,
        ),
        (
            "end",
            6,
            9,
            LineByte { row: 1, column: 3 },
            Some(Cell { row: 1, column: 1 }),
        ),
    ];
    for (name, scalar, byte, line, cell) in cases {
        let point = Scalar(scalar);
        assert_eq!(coordinates.to_byte(point).unwrap(), Byte(byte), "{name}");
        assert_eq!(coordinates.from_byte(Byte(byte)).unwrap(), point, "{name}");
        assert_eq!(coordinates.to_line(point).unwrap(), line, "{name}");
        assert_eq!(coordinates.from_line(line).unwrap(), point, "{name}");
        match cell {
            Some(cell) => {
                assert_eq!(coordinates.to_cell(point).unwrap(), cell, "{name}");
                assert_eq!(coordinates.from_cell(cell).unwrap(), point, "{name}");
            }
            None => assert_eq!(
                coordinates.to_cell(point).unwrap_err(),
                Error::Cluster(point),
                "{name}"
            ),
        }
    }
}

// Motion must never stop inside emoji, combining sequences, flags, or CRLF.
#[test]
fn grapheme_motion() {
    let mut buffers = Buffers::new().unwrap();
    let cases: [(&str, &str, &[usize]); 6] = [
        ("empty", "", &[0]),
        ("ascii", "ab", &[0, 1, 2]),
        ("combining", "e\u{301}", &[0, 2]),
        ("emoji modifier", "👩🏽‍💻", &[0, 4]),
        ("flag", "🇵🇱", &[0, 2]),
        ("mixed", "a👩‍💻e\u{301}🇵🇱\r\n界", &[0, 1, 4, 6, 8, 10, 11]),
    ];
    for (name, text, boundaries) in cases {
        let id = buffers.create(text).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        for point in 0..=text.chars().count() {
            let next = boundaries
                .iter()
                .copied()
                .find(|end| *end > point)
                .unwrap_or(point);
            let previous = boundaries
                .iter()
                .copied()
                .rfind(|start| *start < point)
                .unwrap_or(point);
            assert_eq!(
                coordinates.is_boundary(Scalar(point)).unwrap(),
                boundaries.contains(&point),
                "{name}:{point}"
            );
            assert_eq!(
                coordinates.next(Scalar(point)).unwrap(),
                Scalar(next),
                "{name}:{point}"
            );
            assert_eq!(
                coordinates.previous(Scalar(point)).unwrap(),
                Scalar(previous),
                "{name}:{point}"
            );
        }
    }
}

// Interior cells select the complete tab/wide cluster; scalar boundaries stay distinct.
#[test]
fn display_cells() {
    let mut buffers = Buffers::new().unwrap();
    let cases = [
        (
            "mixed",
            "a\t界e\u{301}👩‍💻",
            &[(0, 0), (1, 1), (2, 4), (3, 6), (5, 7), (8, 9)][..],
        ),
        ("tab stops", "\t\t", &[(0, 0), (1, 4), (2, 8)][..]),
        ("leading mark", "\u{301}x", &[(0, 0), (1, 1), (2, 2)][..]),
    ];
    for (name, text, boundaries) in cases {
        let id = buffers.create(text).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        for &(scalar, column) in boundaries {
            let cell = Cell { row: 0, column };
            assert_eq!(coordinates.to_cell(Scalar(scalar)).unwrap(), cell, "{name}");
            assert_eq!(
                coordinates.from_cell(cell).unwrap(),
                Scalar(scalar),
                "{name}"
            );
        }
    }
    let id = buffers.create("a\t界e\u{301}👩‍💻").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    let expected = [0, 1, 1, 1, 2, 2, 3, 5, 5, 8];
    for (column, scalar) in expected.into_iter().enumerate() {
        assert_eq!(
            coordinates.from_cell(Cell { row: 0, column }).unwrap(),
            Scalar(scalar)
        );
    }
    assert_eq!(
        coordinates
            .from_cell(Cell {
                row: 0,
                column: usize::MAX
            })
            .unwrap(),
        Scalar(8)
    );
}

// Document controls must become visible text, never terminal control sequences.
#[test]
fn safe_controls() {
    let mut buffers = Buffers::new().unwrap();
    let cases = [
        ("nul", "\0", "^@"),
        ("escape", "\x1b", "^["),
        ("carriage return", "\r", "^M"),
        ("delete", "\u{7f}", "^?"),
        ("next line", "\u{85}", "<85>"),
        ("csi", "\u{9b}", "<9B>"),
        ("unit separator", "\u{1f}", "^_"),
        ("vertical tab", "\u{b}", "^K"),
        ("form feed", "\u{c}", "^L"),
    ];
    for (name, control, expected) in cases {
        let id = buffers.create(&format!("a{control}b")).unwrap();
        let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(glyphs.len(), 3, "{name}");
        assert_eq!(glyphs[1].text, expected, "{name}");
        let rendered: String = glyphs.iter().map(|glyph| glyph.text.as_ref()).collect();
        assert!(!rendered.chars().any(char::is_control), "{name}");
        assert_eq!(
            coordinates.to_cell(Scalar(3)).unwrap(),
            Cell {
                row: 0,
                column: expected.len() + 2
            },
            "{name}"
        );
    }
    let id = buffers.create("\u{301}").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    let glyph = coordinates.glyphs(0).unwrap().next().unwrap().unwrap();
    assert_eq!(glyph.text, "◌\u{301}");
    assert_eq!(glyph.columns, 0..1);
}

// Invalid byte offsets and row aliases must fail before unchecked arithmetic or slicing.
#[test]
fn invalid_positions() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}").unwrap();
    let coordinates = Coordinates::new(buffers.get(id).unwrap()).unwrap();
    for point in [7, usize::MAX] {
        assert_eq!(
            coordinates.to_byte(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
        assert_eq!(
            coordinates.next(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
        assert_eq!(
            coordinates.previous(Scalar(point)),
            Err(Error::Scalar(Scalar(point)))
        );
    }
    for byte in [2, 3, 8, 10, usize::MAX] {
        assert_eq!(
            coordinates.from_byte(Byte(byte)),
            Err(Error::Byte(Byte(byte)))
        );
    }
    let cases = [
        ("interior byte", LineByte { row: 0, column: 2 }),
        ("next row alias", LineByte { row: 0, column: 6 }),
        ("interior mark", LineByte { row: 1, column: 2 }),
        ("past line", LineByte { row: 1, column: 4 }),
        ("past row", LineByte { row: 2, column: 0 }),
        (
            "huge row",
            LineByte {
                row: usize::MAX,
                column: 0,
            },
        ),
    ];
    for (name, point) in cases {
        assert_eq!(
            coordinates.from_line(point),
            Err(Error::Line(point)),
            "{name}"
        );
    }
    assert_eq!(
        coordinates.from_cell(Cell {
            row: usize::MAX,
            column: 0
        }),
        Err(Error::Line(LineByte {
            row: usize::MAX,
            column: 0
        }))
    );
}

// Flat segmentation is an oracle for context-fed segmentation at real rope seams.
#[test]
fn chunk_context() {
    let cases = [
        ("combining", format!("xa{}z", "\u{301}".repeat(2048))),
        (
            "emoji context",
            format!("x👩{}\u{200d}💻z", "\u{301}".repeat(1024)),
        ),
        ("regional parity", format!("x{}z", "\u{1f1f5}".repeat(769))),
        ("indic context", format!("x{}कz", "क्".repeat(700))),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text) in cases {
        let id = buffers.create(&text).unwrap();
        let buffer = buffers.get(id).unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        let mut bytes: Vec<usize> = text.grapheme_indices(true).map(|(byte, _)| byte).collect();
        bytes.push(text.len());
        let scalars: Vec<usize> = bytes
            .iter()
            .map(|byte| text[..*byte].chars().count())
            .collect();
        let span = buffer.span(0..buffer.len_chars()).unwrap();
        let mut offset = 0;
        let seams: Vec<usize> = buffer
            .chunks(&span)
            .unwrap()
            .map(|chunk| {
                offset += chunk.len();
                offset
            })
            .collect();
        assert!(
            seams.len() > 1,
            "{name}: fixture must contain multiple rope chunks"
        );
        assert!(
            seams.iter().any(|seam| !bytes.contains(seam)),
            "{name}: a grapheme must cross a rope seam"
        );
        for pair in scalars.windows(2) {
            assert_eq!(
                coordinates.next(Scalar(pair[0])).unwrap(),
                Scalar(pair[1]),
                "{name}: forward {}",
                pair[0]
            );
            assert_eq!(
                coordinates.previous(Scalar(pair[1])).unwrap(),
                Scalar(pair[0]),
                "{name}: backward {}",
                pair[1]
            );
        }
        for seam in seams {
            let scalar = coordinates.from_byte(Byte(seam)).unwrap();
            assert_eq!(
                coordinates.is_boundary(scalar).unwrap(),
                bytes.contains(&seam),
                "{name}: seam {seam}"
            );
        }
        let glyphs = coordinates
            .glyphs(0)
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let starts: Vec<usize> = glyphs.iter().map(|glyph| glyph.source.start.0).collect();
        assert_eq!(starts, scalars[..scalars.len() - 1], "{name}");
        let rendered: String = glyphs.iter().map(|glyph| glyph.text.as_ref()).collect();
        assert_eq!(rendered, text, "{name}");
    }
}

// Empty text has one logical line and valid BOF/EOF coordinates without glyphs.
#[test]
fn empty_coordinates() {
    let buffers = Buffers::new().unwrap();
    let coordinates = Coordinates::new(buffers.get(buffers.initial()).unwrap()).unwrap();
    let origin = LineByte { row: 0, column: 0 };
    assert_eq!(coordinates.to_byte(Scalar(0)).unwrap(), Byte(0));
    assert_eq!(coordinates.from_byte(Byte(0)).unwrap(), Scalar(0));
    assert_eq!(coordinates.to_line(Scalar(0)).unwrap(), origin);
    assert_eq!(coordinates.from_line(origin).unwrap(), Scalar(0));
    assert_eq!(
        coordinates.to_cell(Scalar(0)).unwrap(),
        Cell { row: 0, column: 0 }
    );
    assert_eq!(
        coordinates
            .from_cell(Cell {
                row: 0,
                column: 100
            })
            .unwrap(),
        Scalar(0)
    );
    let mut glyphs = coordinates.glyphs(0).unwrap();
    assert!(glyphs.next().is_none());
    assert!(glyphs.next().is_none());
}
