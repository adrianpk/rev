#![forbid(unsafe_code)]

#[path = "editing/kills.rs"]
mod kills;
#[path = "editing/linear.rs"]
mod linear;
#[path = "editing/regions.rs"]
mod regions;
#[path = "editing/yank.rs"]
mod yank;

use rev::buffer::edit::{Cursor, Error, Point, Replacement};
use rev::buffer::{Buffer, Buffers, Span, Stamp};
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

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    text: String,
    stamp: Stamp,
    mark: Option<Scalar>,
    read_only: bool,
    newline: &'static str,
    history: usize,
    points: Vec<Point>,
    state: rev::buffer::State,
    dirty: bool,
    redo: usize,
}

fn snapshot(buffer: &Buffer, points: &[Point]) -> Snapshot {
    Snapshot {
        text: text(buffer),
        stamp: buffer.stamp(),
        mark: buffer.mark(),
        read_only: buffer.is_read_only(),
        newline: buffer.newline(),
        history: buffer.history().len(),
        points: points.to_vec(),
        state: buffer.state(),
        dirty: buffer.is_modified(),
        redo: buffer.redo_history().len(),
    }
}

// Admission rejects the complete batch, even when its first replacement is
// valid. A nonempty prior history and stored positions must survive unchanged.
#[test]
fn rejected_ranges() {
    use rev::buffer::Error as Source;
    let mut buffers = Buffers::new().unwrap();
    let foreign = buffers.create("foreign").unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let prepared = buffer
        .prepare(&[replacement(buffer, 0..0, "abcdef")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    let points = [point(buffer, 2), point(buffer, 5)];
    let stamp = buffer.stamp();
    let make = |range| Replacement {
        source: Span { stamp, range },
        text: "Ω".into(),
    };
    let stale = Replacement {
        source: Span {
            stamp: Stamp {
                revision: stamp.revision - 1,
                ..stamp
            },
            range: 2..3,
        },
        text: "!".into(),
    };
    let other = Replacement {
        source: Span {
            stamp: Stamp {
                buffer: foreign,
                ..stamp
            },
            range: 2..3,
        },
        text: "!".into(),
    };
    let cases = [
        ("empty batch", vec![], Error::Empty),
        ("overlap", vec![make(1..4), make(3..5)], Error::Order),
        ("descending", vec![make(3..4), make(1..2)], Error::Order),
        ("same insertion", vec![make(1..1), make(1..1)], Error::Order),
        (
            "reversed",
            vec![make(0..1), make(std::ops::Range { start: 4, end: 2 })],
            Error::Source(Source::Bounds {
                start: 4,
                end: 2,
                len: 6,
            }),
        ),
        (
            "outside",
            vec![make(0..1), make(6..usize::MAX)],
            Error::Source(Source::Bounds {
                start: 6,
                end: usize::MAX,
                len: 6,
            }),
        ),
        (
            "stale second",
            vec![make(0..1), stale],
            Error::Source(Source::Stale {
                expected: stamp.revision,
                actual: stamp.revision - 1,
            }),
        ),
        (
            "foreign second",
            vec![make(0..1), other],
            Error::Source(Source::Foreign {
                expected: stamp.buffer,
                actual: foreign,
            }),
        ),
    ];
    for (name, edits, expected) in cases {
        let before = snapshot(buffer, &points);
        assert_eq!(
            buffer.prepare(&edits, &points, None).err(),
            Some(expected),
            "{name}"
        );
        assert_eq!(snapshot(buffer, &points), before, "{name}");
    }
}

// Cursor validation occurs after the candidate replacement and position map;
// its failure is a recoverable preparation failure, with nothing to roll back.
#[test]
fn failed_preparation() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}界").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(2))).unwrap();
    let points = [point(buffer, 2)];
    let edits = [replacement(buffer, 0..2, "replacement")];
    for (name, cursor) in [
        (
            "missing initiator",
            Cursor {
                index: 1,
                position: Scalar(0),
            },
        ),
        (
            "invalid result point",
            Cursor {
                index: 0,
                position: Scalar(usize::MAX),
            },
        ),
    ] {
        let before = snapshot(buffer, &points);
        assert!(
            buffer.prepare(&edits, &points, Some(cursor)).is_err(),
            "{name}"
        );
        assert_eq!(snapshot(buffer, &points), before, "{name}");
    }
    for (name, invalid) in [
        ("inside grapheme", point(buffer, 1)),
        ("outside source", point(buffer, 4)),
        (
            "stale point",
            Point {
                stamp: Stamp {
                    revision: 0,
                    ..buffer.stamp()
                },
                position: Scalar(0),
            },
        ),
    ] {
        let before = snapshot(buffer, &[invalid]);
        assert!(buffer.prepare(&edits, &[invalid], None).is_err(), "{name}");
        assert_eq!(snapshot(buffer, &[invalid]), before, "{name}");
    }
    let before = snapshot(buffer, &points);
    assert!(buffer.set_mark(Some(Scalar(1))).is_err());
    assert_eq!(snapshot(buffer, &points), before);
}

// Read-only admission is enforced both before preparation and immediately
// before publication; a prepared candidate confers no lasting write authority.
#[test]
fn read_only() {
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0)];
    let edits = [replacement(buffer, 0..0, "new")];
    let prepared = buffer.prepare(&edits, &points, None).unwrap();
    buffer.set_read_only(true);
    let before = snapshot(buffer, &points);
    assert_eq!(
        buffer.prepare(&edits, &points, None).err(),
        Some(Error::ReadOnly)
    );
    assert_eq!(buffer.publish(prepared, &mut points), Err(Error::ReadOnly));
    assert_eq!(snapshot(buffer, &points), before);
}

// Publication cannot overwrite a subsequent edit or live point/mark movement.
// A foreign candidate is rejected even when its scalar ranges would fit.
#[test]
fn changed_sources() {
    let mut buffers = Buffers::new().unwrap();
    let other = buffers.create("abc").unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0)];
    let edits = [replacement(buffer, 0..0, "abc")];
    let stale = buffer.prepare(&edits, &points, None).unwrap();
    let current = buffer.prepare(&edits, &points, None).unwrap();
    buffer.publish(current, &mut points).unwrap();
    let before = snapshot(buffer, &points);
    assert!(matches!(
        buffer.publish(stale, &mut points),
        Err(Error::Source(rev::buffer::Error::Stale { .. }))
    ));
    assert_eq!(snapshot(buffer, &points), before);
    for name in ["point", "mark", "point count"] {
        let prepared = buffer
            .prepare(&[replacement(buffer, 1..2, "X")], &points, None)
            .unwrap();
        let mut live = points.to_vec();
        match name {
            "point" => live[0].position = Scalar(0),
            "mark" => buffer.set_mark(Some(Scalar(1))).unwrap(),
            _ => live.clear(),
        }
        let before = snapshot(buffer, &live);
        assert_eq!(
            buffer.publish(prepared, &mut live),
            Err(Error::ChangedPositions),
            "{name}"
        );
        assert_eq!(snapshot(buffer, &live), before, "{name}");
    }
    let prepared = buffer
        .prepare(&[replacement(buffer, 0..0, "foreign")], &[], None)
        .unwrap();
    let buffer = buffers.get_mut(other).unwrap();
    let before = snapshot(buffer, &[]);
    assert!(matches!(
        buffer.publish(prepared, &mut []),
        Err(Error::Source(rev::buffer::Error::Foreign { .. }))
    ));
    assert_eq!(snapshot(buffer, &[]), before);
}

