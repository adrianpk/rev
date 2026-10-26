#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::ops::Range;

use rev::buffer::{Buffer, Buffers, Error, Span, Stamp};

// Empty and loaded stores must retain exact UTF-8 text and separator counts.
#[test]
fn initial_store() {
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let buffer = buffers.get(initial).unwrap();
    assert_eq!(buffer.id(), initial);
    assert_eq!(
        (buffer.len_chars(), buffer.len_bytes(), buffer.len_lines()),
        (0, 0, 1)
    );
    assert_eq!(buffer.slice(&buffer.span(0..0).unwrap()).unwrap(), "");

    let cases = [
        ("empty", "", 0, 0, 1),
        ("unicode", "a🙂é", 3, 7, 1),
        ("multiline", "a\nβ\r\n", 5, 6, 3),
        ("final text", "first\nlast", 10, 10, 2),
    ];
    for (name, text, chars, bytes, lines) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert_eq!(
            (buffer.len_chars(), buffer.len_bytes(), buffer.len_lines()),
            (chars, bytes, lines),
            "{name}"
        );
        let span = buffer.span(0..chars).unwrap();
        assert_eq!(buffer.slice(&span).unwrap(), text, "{name}");
    }
    assert_eq!(buffers.get(initial).unwrap().len_chars(), 0);
}

// Scalar positions may split graphemes internally, but never UTF-8 scalars.
#[test]
fn scalar_spans() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a🙂e\u{301}").unwrap();
    let buffer = buffers.get(id).unwrap();
    let before = buffer.stamp();
    let cases = [
        ("start insertion", 0..0, Some("")),
        ("end insertion", 4..4, Some("")),
        ("emoji", 1..2, Some("🙂")),
        ("combining mark", 3..4, Some("\u{301}")),
        ("all text", 0..4, Some("a🙂e\u{301}")),
        ("reversed", Range { start: 3, end: 2 }, None),
        ("past end", 0..5, None),
        ("past insertion", 5..5, None),
        ("huge index", usize::MAX..usize::MAX, None),
    ];
    for (name, range, expected) in cases {
        let result = buffer.span(range.clone());
        match expected {
            Some(text) => assert_eq!(buffer.slice(&result.unwrap()).unwrap(), text, "{name}"),
            None => assert_eq!(
                result.unwrap_err(),
                Error::Bounds {
                    start: range.start,
                    end: range.end,
                    len: 4
                },
                "{name}",
            ),
        }
        assert_eq!(buffer.stamp(), before, "{name}");
    }
}

// Ropey's byte-to-char conversion rounds down; admission must reject that loss.
#[test]
fn byte_spans() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a🙂é").unwrap();
    let buffer = buffers.get(id).unwrap();
    let before = buffer.stamp();
    let cases = [
        ("start insertion", 0..0, Ok(0..0)),
        ("ascii", 0..1, Ok(0..1)),
        ("emoji", 1..5, Ok(1..2)),
        ("accent", 5..7, Ok(2..3)),
        ("all bytes", 0..7, Ok(0..3)),
        ("end insertion", 7..7, Ok(3..3)),
        ("middle start", 2..5, Err(Error::Boundary(2))),
        ("middle end", 1..4, Err(Error::Boundary(4))),
        ("middle insertion", 6..6, Err(Error::Boundary(6))),
        (
            "past end",
            0..8,
            Err(Error::Bounds {
                start: 0,
                end: 8,
                len: 7,
            }),
        ),
        (
            "reversed",
            Range { start: 5, end: 1 },
            Err(Error::Bounds {
                start: 5,
                end: 1,
                len: 7,
            }),
        ),
        (
            "huge index",
            usize::MAX..usize::MAX,
            Err(Error::Bounds {
                start: usize::MAX,
                end: usize::MAX,
                len: 7,
            }),
        ),
    ];
    for (name, range, expected) in cases {
        assert_eq!(
            buffer.byte_span(range).map(|span| span.range),
            expected,
            "{name}"
        );
        assert_eq!(buffer.stamp(), before, "{name}");
    }
}

