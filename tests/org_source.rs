#![forbid(unsafe_code)]

#[path = "org_source/blocks.rs"]
mod blocks;
#[path = "org_source/incremental.rs"]
mod incremental;
#[path = "org_source/index.rs"]
mod index;
#[path = "org_source/inline.rs"]
mod inline;
#[path = "org_source/inline_bounds.rs"]
mod inline_bounds;
#[path = "org_source/jobs.rs"]
mod jobs;
#[path = "org_source/structure.rs"]
mod structure;

use rev::buffer::edit::{self, History, Point, Replacement};
use rev::buffer::{Buffer, Buffers, Error};
use rev::coordinates::{self, Byte, Scalar};
use rev::file::{self, Loaded};
use rev::org::{FormKind, Source};

fn bytes(buffer: &Buffer) -> Vec<u8> {
    let mut output = Vec::new();
    file::write(buffer, &mut output).unwrap();
    output
}

// Org byte locations must resolve to canonical scalar spans without normalizing
// combining characters, tabs, CRLF or the empty EOF insertion position.
#[test]
fn source_ranges() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}\t").unwrap();
    let buffer = buffers.get(id).unwrap();
    let source = Source::new(buffer);
    let cases = [
        ("BOF", 0..0, 0..0, ""),
        ("wide character", 1..4, 1..2, "界"),
        ("CRLF", 4..6, 2..4, "\r\n"),
        ("combining cluster", 6..9, 4..6, "e\u{301}"),
        ("tab", 9..10, 6..7, "\t"),
        ("EOF", 10..10, 7..7, ""),
    ];
    for (name, bytes, scalars, expected) in cases {
        let range = source.range(Byte(bytes.start)..Byte(bytes.end)).unwrap();
        assert_eq!(source.text(&range).unwrap().to_string(), expected, "{name}");
        assert_eq!(range.span().range, scalars, "{name}");
        assert_eq!(range.stamp(), buffer.stamp(), "{name}");
    }
    assert_eq!(source.stamp(), buffer.stamp());
    assert!(buffer.history().is_empty());
}

// Reject bad offsets before they can become a form or edit, including overflow-
// sized inputs and bytes inside a multibyte character.
#[test]
fn invalid_ranges() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("a界\r\ne\u{301}\t").unwrap();
    let source = Source::new(buffers.get(id).unwrap());
    let cases = [
        ("interior start", 2, 4, Error::Boundary(2)),
        ("interior end", 1, 3, Error::Boundary(3)),
        ("interior empty", 8, 8, Error::Boundary(8)),
        ("reversed", 7, 1, bounds(7, 1)),
        ("past EOF", 0, 11, bounds(0, 11)),
        ("past EOF empty", 11, 11, bounds(11, 11)),
        ("maximum offset", 0, usize::MAX, bounds(0, usize::MAX)),
    ];
    for (name, start, end, error) in cases {
        assert_eq!(
            source.range(Byte(start)..Byte(end)).unwrap_err(),
            error,
            "{name}"
        );
    }
}

fn bounds(start: usize, end: usize) -> Error {
    Error::Bounds {
        start,
        end,
        len: 10,
    }
}