// Mapping uses insertion affinity and then repairs newly joined graphemes,
// including control separators, combining text, flags and emoji ZWJ sequences.
#[test]
fn position_affinity() {
    let cases = [
        ("insertion", "ab", 1..1, "XY", "aXYb", 3, 1),
        ("replacement", "abcd", 1..3, "XY", "aXYd", 3, 1),
        ("deletion", "abcd", 1..3, "", "ad", 1, 1),
        (
            "combining insertion",
            "ab",
            1..1,
            "\u{301}",
            "a\u{301}b",
            2,
            0,
        ),
        ("combining join", "aX\u{301}b", 1..2, "", "a\u{301}b", 2, 0),
        ("CRLF join", "\rX\n", 1..2, "", "\r\n", 2, 0),
        (
            "emoji join",
            "👩X\u{200d}💻",
            1..2,
            "",
            "👩\u{200d}💻",
            3,
            0,
        ),
        ("flag join", "🇺X🇸", 1..2, "", "🇺🇸", 2, 0),
    ];
    for (name, source, range, insert, expected, point_end, mark_end) in cases {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_mark(Some(Scalar(1))).unwrap();
        let mut points = [point(buffer, 1)];
        let prepared = buffer
            .prepare(&[replacement(buffer, range, insert)], &points, None)
            .unwrap();
        buffer.publish(prepared, &mut points).unwrap();
        assert_eq!(text(buffer), expected, "{name}");
        assert_eq!(points[0].position, Scalar(point_end), "{name}");
        assert_eq!(buffer.mark(), Some(Scalar(mark_end)), "{name}");
    }
}

// Adjacent base ranges and an insertion at a removed range's end must preserve
// endpoint affinity. Surviving points on either side shift by the exact delta.
#[test]
fn adjacent_ranges() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("abcd").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let edits = [
        replacement(buffer, 0..1, "XX"),
        replacement(buffer, 1..1, "Y"),
        replacement(buffer, 2..4, ""),
    ];
    let mut points: Vec<_> = (0..=4).map(|position| point(buffer, position)).collect();
    buffer.set_mark(Some(Scalar(1))).unwrap();
    let prepared = buffer.prepare(&edits, &points, None).unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(text(buffer), "XXYb");
    assert_eq!(
        points
            .iter()
            .map(|point| point.position.0)
            .collect::<Vec<_>>(),
        [2, 3, 4, 4, 4]
    );
    assert_eq!(buffer.mark(), Some(Scalar(2)));
}

// A join at a real rope seam uses complete grapheme context after publication,
// without converting the document into a second mutable string store.
#[test]
fn chunk_join() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"界".repeat(6000)).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let span = buffer.span(0..buffer.len_chars()).unwrap();
    let seam = buffer
        .chunks(&span)
        .unwrap()
        .next()
        .unwrap()
        .chars()
        .count();
    assert!(seam > 0 && seam < buffer.len_chars());
    buffer.set_mark(Some(Scalar(seam))).unwrap();
    let mut points = [point(buffer, seam), point(buffer, buffer.len_chars())];
    let prepared = buffer
        .prepare(&[replacement(buffer, seam..seam, "\u{301}")], &points, None)
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(points[0].position, Scalar(seam + 1));
    assert_eq!(points[1].position, Scalar(6001));
    assert_eq!(buffer.mark(), Some(Scalar(seam - 1)));
    assert_eq!(
        buffer
            .slice(&buffer.span(seam - 1..seam + 2).unwrap())
            .unwrap()
            .to_string(),
        "界\u{301}界"
    );
}

