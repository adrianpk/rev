#![forbid(unsafe_code)]

use std::io;

use rev::buffer::{Buffer, Buffers};
use rev::file::{self, Loaded};

fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn bytes(buffer: &Buffer) -> Vec<u8> {
    let mut bytes = Vec::new();
    file::write(buffer, &mut bytes).unwrap();
    bytes
}

// Valid ingress installs one fresh clean owner, separates only a leading BOM,
// and serializes exactly the original UTF-8/separator/final-newline bytes.
#[test]
fn loaded_text() {
    let cases = [
        ("empty", "", "", false, "\n"),
        ("LF", "a\nb\n", "a\nb\n", false, "\n"),
        ("CRLF", "a\r\nb\r\n", "a\r\nb\r\n", false, "\r\n"),
        (
            "mixed",
            "a\r\nb\nc\r\nlast",
            "a\r\nb\nc\r\nlast",
            false,
            "\r\n",
        ),
        ("tie", "a\r\nb\n", "a\r\nb\n", false, "\n"),
        (
            "Unicode",
            "e\u{301}界👩\u{200d}💻",
            "e\u{301}界👩\u{200d}💻",
            false,
            "\n",
        ),
        ("BOM", "\u{feff}a\r\nb", "a\r\nb", true, "\r\n"),
        ("BOM only", "\u{feff}", "", true, "\n"),
        ("double BOM", "\u{feff}\u{feff}x", "\u{feff}x", true, "\n"),
        ("inner BOM", "a\u{feff}b", "a\u{feff}b", false, "\n"),
        ("visible controls", "a\rb\x1b", "a\rb\x1b", false, "\n"),
    ];
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    for (name, source, body, bom, newline) in cases {
        let loaded = Loaded::read(source.as_bytes()).unwrap();
        let id = buffers.load(loaded).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(text(buffer), body, "{name}");
        assert_eq!(buffer.has_bom(), bom, "{name}");
        assert_eq!(buffer.newline(), newline, "{name}");
        assert_eq!(buffer.len_bytes(), body.len(), "{name}");
        assert_eq!(buffer.len_chars(), body.chars().count(), "{name}");
        assert_eq!(
            buffer.len_lines(),
            body.bytes().filter(|b| *b == b'\n').count() + 1,
            "{name}"
        );
        assert_eq!(buffer.stamp().revision, 1, "{name}");
        assert!(!buffer.is_modified(), "{name}");
        assert!(
            buffer.history().is_empty() && buffer.redo_history().is_empty(),
            "{name}"
        );
        assert_eq!(bytes(buffer), source.as_bytes(), "{name}");
    }
    assert_eq!(text(buffers.get(initial).unwrap()), "");
}

// Invalid UTF-8 and any NUL are errors, including after a recognized BOM;
// no lossy string or partially decoded candidate is returned to a publisher.
#[test]
fn rejected_text() {
    let cases: [(&str, &[u8], &str); 6] = [
        ("invalid byte", b"\xff", "UTF-8"),
        ("truncated scalar", b"\xe2\x82", "UTF-8"),
        ("overlong scalar", b"\xc0\xaf", "UTF-8"),
        ("BOM then invalid", b"\xef\xbb\xbf\xff", "UTF-8"),
        ("embedded NUL", b"a\0b", "NUL"),
        ("BOM then NUL", b"\xef\xbb\xbf\0", "NUL"),
    ];
    for (name, source, message) in cases {
        let error = Loaded::read(source)
            .err()
            .expect("invalid input must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{name}");
        assert!(error.to_string().contains(message), "{name}: {error}");
    }
}
