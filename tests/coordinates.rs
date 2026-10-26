#![forbid(unsafe_code)]

use rev::buffer::Buffers;
use rev::coordinates::{Byte, Cell, Coordinates, Error, LineByte, Scalar};

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
