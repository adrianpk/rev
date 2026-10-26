#![forbid(unsafe_code)]

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
    assert_eq!(buffer.history().len(), 7);
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
        ("text", "", 0, press(KeyCode::Char('é')), "éé"),
        ("newlines", "", 0, press(KeyCode::Enter), "\n\n"),
        ("tabs", "", 0, press(KeyCode::Tab), "\t\t"),
        ("backward", "abc", 3, press(KeyCode::Backspace), "a"),
        ("forward", "abc", 0, chord('d'), "c"),
    ];
    for (name, source, position, event, expected) in cases {
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
        assert_eq!(session.buffer().unwrap().history().len(), 2, "{name}");
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
            assert_eq!(
                session.echo(),
                "Unsaved changes; cannot quit before saving is available",
                "{name}"
            );
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