// Retention drops whole oldest operations and keeps the newest reversible
// fragments and monotonically increasing publication stamp intact.
#[test]
fn retained_records() {
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    for _ in 0..515 {
        let end = buffer.len_chars();
        let prepared = buffer
            .prepare(&[replacement(buffer, end..end, "x")], &[], None)
            .unwrap();
        buffer.publish(prepared, &mut []).unwrap();
    }
    assert_eq!(buffer.len_chars(), 515);
    assert_eq!(buffer.stamp().revision, 516);
    assert_eq!(buffer.history().len(), 512);
    assert_eq!(buffer.history().front().unwrap().before.revision, 4);
    let last = buffer.history().back().unwrap();
    assert_eq!(last.after, buffer.stamp());
    assert_eq!(last.changes()[0].before(), 514..514);
    assert_eq!(last.changes()[0].inserted().to_string(), "x");
    buffer.mark_saved(buffer.stamp()).unwrap();
    for _ in 0..512 {
        buffer
            .navigate(rev::buffer::edit::History::Undo, &mut [], None)
            .unwrap();
    }
    assert_eq!(text(buffer), "xxx");
    assert!(buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert_eq!(buffer.redo_history().len(), 512);
    let before = snapshot(buffer, &[]);
    assert!(
        buffer
            .navigate(rev::buffer::edit::History::Undo, &mut [], None)
            .is_err()
    );
    assert_eq!(snapshot(buffer, &[]), before);
    for _ in 0..512 {
        buffer
            .navigate(rev::buffer::edit::History::Redo, &mut [], None)
            .unwrap();
    }
    assert_eq!(buffer.len_chars(), 515);
    assert_eq!(buffer.stamp().revision, 1540);
    assert!(!buffer.is_modified());
    assert!(buffer.redo_history().is_empty());
}

fn editor(source: &str, position: usize, read_only: bool) -> rev::session::Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_read_only(read_only);
    let mut view = rev::view::View::new(buffer);
    view.point.position = Scalar(position);
    rev::session::Session::new(
        buffers,
        view,
        rev::display::Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

fn press(code: crossterm::event::KeyCode) -> crossterm::event::Event {
    use crossterm::event::{Event, KeyEvent, KeyModifiers};
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn chord(character: char) -> crossterm::event::Event {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    Event::Key(KeyEvent::new(
        KeyCode::Char(character),
        KeyModifiers::CONTROL,
    ))
}

// Actual session events enter the canonical rope without a mode switch and
// produce a mapped cursor, dirty status, visible tabs, and a new logical line.
#[test]
fn direct_input() {
    use crossterm::event::KeyCode;
    use rev::display::Position;
    use rev::session::Action;
    let mut session = editor("", 0, false);
    for character in "é界e\u{301}".chars() {
        assert_eq!(
            session.handle(press(KeyCode::Char(character))),
            Action::Redraw
        );
    }
    for code in [KeyCode::Tab, KeyCode::Enter, KeyCode::Char('x')] {
        assert_eq!(session.handle(press(code)), Action::Redraw);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(text(buffer), "é界e\u{301}\t\nx");
    assert_eq!(buffer.history().len(), 4);
    assert_eq!(buffer.stamp().revision, 8);
    assert_eq!(session.view().point.position, Scalar(7));
    assert_eq!(session.view().point.stamp, buffer.stamp());
    let frame = session.frame().unwrap();
    assert_eq!(frame.cursor(), Some(Position { column: 1, row: 1 }));
    assert!(frame.rows()[0].text().starts_with("é界e\u{301}    "));
    assert!(frame.rows()[1].text().starts_with('x'));
    assert!(frame.rows()[6].text().contains("** RW L2 C2 Text"));
}

// Bound commands remove entire preceding/following graphemes and CRLF pairs.
// Empty boundaries do not publish an empty transaction or move the point.
#[test]
fn bound_deletion() {
    use crossterm::event::KeyCode;
    let cases = [
        (
            "back combining",
            "e\u{301}界",
            2,
            press(KeyCode::Backspace),
            "界",
            0,
            1,
        ),
        (
            "back emoji",
            "a👩🏽\u{200d}💻b",
            5,
            press(KeyCode::Backspace),
            "ab",
            1,
            1,
        ),
        (
            "forward emoji",
            "a👩🏽\u{200d}💻b",
            1,
            press(KeyCode::Delete),
            "ab",
            1,
            1,
        ),
        (
            "back CRLF",
            "a\r\nb",
            3,
            press(KeyCode::Backspace),
            "ab",
            1,
            1,
        ),
        ("forward CRLF", "a\r\nb", 1, chord('d'), "ab", 1, 1),
        ("back flag", "🇵🇱x", 2, press(KeyCode::Backspace), "x", 0, 1),
        (
            "empty beginning",
            "",
            0,
            press(KeyCode::Backspace),
            "",
            0,
            0,
        ),
        ("empty end", "", 0, chord('d'), "", 0, 0),
    ];
    for (name, source, position, event, expected, point, records) in cases {
        let mut session = editor(source, position, false);
        session.handle(event);
        let buffer = session.buffer().unwrap();
        assert_eq!(text(buffer), expected, "{name}");
        assert_eq!(session.view().point.position, Scalar(point), "{name}");
        assert_eq!(buffer.history().len(), records, "{name}");
        assert_eq!(buffer.stamp().revision, 1 + records as u64, "{name}");
    }
}

// Every mutating binding reaches the read-only admission boundary, preserving
// canonical bytes, history, positions, and newline metadata on rejection.
#[test]
fn readonly_input() {
    use crossterm::event::KeyCode;
    use rev::session::Action;
    for (name, event) in [
        ("text", press(KeyCode::Char('界'))),
        ("newline", press(KeyCode::Enter)),
        ("tab", press(KeyCode::Tab)),
        ("backward", press(KeyCode::Backspace)),
        ("delete key", press(KeyCode::Delete)),
        ("delete binding", chord('d')),
    ] {
        let mut session = editor("aé\r\n界", 2, true);
        let before = snapshot(session.buffer().unwrap(), &[session.view().point]);
        assert_eq!(session.handle(event), Action::Redraw, "{name}");
        assert_eq!(session.echo(), "buffer is read-only", "{name}");
        assert_eq!(
            snapshot(session.buffer().unwrap(), &[session.view().point]),
            before,
            "{name}"
        );
        let frame = session.frame().unwrap();
        assert!(
            frame.rows()[6].text().contains("-- RO L1 C3 Text"),
            "{name}"
        );
    }
}

fn phase(
    mut event: crossterm::event::Event,
    kind: crossterm::event::KeyEventKind,
) -> crossterm::event::Event {
    if let crossterm::event::Event::Key(key) = &mut event {
        key.kind = kind;
    }
    event
}

// Text, separators and deletion can repeat in editing focus; their releases
// never publish again. Prompt Enter still has its separate confirmation rule.
#[test]
fn editing_phases() {
    use crossterm::event::{
        KeyCode,
        KeyEventKind::{Release, Repeat},
    };
    use rev::session::Action;
    let cases = [
        ("text", "", 0, press(KeyCode::Char('é')), "éé", 1),
        ("newlines", "", 0, press(KeyCode::Enter), "\n\n", 2),
        ("tabs", "", 0, press(KeyCode::Tab), "\t\t", 2),
        ("backward", "abc", 3, press(KeyCode::Backspace), "a", 2),
        ("forward", "abc", 0, chord('d'), "c", 2),
    ];
    for (name, source, position, event, expected, groups) in cases {
        let mut session = editor(source, position, false);
        assert_eq!(session.handle(event.clone()), Action::Redraw, "{name}");
        assert_eq!(
            session.handle(phase(event.clone(), Repeat)),
            Action::Redraw,
            "{name}"
        );
        assert_eq!(
            session.handle(phase(event, Release)),
            Action::Wait,
            "{name}"
        );
        assert_eq!(text(session.buffer().unwrap()), expected, "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), groups, "{name}");
    }
}

// Unsupported modified/control input, non-key events, and a canceled nested
// prefix cannot leak into the document. One resumed ordinary key publishes once.
#[test]
fn isolated_input() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    let mut session = editor("original", 8, false);
    let before = snapshot(session.buffer().unwrap(), &[session.view().point]);
    let events = [
        chord('q'),
        chord('z'),
        Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT)),
        Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL,
        )),
        press(KeyCode::Char('\0')),
        press(KeyCode::Char('\u{1b}')),
        press(KeyCode::F(1)),
        Event::Paste("\u{18}\u{3}".into()),
        Event::Resize(0, 0),
        chord('x'),
        press(KeyCode::Char('r')),
        chord('g'),
        chord('x'),
        press(KeyCode::Char('q')),
    ];
    for event in events {
        session.handle(event);
        assert_eq!(
            snapshot(session.buffer().unwrap(), &[session.view().point]),
            before
        );
    }
    session.handle(press(KeyCode::Char('界')));
    assert_eq!(text(session.buffer().unwrap()), "original界");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(session.view().point.position, Scalar(9));
}

// A name and its key sequence enter exactly the same production handler,
// including deletion and the guarded quit command, with equivalent state.
#[test]
fn named_edits() {
    use crossterm::event::KeyCode;
    use rev::command::Command;
    let cases = [
        ("newline", vec![press(KeyCode::Enter)]),
        ("insert-tab", vec![press(KeyCode::Tab)]),
        ("backward-delete-char", vec![press(KeyCode::Backspace)]),
        ("delete-char", vec![chord('d')]),
        ("keyboard-quit", vec![chord('g')]),
        ("kill-terminal", vec![chord('x'), chord('c')]),
    ];
    for (name, events) in cases {
        let mut named = editor("aé\r\n界", 2, false);
        let mut bound = editor("aé\r\n界", 2, false);
        let action = named.execute(Command::named(name).unwrap());
        let mut actions = Vec::new();
        for event in events {
            actions.push(bound.handle(event));
        }
        assert_eq!(actions.last(), Some(&action), "{name}");
        assert_eq!(
            text(named.buffer().unwrap()),
            text(bound.buffer().unwrap()),
            "{name}"
        );
        assert_eq!(
            named.view().point.position,
            bound.view().point.position,
            "{name}"
        );
        assert_eq!(
            named.buffer().unwrap().history().len(),
            bound.buffer().unwrap().history().len(),
            "{name}"
        );
        if action == rev::session::Action::Redraw {
            assert_eq!(named.echo(), bound.echo(), "{name}");
        }
    }
}

