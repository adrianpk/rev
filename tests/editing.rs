#![forbid(unsafe_code)]

use rev::buffer::edit::{Cursor, Point, Replacement};
use rev::buffer::{Buffer, Buffers};
use rev::coordinates::Scalar;

fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn replacement(buffer: &Buffer, range: std::ops::Range<usize>, text: &str) -> Replacement {
    Replacement {
        source: buffer.span(range).unwrap(),
        text: text.into(),
    }
}

fn point(buffer: &Buffer, position: usize) -> Point {
    Point {
        stamp: buffer.stamp(),
        position: Scalar(position),
    }
}

// Unequal replacements use base offsets; untouched middle positions must not
// collapse into a coarse invalidation span, and the initiator overrides mapping.
#[test]
fn coherent_publication() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("αbc\r\n界def").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let before = buffer.stamp();
    buffer.set_mark(Some(Scalar(2))).unwrap();
    let edits = [
        replacement(buffer, 1..3, "😀"),
        replacement(buffer, 6..8, "XYZ"),
    ];
    let mut points = [point(buffer, 2), point(buffer, 5), point(buffer, 8)];
    let prepared = buffer
        .prepare(
            &edits,
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(0),
            }),
        )
        .unwrap();
    assert_eq!(text(buffer), "αbc\r\n界def");
    assert!(buffer.history().is_empty());
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(text(buffer), "α😀\r\n界XYZf");
    assert_eq!(buffer.stamp().revision, before.revision + 1);
    assert_eq!(
        points.map(|point| point.position),
        [Scalar(0), Scalar(4), Scalar(8)]
    );
    assert!(points.iter().all(|point| point.stamp == buffer.stamp()));
    assert_eq!(buffer.mark(), Some(Scalar(1)));
    let record = buffer.history().back().unwrap();
    assert_eq!((record.before, record.after), (before, buffer.stamp()));
    assert_eq!(record.before_points()[0].position, Scalar(2));
    assert_eq!(record.after_points(), &points);
    assert_eq!(
        (record.before_mark, record.after_mark),
        (Some(Scalar(2)), Some(Scalar(1)))
    );
    assert_eq!(record.changes()[0].removed().to_string(), "bc");
    assert_eq!(record.changes()[1].inserted().to_string(), "XYZ");
    assert_eq!(record.changes()[1].before(), 6..8);
    assert_eq!(record.changes()[1].after(), 5..8);
}

// A prepared rope remains private, and retained fragments can reconstruct the
// exact original bytes through the same checked owner, including CRLF and EOF.
#[test]
fn reversible_fragments() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("aé\r\n界").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edits = [
        replacement(buffer, 0..0, "Ω"),
        replacement(buffer, 2..5, "!"),
    ];
    let abandoned = buffer.prepare(&edits, &[], None).unwrap();
    drop(abandoned);
    assert_eq!(text(buffer), "aé\r\n界");
    assert!(buffer.history().is_empty());
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(text(buffer), "Ωaé!");
    let inverse: Vec<_> = buffer
        .history()
        .back()
        .unwrap()
        .changes()
        .iter()
        .map(|change| replacement(buffer, change.after(), &change.removed().to_string()))
        .collect();
    let prepared = buffer.prepare(&inverse, &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(text(buffer), "aé\r\n界");
    assert_eq!(buffer.history().len(), 2);
}