// A prepared range cannot cross owners, revisions, or revalidated scalar bounds.
#[test]
fn qualified_spans() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a🙂é").unwrap();
    let other = buffers.create("a🙂é").unwrap();
    let registry = Buffers::new().unwrap();
    let buffer = buffers.get(id).unwrap();
    let span = buffer.span(0..3).unwrap();
    let stamp = span.stamp;
    let cases = [
        (
            "foreign buffer",
            Span {
                stamp: buffers.get(other).unwrap().stamp(),
                ..span.clone()
            },
            Error::Foreign {
                expected: id,
                actual: other,
            },
        ),
        (
            "foreign registry",
            Span {
                stamp: registry.get(registry.initial()).unwrap().stamp(),
                ..span.clone()
            },
            Error::Foreign {
                expected: id,
                actual: registry.initial(),
            },
        ),
        (
            "stale revision",
            Span {
                stamp: Stamp {
                    revision: 0,
                    ..stamp
                },
                ..span.clone()
            },
            Error::Stale {
                expected: stamp.revision,
                actual: 0,
            },
        ),
        (
            "future revision",
            Span {
                stamp: Stamp {
                    revision: u64::MAX,
                    ..stamp
                },
                ..span.clone()
            },
            Error::Stale {
                expected: stamp.revision,
                actual: u64::MAX,
            },
        ),
        (
            "past end",
            Span { stamp, range: 0..4 },
            Error::Bounds {
                start: 0,
                end: 4,
                len: 3,
            },
        ),
        (
            "reversed",
            Span {
                stamp,
                range: Range { start: 2, end: 1 },
            },
            Error::Bounds {
                start: 2,
                end: 1,
                len: 3,
            },
        ),
        (
            "huge index",
            Span {
                stamp,
                range: 0..usize::MAX,
            },
            Error::Bounds {
                start: 0,
                end: usize::MAX,
                len: 3,
            },
        ),
    ];
    for (name, invalid, expected) in cases {
        assert_eq!(buffer.slice(&invalid).unwrap_err(), expected, "{name}");
        assert_eq!(
            buffer.chunks(&invalid).map(|_| ()).unwrap_err(),
            expected,
            "{name}"
        );
        assert_eq!(buffer.stamp(), stamp, "{name}");
        assert_eq!(buffer.slice(&span).unwrap(), "a🙂é", "{name}");
    }
}

// A bounded read must span real rope chunks while retaining CRLF and UTF-8 bytes.
#[test]
fn bounded_reads() {
    let text = "αβ🙂\r\n".repeat(4096);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&text).unwrap();
    let buffer = buffers.get(id).unwrap();
    let span = buffer.span(127..9001).unwrap();
    let expected: String = text.chars().skip(127).take(9001 - 127).collect();
    let fragments: Vec<&str> = buffer.chunks(&span).unwrap().collect();
    assert!(
        fragments.len() > 1,
        "fixture must exercise multiple rope chunks"
    );
    assert_eq!(fragments.concat(), expected);
    assert_eq!(buffer.slice(&span).unwrap(), expected.as_str());

    let cases = [
        ("start line", 0, 0..3, "αβ🙂"),
        ("middle fragment", 2000, 1..3, "β🙂"),
        ("separator", 2000, 3..5, "\r\n"),
        ("line insertion", 2000, 5..5, ""),
        ("final empty line", 4096, 0..0, ""),
    ];
    for (name, line, range, expected) in cases {
        assert_eq!(buffer.line(line, range).unwrap(), expected, "{name}");
    }
    assert_eq!(
        buffer.line(4097, 0..0).unwrap_err(),
        Error::Line {
            index: 4097,
            len: 4097
        }
    );
    assert_eq!(
        buffer.line(0, 0..6).unwrap_err(),
        Error::Bounds {
            start: 0,
            end: 6,
            len: 5
        }
    );
}

// Shared handles borrow one owner across map growth; registry replacement cannot reuse IDs.
#[test]
fn shared_owner() {
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let id = buffers.create("shared 🙂").unwrap();
    let handles = [id, id];
    let left: &Buffer = buffers.get(handles[0]).unwrap();
    let right: &Buffer = buffers.get(handles[1]).unwrap();
    let span = left.span(0..left.len_chars()).unwrap();
    assert_eq!(left.stamp(), right.stamp());
    assert_eq!(left.slice(&span).unwrap(), right.slice(&span).unwrap());

    let mut identities = BTreeSet::from([initial, id]);
    for _ in 0..128 {
        assert!(identities.insert(buffers.create("other").unwrap()));
    }
    assert_eq!(buffers.initial(), initial);
    assert_eq!(buffers.get(id).unwrap().slice(&span).unwrap(), "shared 🙂");
    assert_eq!(buffers.get(initial).unwrap().len_chars(), 0);

    drop(buffers);
    let replacement = Buffers::new().unwrap();
    assert_ne!(replacement.initial(), initial);
    assert_eq!(replacement.get(id).err(), Some(Error::Missing(id)));
}