// Returning to equal bytes by a distinct edit state does not silently discard
// unsaved work. A clean buffer may quit; every dirty session retains its state.
#[test]
fn quit_guard() {
    use crossterm::event::KeyCode;
    use rev::session::Action;
    for (name, input, expected) in [
        ("clean", vec![], Action::Exit),
        ("dirty", vec![press(KeyCode::Char('x'))], Action::Redraw),
        (
            "equal bytes",
            vec![press(KeyCode::Char('x')), press(KeyCode::Backspace)],
            Action::Redraw,
        ),
    ] {
        let mut session = editor("", 0, false);
        for event in input {
            session.handle(event);
        }
        let before = snapshot(session.buffer().unwrap(), &[session.view().point]);
        session.handle(chord('x'));
        assert_eq!(session.handle(chord('c')), expected, "{name}");
        assert_eq!(
            snapshot(session.buffer().unwrap(), &[session.view().point]),
            before,
            "{name}"
        );
        if expected == Action::Redraw {
            assert_eq!(session.echo(), "Unsaved changes; cannot quit", "{name}");
        }
    }
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let prepared = buffer
        .prepare(&[replacement(buffer, 0..0, "unsaved")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let clean = buffers.create("clean").unwrap();
    let view = rev::view::View::new(buffers.get(clean).unwrap());
    let mut session = rev::session::Session::new(
        buffers,
        view,
        rev::display::Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    assert_eq!(session.execute(rev::command::Command::Quit), Action::Redraw);
    assert_eq!(text(session.buffer().unwrap()), "clean");
    assert!(session.echo().starts_with("Unsaved changes"));
}

fn prevailing(text: &str) -> &'static str {
    let crlf = text.matches("\r\n").count();
    let lf = text.matches('\n').count() - crlf;
    if crlf > lf { "\r\n" } else { "\n" }
}

// Enter preserves existing bytes and inserts the prevailing current style,
// choosing LF for ties and new text rather than normalizing the entire buffer.
#[test]
fn prevailing_input() {
    use crossterm::event::KeyCode;
    for source in [
        "",
        "abc",
        "a\r\nb",
        "a\r\nb\r\nc\nd",
        "a\nb\nc\r\nd",
        "a\nb\r\nc",
    ] {
        let position = source.chars().count();
        let mut session = editor(source, position, false);
        let newline = prevailing(source);
        assert_eq!(session.buffer().unwrap().newline(), newline);
        session.handle(press(KeyCode::Enter));
        assert_eq!(
            text(session.buffer().unwrap()),
            format!("{source}{newline}")
        );
        assert_eq!(
            session.view().point.position,
            Scalar(position + newline.chars().count())
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}

// A whole-text oracle on small fixtures checks incremental seam accounting for
// every scalar replacement range, including creation and splitting of CRLF.
#[test]
fn newline_accounting() {
    let mut buffers = Buffers::new().unwrap();
    for source in [
        "",
        "\r",
        "\n",
        "\r\n",
        "\rX\n",
        "\r\n\n",
        "\n\r\n",
        "é\rX\n界",
    ] {
        let len = source.chars().count();
        let ranges = (0..=len).flat_map(|start| (start..=len).map(move |end| start..end));
        for range in ranges {
            for insert in ["", "x", "\r", "\n", "\r\n"] {
                let id = buffers.create(source).unwrap();
                let buffer = buffers.get_mut(id).unwrap();
                let prepared = buffer
                    .prepare(&[replacement(buffer, range.clone(), insert)], &[], None)
                    .unwrap();
                buffer.publish(prepared, &mut []).unwrap();
                let current = text(buffer);
                assert_eq!(
                    buffer.newline(),
                    prevailing(&current),
                    "source={source:?}, range={range:?}, insert={insert:?}"
                );
            }
        }
    }
}

// Neighbor windows of adjacent base edits overlap; counting them independently
// would corrupt newline metadata even though the final rope bytes are correct.
#[test]
fn compound_newlines() {
    let cases = [
        (
            "several joins",
            "\rX\nY\rZ\n",
            vec![(1..2, ""), (3..4, ""), (5..6, "")],
            "\r\n\r\n",
        ),
        (
            "adjacent removals",
            "\r\n",
            vec![(0..1, ""), (1..2, "")],
            "",
        ),
        (
            "joined insertion",
            "X\n",
            vec![(0..1, "\r"), (2..2, "\n")],
            "\r\n\n",
        ),
    ];
    for (name, source, ranges, expected) in cases {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let edits: Vec<_> = ranges
            .into_iter()
            .map(|(range, text)| replacement(buffer, range, text))
            .collect();
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        assert_eq!(buffer.newline(), prevailing(source), "{name}");
        buffer.publish(prepared, &mut []).unwrap();
        assert_eq!(text(buffer), expected, "{name}");
        assert_eq!(buffer.newline(), prevailing(expected), "{name}");
    }
}

// A candidate that would create CRLF cannot publish its metadata early, either
// on late cursor failure, cancellation, or read-only rejection at publication.
#[test]
fn newline_rollback() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("\rX\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [point(buffer, 1)];
    let edits = [replacement(buffer, 1..2, "")];
    let before = snapshot(buffer, &points);
    assert!(
        buffer
            .prepare(
                &edits,
                &points,
                Some(Cursor {
                    index: 0,
                    position: Scalar(usize::MAX)
                })
            )
            .is_err()
    );
    assert_eq!(snapshot(buffer, &points), before);
    drop(buffer.prepare(&edits, &points, None).unwrap());
    assert_eq!(snapshot(buffer, &points), before);
    let prepared = buffer.prepare(&edits, &points, None).unwrap();
    buffer.set_read_only(true);
    let before = snapshot(buffer, &points);
    assert_eq!(buffer.publish(prepared, &mut points), Err(Error::ReadOnly));
    assert_eq!(snapshot(buffer, &points), before);
    buffer.set_read_only(false);
    let prepared = buffer.prepare(&edits, &points, None).unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(text(buffer), "\r\n");
    assert_eq!(buffer.newline(), "\r\n");
    assert_eq!(points[0].position, Scalar(2));
}

fn history(session: &mut rev::session::Session, name: &str) {
    let command = rev::command::Command::named(name).expect("implemented history command");
    assert_eq!(session.execute(command), rev::session::Action::Redraw);
    assert!(session.echo().is_empty(), "{name}: {}", session.echo());
    assert!(session.frame().unwrap().cursor().is_some());
}

// Undo/redo restore the initiating cursor even after motion, but publish fresh
// revisions while returning to the original logical state and clean marker.
#[test]
fn history_cycle() {
    use rev::command::Command;
    let mut session = editor("α\r\n界", 1, false);
    let initial = session.buffer().unwrap().state();
    session.insert('!');
    let edited = session.buffer().unwrap().state();
    let stale = session.buffer().unwrap().span(0..1).unwrap();
    session.execute(Command::named("end-of-buffer").unwrap());
    history(&mut session, "undo");
    let buffer = session.buffer().unwrap();
    assert_eq!(text(buffer), "α\r\n界");
    assert_eq!(buffer.state(), initial);
    assert_eq!(buffer.stamp().revision, 3);
    assert!(!buffer.is_modified());
    assert!(buffer.slice(&stale).is_err());
    assert_eq!(session.view().point.position, Scalar(1));
    history(&mut session, "undo-redo");
    let buffer = session.buffer().unwrap();
    assert_eq!(text(buffer), "α!\r\n界");
    assert_eq!(buffer.state(), edited);
    assert_eq!(buffer.stamp().revision, 4);
    assert!(buffer.is_modified());
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(buffer.history().len(), 1);
    assert!(buffer.redo_history().is_empty());
}

// Save acknowledgment preserves retained history and targets an exact revision;
// later undo can revisit that clean logical state despite a newer revision.
#[test]
fn saved_history() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let foreign = buffers.create("foreign").unwrap();
    let foreign_stamp = buffers.get(foreign).unwrap().stamp();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let prepared = buffer
        .prepare(&[replacement(buffer, 0..0, "saved")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let saved_stamp = buffer.stamp();
    let saved_state = buffer.state();
    buffer.mark_saved(saved_stamp).unwrap();
    assert_eq!(buffer.history().len(), 1);
    assert!(!buffer.is_modified());
    let prepared = buffer
        .prepare(&[replacement(buffer, 5..5, "dirty")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "saved");
    assert_eq!(buffer.state(), saved_state);
    assert_ne!(buffer.stamp(), saved_stamp);
    assert!(!buffer.is_modified());
    assert_eq!(buffer.redo_history().len(), 1);
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "saveddirty");
    assert!(buffer.is_modified());
    assert!(buffer.mark_saved(saved_stamp).is_err());
    assert!(buffer.mark_saved(foreign_stamp).is_err());
    assert!(buffer.is_modified());
}

// Repeated traversal must consume/reapply existing transactions rather than
// recording undo as another edit or losing the initiating cursor at either end.
#[test]
fn history_walk() {
    let mut session = editor("", 0, false);
    session.insert('é');
    session.execute(rev::command::Command::KeyboardQuit);
    session.insert('界');
    let cases = [
        ("undo", "é", 1, true, 1, 1),
        ("undo", "", 0, false, 0, 2),
        ("undo-redo", "é", 1, true, 1, 1),
        ("undo-redo", "é界", 2, true, 2, 0),
        ("undo", "é", 1, true, 1, 1),
        ("undo-redo", "é界", 2, true, 2, 0),
    ];
    for (index, (name, source, cursor, dirty, undo, redo)) in cases.into_iter().enumerate() {
        history(&mut session, name);
        let buffer = session.buffer().unwrap();
        assert_eq!(text(buffer), source);
        assert_eq!(buffer.stamp().revision, 4 + index as u64);
        assert_eq!(buffer.is_modified(), dirty);
        assert_eq!(
            (buffer.history().len(), buffer.redo_history().len()),
            (undo, redo)
        );
        assert_eq!(session.view().point.position, Scalar(cursor));
    }
    let before = snapshot(session.buffer().unwrap(), &[session.view().point]);
    session.execute(rev::command::Command::named("undo-redo").unwrap());
    assert_eq!(session.echo(), "Nothing to redo");
    assert_eq!(
        snapshot(session.buffer().unwrap(), &[session.view().point]),
        before
    );
}

// A new edit after undo invalidates redo only when published. Even equal bytes
// receive a distinct logical identity and cannot reuse a discarded save point.
#[test]
fn redo_branch() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    for character in ["a", "b"] {
        let end = buffer.len_chars();
        let prepared = buffer
            .prepare(&[replacement(buffer, end..end, character)], &[], None)
            .unwrap();
        buffer.publish(prepared, &mut []).unwrap();
    }
    let abandoned_state = buffer.state();
    buffer.mark_saved(buffer.stamp()).unwrap();
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    let prepared = buffer
        .prepare(&[replacement(buffer, 1..1, "ignored")], &[], None)
        .unwrap();
    drop(prepared);
    assert_eq!(buffer.redo_history().len(), 1);
    assert_eq!(text(buffer), "a");
    let prepared = buffer
        .prepare(&[replacement(buffer, 1..1, "b")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(text(buffer), "ab");
    assert_ne!(buffer.state(), abandoned_state);
    assert!(buffer.is_modified());
    assert!(buffer.redo_history().is_empty());
    let before = snapshot(buffer, &[]);
    assert_eq!(
        buffer.navigate(History::Redo, &mut [], None).unwrap_err(),
        Error::History(History::Redo)
    );
    assert_eq!(snapshot(buffer, &[]), before);
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "a");
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "ab");
    assert!(buffer.is_modified());
}

// The selected live initiator receives its recorded cursor; other live points
// map from their current positions, even when their array order has changed.
#[test]
fn history_positions() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("ab界cd").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_mark(Some(Scalar(2))).unwrap();
    let mut points = [point(buffer, 0), point(buffer, 3), point(buffer, 5)];
    let prepared = buffer
        .prepare(
            &[replacement(buffer, 1..3, "👩\u{200d}💻")],
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(4),
            }),
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(buffer.mark(), Some(Scalar(1)));
    points[0].position = Scalar(6);
    points[1].position = Scalar(5);
    points[2].position = Scalar(0);
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "ab界cd");
    assert_eq!(
        points.map(|p| p.position),
        [Scalar(0), Scalar(4), Scalar(0)]
    );
    assert_eq!(buffer.mark(), Some(Scalar(2)));
    points.swap(0, 2);
    buffer
        .navigate(History::Redo, &mut points, Some(2))
        .unwrap();
    assert_eq!(text(buffer), "a👩\u{200d}💻cd");
    assert_eq!(
        points.map(|p| p.position),
        [Scalar(0), Scalar(5), Scalar(4)]
    );
    assert!(points.iter().all(|p| p.stamp == buffer.stamp()));
    assert_eq!(buffer.mark(), Some(Scalar(1)));
}

// Compound inverses can contain equal insertion starts after adjacent deletions;
// replay must preserve their order and incremental newline accounting exactly.
#[test]
fn compound_history() {
    use rev::buffer::edit::History;
    let cases = [
        (
            "adjacent deletion",
            "abcd",
            vec![(0..1, ""), (1..2, "")],
            "cd",
        ),
        ("CRLF seams", "a\r\nb", vec![(1..2, ""), (2..3, "")], "ab"),
        (
            "mixed replacement",
            "α\r\nx界yz",
            vec![(0..1, ""), (3..5, "e\u{301}")],
            "\r\ne\u{301}yz",
        ),
    ];
    let mut buffers = Buffers::new().unwrap();
    for (name, source, changes, target) in cases {
        let id = buffers.create(source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let edits: Vec<_> = changes
            .into_iter()
            .map(|(range, text)| replacement(buffer, range, text))
            .collect();
        let prepared = buffer.prepare(&edits, &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        for _ in 0..3 {
            buffer.navigate(History::Undo, &mut [], None).unwrap();
            assert_eq!(text(buffer), source, "{name}");
            assert_eq!(buffer.newline(), prevailing(source), "{name}");
            assert!(!buffer.is_modified(), "{name}");
            buffer.navigate(History::Redo, &mut [], None).unwrap();
            assert_eq!(text(buffer), target, "{name}");
            assert_eq!(buffer.newline(), prevailing(target), "{name}");
            assert_eq!(buffer.history().len(), 1, "{name}");
        }
    }
}

fn history_fixture(direction: rev::buffer::edit::History) -> (Buffers, Vec<Point>) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}\r\n界").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = vec![point(buffer, 5)];
    let prepared = buffer
        .prepare(
            &[replacement(buffer, 5..5, "!")],
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(6),
            }),
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    if direction == rev::buffer::edit::History::Redo {
        buffer
            .navigate(rev::buffer::edit::History::Undo, &mut points, Some(0))
            .unwrap();
    }
    (buffers, points)
}