// Recognizer-supplied classifications remain read-only observations. Unknown
// directives and malformed blocks serialize exactly, including BOM and EOF state.
#[test]
fn preserved_forms() {
    let cases = [
        ("heading", FormKind::Heading, "* Heading\r\n"),
        ("directive", FormKind::Directive, "#+TITLE: Example\n"),
        ("text", FormKind::Text, "e\u{301}\t界"),
        ("unknown", FormKind::Unknown, "#+FUTURE: untouched\r\n"),
        (
            "malformed block",
            FormKind::Malformed,
            "#+BEGIN_SRC sh\r\nprintf preserved\r\n",
        ),
        (
            "executable source",
            FormKind::Block,
            "#+BEGIN_SRC sh\nexit 99\n#+END_SRC\n",
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, kind, body) in cases {
        let original = format!("\u{feff}{body}");
        let id = buffers
            .load(Loaded::read(original.as_bytes()).unwrap())
            .unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        let before = (buffer.stamp(), buffer.state());
        let source = Source::new(buffer);
        let range = source.range(Byte(0)..Byte(body.len())).unwrap();
        let form = source.form(kind, range).unwrap();
        assert_eq!(form.kind(), kind, "{name}");
        assert_eq!(
            source.text(form.source()).unwrap().to_string(),
            body,
            "{name}"
        );
        assert_eq!(bytes(buffer), original.as_bytes(), "{name}");
        assert_eq!((buffer.stamp(), buffer.state()), before, "{name}");
        assert!(buffer.history().is_empty(), "{name}");
        assert!(!buffer.is_modified(), "{name}");
    }
}

// Identical text in another buffer cannot confer read, form or edit authority.
#[test]
fn foreign_ranges() {
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.create("* Same\n").unwrap();
    let second = buffers.create("* Same\n").unwrap();
    let range = Source::new(buffers.get(first).unwrap())
        .range(Byte(0)..Byte(6))
        .unwrap();
    let buffer = buffers.get(second).unwrap();
    let source = Source::new(buffer);
    let error = Error::Foreign {
        expected: second,
        actual: first,
    };
    assert_eq!(source.text(&range).unwrap_err(), error);
    assert_eq!(
        source.form(FormKind::Heading, range.clone()).unwrap_err(),
        error
    );
    let edits = [Replacement {
        source: range.span().clone(),
        text: "* Changed".into(),
    }];
    assert!(matches!(buffer.prepare(&edits, &[], None), Err(edit::Error::Source(e)) if e == error));
    assert_eq!(bytes(buffer), b"* Same\n");
}

// Editing a heading preserves unrelated unknown/malformed bytes. Saved ranges
// stay stale even after undo restores identical bytes at identical offsets.
#[test]
fn stale_observations() {
    let original = "\u{feff}* Title\r\n#+MYSTERY: λ\t \r\n#+BEGIN_SRC sh\r\nprintf preserved\r\n";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers
        .load(Loaded::read(original.as_bytes()).unwrap())
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let source = Source::new(buffer);
    let heading = source.range(Byte(2)..Byte(7)).unwrap();
    let range = source.range(Byte(9)..Byte(26)).unwrap();
    let form = source.form(FormKind::Unknown, range.clone()).unwrap();
    assert_eq!(
        source.text(form.source()).unwrap().to_string(),
        "#+MYSTERY: λ\t \r\n"
    );
    let edits = [Replacement {
        source: heading.span().clone(),
        text: "Revised".into(),
    }];
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(
        bytes(buffer),
        original.replacen("Title", "Revised", 1).as_bytes()
    );
    for name in ["after edit", "after undo"] {
        let source = Source::new(buffer);
        let error = Error::Stale {
            expected: buffer.stamp().revision,
            actual: range.stamp().revision,
        };
        assert_eq!(source.text(form.source()).unwrap_err(), error, "{name}");
        assert_eq!(
            source.form(FormKind::Unknown, range.clone()).unwrap_err(),
            error,
            "{name}"
        );
        assert!(
            matches!(buffer.prepare(&edits, &[], None), Err(edit::Error::Source(e)) if e == error),
            "{name}"
        );
        if name == "after edit" {
            buffer.navigate(History::Undo, &mut [], None).unwrap();
        }
    }
    assert_eq!(bytes(buffer), original.as_bytes());
    let source = Source::new(buffer);
    let fresh = source.range(Byte(9)..Byte(26)).unwrap();
    assert_eq!(
        source.text(&fresh).unwrap().to_string(),
        "#+MYSTERY: λ\t \r\n"
    );
}

// A parser range does not make an interior grapheme position a valid view point.
// Org edits retain canonical point admission and read-only protections.
#[test]
fn edit_authority() {
    let cases = [
        (
            "combining interior",
            "e\u{301}",
            1..3,
            false,
            Scalar(1),
            edit::Error::Coordinates(coordinates::Error::Cluster(Scalar(1))),
        ),
        (
            "CRLF interior",
            "\r\n",
            1..2,
            false,
            Scalar(1),
            edit::Error::Coordinates(coordinates::Error::Cluster(Scalar(1))),
        ),
        (
            "read-only source",
            "* Heading",
            2..9,
            true,
            Scalar(0),
            edit::Error::ReadOnly,
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, text, bytes, read_only, position, error) in cases {
        let id = buffers.create(text).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(read_only);
        let source = Source::new(buffer);
        let range = source.range(Byte(bytes.start)..Byte(bytes.end)).unwrap();
        let edits = [Replacement {
            source: range.span().clone(),
            text: String::new(),
        }];
        let points = [Point {
            stamp: source.stamp(),
            position,
        }];
        assert_eq!(
            buffer.prepare(&edits, &points, None).map(|_| ()),
            Err(error),
            "{name}"
        );
        assert_eq!(source.stamp(), range.stamp(), "{name}");
        assert_eq!(
            source
                .text(&source.range(Byte(0)..Byte(text.len())).unwrap())
                .unwrap()
                .to_string(),
            text,
            "{name}"
        );
        assert!(buffer.history().is_empty(), "{name}");
    }
}