// Foreign, stale, interior and out-of-range points and unavailable initiators
// reject the whole replay. Read-only undo/redo preserve both history futures.
#[test]
fn invalid_history() {
    use rev::buffer::edit::History;
    for direction in [History::Undo, History::Redo] {
        let (mut buffers, mut points) = history_fixture(direction);
        let foreign = buffers.get(buffers.initial()).unwrap().stamp();
        let buffer = buffers.get_mut(points[0].stamp.buffer).unwrap();
        let stamp = buffer.stamp();
        for (name, invalid) in [
            (
                "foreign",
                Point {
                    stamp: foreign,
                    position: Scalar(0),
                },
            ),
            (
                "stale",
                Point {
                    stamp: Stamp {
                        revision: stamp.revision - 1,
                        ..stamp
                    },
                    position: Scalar(0),
                },
            ),
            ("cluster interior", point(buffer, 1)),
            ("past EOF", point(buffer, 999)),
        ] {
            let mut supplied = [invalid];
            let before = snapshot(buffer, &supplied);
            assert!(
                buffer.navigate(direction, &mut supplied, Some(0)).is_err(),
                "{name}"
            );
            assert_eq!(snapshot(buffer, &supplied), before, "{name}");
        }
        let before = snapshot(buffer, &points);
        assert_eq!(
            buffer
                .navigate(direction, &mut points, Some(99))
                .unwrap_err(),
            Error::Cursor(99)
        );
        assert_eq!(snapshot(buffer, &points), before);
        buffer.set_read_only(true);
        let before = snapshot(buffer, &points);
        assert_eq!(
            buffer
                .navigate(direction, &mut points, Some(0))
                .unwrap_err(),
            Error::ReadOnly
        );
        assert_eq!(snapshot(buffer, &points), before);
    }
}

// Buffers with coincident local sequence numbers still have distinct logical
// identities and independent undo/redo queues and saved-state markers.
#[test]
fn independent_history() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.initial();
    let second = buffers.create("other").unwrap();
    for id in [first, second] {
        let buffer = buffers.get_mut(id).unwrap();
        let end = buffer.len_chars();
        let prepared = buffer
            .prepare(&[replacement(buffer, end..end, "!")], &[], None)
            .unwrap();
        buffer.publish(prepared, &mut []).unwrap();
    }
    assert_ne!(
        buffers.get(first).unwrap().state(),
        buffers.get(second).unwrap().state()
    );
    let other = snapshot(buffers.get(second).unwrap(), &[]);
    buffers
        .get_mut(first)
        .unwrap()
        .navigate(History::Undo, &mut [], None)
        .unwrap();
    assert_eq!(snapshot(buffers.get(second).unwrap(), &[]), other);
    buffers
        .get_mut(second)
        .unwrap()
        .navigate(History::Undo, &mut [], None)
        .unwrap();
    assert_eq!(text(buffers.get(second).unwrap()), "other");
    assert!(!buffers.get(second).unwrap().is_modified());
    buffers
        .get_mut(first)
        .unwrap()
        .navigate(History::Redo, &mut [], None)
        .unwrap();
    assert_eq!(text(buffers.get(first).unwrap()), "!");
    assert!(buffers.get(first).unwrap().is_modified());
    assert_eq!(buffers.get(second).unwrap().redo_history().len(), 1);
}

// Each required reportable undo binding reaches the named handler; releases
// cannot consume history while a repeated direct undo can consume one record.
#[test]
fn history_keys() {
    use crossterm::event::{KeyCode, KeyEventKind};
    let cases = [
        ("Control slash", vec![chord('/')]),
        ("Control underscore", vec![chord('_')]),
        (
            "Control X suffix",
            vec![chord('x'), press(KeyCode::Char('u'))],
        ),
    ];
    for (name, events) in cases {
        let mut session = editor("e\u{301}", 2, false);
        session.insert('!');
        for event in events {
            session.handle(event);
        }
        assert_eq!(text(session.buffer().unwrap()), "e\u{301}", "{name}");
        assert_eq!(session.view().point.position, Scalar(2), "{name}");
        assert!(!session.buffer().unwrap().is_modified(), "{name}");
        assert!(session.echo().is_empty(), "{name}");
    }
    let mut session = editor("", 0, false);
    session.insert('a');
    assert_eq!(
        session.handle(phase(chord('/'), KeyEventKind::Release)),
        rev::session::Action::Wait
    );
    assert_eq!(text(session.buffer().unwrap()), "a");
    session.handle(phase(chord('/'), KeyEventKind::Repeat));
    assert_eq!(text(session.buffer().unwrap()), "");
    session.execute(rev::command::Command::named("undo").unwrap());
    assert_eq!(session.echo(), "Nothing to undo");
}

// Chunked history fragments include a combining cluster spanning rope seams;
// repeated replay must restore exact UTF-8 bytes and whole initiating cursors.
#[test]
fn history_chunks() {
    use rev::buffer::edit::History;
    let source = format!("{}e{}🇵🇱\r\n", "α".repeat(1200), "\u{301}".repeat(1300));
    let target = format!("{}\n", "界".repeat(2000));
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [point(buffer, 0)];
    let prepared = buffer
        .prepare(
            &[replacement(buffer, 0..buffer.len_chars(), &target)],
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(target.chars().count()),
            }),
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert!(
        buffer.history().back().unwrap().changes()[0]
            .removed()
            .chunks()
            .count()
            > 2
    );
    for _ in 0..4 {
        buffer
            .navigate(History::Undo, &mut points, Some(0))
            .unwrap();
        assert_eq!(text(buffer), source);
        assert_eq!(points[0].position, Scalar(0));
        assert_eq!(buffer.newline(), "\r\n");
        assert!(!buffer.is_modified());
        buffer
            .navigate(History::Redo, &mut points, Some(0))
            .unwrap();
        assert_eq!(text(buffer), target);
        assert_eq!(points[0].position, Scalar(target.chars().count()));
        assert_eq!(buffer.newline(), "\n");
    }
}

// Revisiting the same logical text does not revive an older prepared revision.
// Rejection must preserve the restored text, positions and both history queues.
#[test]
fn prepared_history() {
    use rev::buffer::edit::History;
    let (mut buffers, mut points) = history_fixture(History::Undo);
    let buffer = buffers.get_mut(points[0].stamp.buffer).unwrap();
    let end = buffer.len_chars();
    let stale = buffer
        .prepare(&[replacement(buffer, end..end, "?")], &points, None)
        .unwrap();
    let logical = buffer.state();
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    buffer
        .navigate(History::Redo, &mut points, Some(0))
        .unwrap();
    assert_eq!(buffer.state(), logical);
    let before = snapshot(buffer, &points);
    assert!(buffer.publish(stale, &mut points).is_err());
    assert_eq!(snapshot(buffer, &points), before);
}

// Adjacent printable scalars, including combining and emoji components, form
// one retained undo unit while every accepted key still publishes a revision.
#[test]
fn typing_group() {
    let mut session = editor("", 0, false);
    let source = "ab e\u{301}👩\u{200d}💻";
    for character in source.chars() {
        session.insert(character);
    }
    let buffer = session.buffer().unwrap();
    assert_eq!(text(buffer), source);
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.stamp().revision, 1 + source.chars().count() as u64);
    assert_eq!(buffer.history().back().unwrap().changes().len(), 1);
    history(&mut session, "undo");
    assert_eq!(text(session.buffer().unwrap()), "");
    assert_eq!(session.view().point.position, Scalar(0));
    assert!(!session.buffer().unwrap().is_modified());
    history(&mut session, "undo-redo");
    assert_eq!(text(session.buffer().unwrap()), source);
    assert_eq!(
        session.view().point.position,
        Scalar(source.chars().count())
    );
}

// Navigation breaks typing continuity even within an existing word. Undo first
// removes the new insertion, then the whole earlier typing group.
#[test]
fn typing_command() {
    let mut session = editor("", 0, false);
    session.insert('a');
    session.insert('b');
    session.handle(chord('b'));
    session.insert('X');
    assert_eq!(text(session.buffer().unwrap()), "aXb");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    history(&mut session, "undo");
    assert_eq!(text(session.buffer().unwrap()), "ab");
    assert_eq!(session.view().point.position, Scalar(1));
    history(&mut session, "undo");
    assert_eq!(text(session.buffer().unwrap()), "");
    assert!(!session.buffer().unwrap().is_modified());
}

fn type_at(buffer: &mut Buffer, points: &mut [Point], character: char, index: usize) {
    let start = points[index].position.0;
    let mut bytes = [0; 4];
    let edit = replacement(buffer, start..start, character.encode_utf8(&mut bytes));
    let prepared = buffer
        .prepare_typing(
            &edit,
            points,
            Cursor {
                index,
                position: Scalar(start + 1),
            },
        )
        .unwrap();
    buffer.publish(prepared, points).unwrap();
}

// A command that leaves point unchanged still ends typing. Prefix handling,
// invalid input and resize must not silently join typing on either side.
#[test]
fn typing_breaks() {
    use crossterm::event::{Event, KeyCode};
    let cases = [
        ("boundary motion", vec![chord('f')]),
        ("away and back", vec![chord('b'), chord('f')]),
        ("cancel", vec![chord('g')]),
        (
            "invalid prefix",
            vec![chord('x'), press(KeyCode::Char('q'))],
        ),
        ("canceled prefix", vec![chord('x'), chord('g')]),
        ("unbound control", vec![chord('q')]),
        ("unbound function", vec![press(KeyCode::F(1))]),
        ("raw control", vec![press(KeyCode::Char('\0'))]),
        ("resize", vec![Event::Resize(40, 8)]),
    ];
    for (name, events) in cases {
        let mut session = editor("", 0, false);
        session.insert('a');
        session.insert('b');
        for event in events {
            session.handle(event);
        }
        session.insert('c');
        session.insert('d');
        assert_eq!(text(session.buffer().unwrap()), "abcd", "{name}");
        assert_eq!(session.buffer().unwrap().history().len(), 2, "{name}");
        history(&mut session, "undo");
        assert_eq!(text(session.buffer().unwrap()), "ab", "{name}");
        history(&mut session, "undo");
        assert_eq!(text(session.buffer().unwrap()), "", "{name}");
    }
}

// Key releases are ordinary typing lifecycle noise, not commands. Repeated
// printable input can extend the same coherent group without consuming one.
#[test]
fn typing_phases() {
    use crossterm::event::{KeyCode, KeyEventKind};
    let mut session = editor("", 0, false);
    session.insert('a');
    let event = press(KeyCode::Char('b'));
    assert_eq!(
        session.handle(phase(event.clone(), KeyEventKind::Release)),
        rev::session::Action::Wait
    );
    session.handle(event.clone());
    session.handle(phase(event, KeyEventKind::Repeat));
    assert_eq!(text(session.buffer().unwrap()), "abb");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    history(&mut session, "undo");
    assert_eq!(text(session.buffer().unwrap()), "");
}

// The original save boundary stays reachable as a whole group boundary. A save
// after preparation invalidates a proposed join before any state is published.
#[test]
fn typing_checkpoint() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0)];
    type_at(buffer, &mut points, 'a', 0);
    type_at(buffer, &mut points, 'b', 0);
    let edit = replacement(buffer, 2..2, "c");
    let prepared = buffer
        .prepare_typing(
            &edit,
            &points,
            Cursor {
                index: 0,
                position: Scalar(3),
            },
        )
        .unwrap();
    let saved = buffer.state();
    buffer.mark_saved(buffer.stamp()).unwrap();
    let before = snapshot(buffer, &points);
    assert_eq!(
        buffer.publish(prepared, &mut points).unwrap_err(),
        Error::ChangedPositions
    );
    assert_eq!(snapshot(buffer, &points), before);
    type_at(buffer, &mut points, 'c', 0);
    type_at(buffer, &mut points, 'd', 0);
    assert_eq!(buffer.history().len(), 2);
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "ab");
    assert_eq!(buffer.state(), saved);
    assert!(!buffer.is_modified());
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "");
    assert!(buffer.is_modified());
    buffer
        .navigate(History::Redo, &mut points, Some(0))
        .unwrap();
    assert!(!buffer.is_modified());
    buffer
        .navigate(History::Redo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "abcd");
    assert!(buffer.is_modified());
}

// Explicit context breaks after preparation reject a cached join even if text,
// mark, point and revision still compare equal to their prepared snapshots.
#[test]
fn prepared_group() {
    for name in ["explicit break", "same mark", "same access"] {
        let mut buffers = Buffers::new().unwrap();
        let buffer = buffers.get_mut(buffers.initial()).unwrap();
        let mut points = [point(buffer, 0)];
        type_at(buffer, &mut points, 'a', 0);
        let edit = replacement(buffer, 1..1, "b");
        let prepared = buffer
            .prepare_typing(
                &edit,
                &points,
                Cursor {
                    index: 0,
                    position: Scalar(2),
                },
            )
            .unwrap();
        match name {
            "explicit break" => buffer.break_typing(),
            "same mark" => buffer.set_mark(None).unwrap(),
            _ => buffer.set_read_only(false),
        }
        let before = snapshot(buffer, &points);
        assert_eq!(
            buffer.publish(prepared, &mut points).unwrap_err(),
            Error::ChangedPositions,
            "{name}"
        );
        assert_eq!(snapshot(buffer, &points), before, "{name}");
        type_at(buffer, &mut points, 'c', 0);
        assert_eq!(text(buffer), "ac", "{name}");
        assert_eq!(buffer.history().len(), 2, "{name}");
    }
}

// Canceled or invalid private candidates do not extend retained fragments or
// clear confirmed typing continuity; the next valid key joins only live text.
#[test]
fn canceled_group() {
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0)];
    type_at(buffer, &mut points, 'a', 0);
    let edit = replacement(buffer, 1..1, "b");
    let prepared = buffer
        .prepare_typing(
            &edit,
            &points,
            Cursor {
                index: 0,
                position: Scalar(2),
            },
        )
        .unwrap();
    drop(prepared);
    let invalid = Replacement {
        source: Span {
            stamp: buffer.stamp(),
            range: 99..99,
        },
        text: "x".into(),
    };
    let before = snapshot(buffer, &points);
    assert!(
        buffer
            .prepare_typing(
                &invalid,
                &points,
                Cursor {
                    index: 0,
                    position: Scalar(100)
                }
            )
            .is_err()
    );
    assert_eq!(snapshot(buffer, &points), before);
    type_at(buffer, &mut points, 'c', 0);
    assert_eq!(text(buffer), "ac");
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(
        buffer.history().back().unwrap().changes()[0]
            .inserted()
            .to_string(),
        "ac"
    );
}

// Changed initiators or non-initiating view positions end a group even when
// insertion offsets remain adjacent. No view inherits another cursor snapshot.
#[test]
fn typing_views() {
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0), point(buffer, 0)];
    type_at(buffer, &mut points, 'a', 0);
    type_at(buffer, &mut points, 'b', 1);
    type_at(buffer, &mut points, 'c', 1);
    assert_eq!(buffer.history().len(), 2);
    points[0].position = Scalar(0);
    type_at(buffer, &mut points, 'd', 1);
    assert_eq!(buffer.history().len(), 3);
    buffer.set_mark(None).unwrap();
    type_at(buffer, &mut points, 'e', 1);
    assert_eq!(buffer.history().len(), 4);
    assert_eq!(text(buffer), "abcde");
    buffer
        .navigate(rev::buffer::edit::History::Undo, &mut points, Some(1))
        .unwrap();
    assert_eq!(text(buffer), "abcd");
    assert_eq!(points.map(|p| p.position), [Scalar(0), Scalar(4)]);
}

// Multi-range replacement remains one complete undo unit next to typing.
// Typing after undo starts a fresh group and discards the compound redo future.
#[test]
fn compound_groups() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("abcdef").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [point(buffer, 0)];
    type_at(buffer, &mut points, 'x', 0);
    type_at(buffer, &mut points, 'y', 0);
    let edits = [
        replacement(buffer, 2..3, "A\n"),
        replacement(buffer, 5..7, "👩\u{200d}💻"),
    ];
    let prepared = buffer
        .prepare(
            &edits,
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(4),
            }),
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(text(buffer), "xyA\nbc👩\u{200d}💻f");
    assert_eq!(buffer.history().len(), 2);
    assert_eq!(buffer.history().back().unwrap().changes().len(), 2);
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "xyabcdef");
    type_at(buffer, &mut points, 'z', 0);
    type_at(buffer, &mut points, '!', 0);
    assert_eq!(text(buffer), "xyz!abcdef");
    assert_eq!(buffer.history().len(), 2);
    assert!(buffer.redo_history().is_empty());
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "xyabcdef");
    buffer
        .navigate(History::Redo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), "xyz!abcdef");
}

// A paste-sized or replacement payload enters through one ordinary transaction,
// preserving the untouched prefix/suffix and surviving one-step undo and redo.
#[test]
fn bulk_group() {
    use rev::buffer::edit::History;
    let source = "left\r\nold\r\nright";
    let payload = "é界\t\r\n".repeat(6000);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let mut points = [point(buffer, 6)];
    let edit = replacement(buffer, 6..9, &payload);
    let prepared = buffer
        .prepare(
            &[edit],
            &points,
            Some(Cursor {
                index: 0,
                position: Scalar(6 + payload.chars().count()),
            }),
        )
        .unwrap();
    buffer.publish(prepared, &mut points).unwrap();
    assert_eq!(text(buffer), format!("left\r\n{payload}\r\nright"));
    assert_eq!(buffer.history().len(), 1);
    buffer.mark_saved(buffer.stamp()).unwrap();
    buffer
        .navigate(History::Undo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), source);
    assert_eq!(points[0].position, Scalar(6));
    assert!(buffer.is_modified());
    buffer
        .navigate(History::Redo, &mut points, Some(0))
        .unwrap();
    assert_eq!(text(buffer), format!("left\r\n{payload}\r\nright"));
    assert!(!buffer.is_modified());
}

// The count bound applies to complete typing groups, not individual key edits.
// Deep traversal must stop at the retained base without inventing a clean state.
#[test]
fn retained_groups() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let mut points = [point(buffer, 0)];
    for _ in 0..515 {
        buffer.break_typing();
        type_at(buffer, &mut points, 'a', 0);
        type_at(buffer, &mut points, 'b', 0);
    }
    assert_eq!(buffer.len_chars(), 1030);
    assert_eq!(buffer.history().len(), 512);
    assert!(
        buffer
            .history()
            .iter()
            .all(|record| record.changes()[0].inserted().len_chars() == 2)
    );
    buffer.mark_saved(buffer.stamp()).unwrap();
    for _ in 0..512 {
        buffer
            .navigate(History::Undo, &mut points, Some(0))
            .unwrap();
    }
    assert_eq!(text(buffer), "ababab");
    assert_eq!(points[0].position, Scalar(6));
    assert!(buffer.is_modified());
    assert!(buffer.history().is_empty());
    for _ in 0..512 {
        buffer
            .navigate(History::Redo, &mut points, Some(0))
            .unwrap();
    }
    assert_eq!(buffer.len_chars(), 1030);
    assert_eq!(points[0].position, Scalar(1030));
    assert!(!buffer.is_modified());
}

// Exercise the actual 64 MiB soft target. The newest two-range operation exceeds
// it alone, yet remains whole and reversible after every older group is pruned.
#[test]
fn newest_group() {
    use rev::buffer::edit::History;
    let mut buffers = Buffers::new().unwrap();
    let buffer = buffers.get_mut(buffers.initial()).unwrap();
    let prepared = buffer
        .prepare(&[replacement(buffer, 0..0, "XY")], &[], None)
        .unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.mark_saved(buffer.stamp()).unwrap();
    let payload = "a".repeat(32 * 1024 * 1024 + 1);
    let edits = [
        replacement(buffer, 0..0, &payload),
        replacement(buffer, 1..1, &payload),
    ];
    let prepared = buffer.prepare(&edits, &[], None).unwrap();
    drop(edits);
    buffer.publish(prepared, &mut []).unwrap();
    assert_eq!(buffer.history().len(), 1);
    let record = buffer.history().back().unwrap();
    assert_eq!(record.changes().len(), 2);
    assert_eq!(
        record
            .changes()
            .iter()
            .map(|change| change.inserted().len_bytes())
            .sum::<usize>(),
        64 * 1024 * 1024 + 2
    );
    assert_eq!(buffer.len_chars(), 64 * 1024 * 1024 + 4);
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(text(buffer), "XY");
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(buffer.len_chars(), 64 * 1024 * 1024 + 4);
    assert_eq!(
        buffer
            .slice(&buffer.span(payload.len()..payload.len() + 1).unwrap())
            .unwrap()
            .to_string(),
        "X"
    );
    assert_eq!(
        buffer
            .slice(
                &buffer
                    .span(buffer.len_chars() - 1..buffer.len_chars())
                    .unwrap()
            )
            .unwrap()
            .to_string(),
        "Y"
    );
    assert!(buffer.is_modified());
}
