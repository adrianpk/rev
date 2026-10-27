use rev::command::{Command, History, RectangleCommand};
use rev::coordinates::{Coordinates, Scalar};
use rev::session::{Action, Session};

use super::{cell, selected, text};

fn kill(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Kill)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle killed");
}

// All changed rows use one base space and one history group. Partial tabs leave
// only their unselected cells; whole Unicode clusters and row separators survive undo.
#[test]
fn atomic_rows() {
    let source = "012345\r\na\t界z\r\nQ👩\u{200d}💻e\u{301}X\r\nx\r\n\r\n012345";
    let mut session = selected(source, cell(0, 2), cell(5, 5));
    let original = session.view().point.position;
    let mark = session.buffer().unwrap().mark();
    let revision = session.buffer().unwrap().stamp().revision;
    kill(&mut session);
    let killed = "015\r\na z\r\nQ\r\nx\r\n\r\n015";
    assert_eq!(text(&session), killed);
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["234", "  界", "👩\u{200d}💻e\u{301}X", "", "", "234"]
    );
    assert_eq!(session.rectangle().unwrap().width(), 3);
    assert_eq!(session.view().point.position, Scalar(2));
    assert!(!session.buffer().unwrap().rectangle_active());
    assert_eq!(session.buffer().unwrap().stamp().revision, revision + 1);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()
            .len(),
        4
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, original);
    assert_eq!(session.buffer().unwrap().mark(), mark);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), killed);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(session.kill_ring().len(), 0);
}

// A partial leading tab on the top row leaves actual prefix spaces. The explicit
// initiating point follows them rather than inheriting a byte/scalar column.
#[test]
fn tab_cursor() {
    let source = "\txyz\n01234567";
    let mut session = selected(source, cell(0, 6), cell(1, 2));
    let before = session.view().point.position;
    kill(&mut session);
    assert_eq!(text(&session), "  z\n0167");
    assert_eq!(session.rectangle().unwrap().rows(), ["  xy", "2345"]);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_cell(Scalar(2))
            .unwrap(),
        cell(0, 2)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, before);
}

// The fixed-name handler and C-x r k reach the same real transaction; the key
// suffix is consumed and cannot insert a literal k or change the linear ring.
#[test]
fn command_routes() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    for named in [false, true] {
        let mut session = selected("abcdef\nabcdef", cell(0, 1), cell(1, 4));
        if named {
            session.execute(Command::ExecuteExtended);
            session.handle(Event::Paste("kill-rectangle".into()));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )));
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('r'),
                KeyModifiers::NONE,
            )));
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
            )));
        }
        assert_eq!(session.echo(), "Rectangle killed");
        assert_eq!(text(&session), "aef\naef");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert_eq!(session.kill_ring().len(), 0);
    }
    assert_eq!(
        Command::named("kill-rectangle"),
        Some(Command::Rectangle(RectangleCommand::Kill))
    );
}

// Copy expands complete tabs while kill retains them. Both keep selected partial
// cells as spaces; deletion materializes only unselected edges and leaves later
// tabs literal. Every multi-row deletion is one operation across undo and redo.
#[test]
fn tab_fragments() {
    let source = "0123456789ABCD\na\tb\tc\tz\n0123456789ABCD";
    let cases = [
        (2, 3, " ", " ", "a  b\tc\tz"),
        (1, 4, "   ", "\t", "ab\tc\tz"),
        (2, 6, "  b ", "  b ", "a   c\tz"),
        (0, 2, "a ", "a ", "  b\tc\tz"),
        (5, 7, "  ", "  ", "a\tb c\tz"),
        (7, 9, " c", " c", "a\tb  \tz"),
        (12, 14, "z", "z", "a\tb\tc\t"),
        (0, 9, "a   b   c", "a\tb\tc", "\tz"),
    ];
    for (left, right, copied, retained, remaining) in cases {
        let mut session = selected(source, cell(0, left), cell(2, right));
        super::copy(&mut session);
        assert_eq!(
            session.rectangle().unwrap().rows()[1],
            copied,
            "{left}..{right}"
        );
        assert_eq!(text(&session), source);
        kill(&mut session);
        assert_eq!(
            session.rectangle().unwrap().rows()[1],
            retained,
            "{left}..{right}"
        );
        assert_eq!(
            text(&session).split('\n').nth(1),
            Some(remaining),
            "{left}..{right}"
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session).split('\n').nth(1), Some(remaining));
    }
}

// Whole touched clusters and literal controls are removed consistently for
// reversed/crossed endpoints. Undo restores Unicode and original CRLF bytes.
#[test]
fn wide_rows() {
    let cases = [
        (
            "e\u{301}界👩\u{200d}💻z",
            2,
            4,
            "界👩\u{200d}💻",
            "e\u{301}z",
        ),
        ("a🇵🇱b", 2, 3, "🇵🇱", "ab"),
        ("a\r\x1bb", 2, 4, "\r\x1b", "ab"),
        ("\u{301}界", 0, 2, "\u{301}界", ""),
    ];
    for (middle, left, right, retained, remaining) in cases {
        let source = format!("0123456789\r\n{middle}\r\n0123456789");
        for reverse in [false, true] {
            let (a, b) = if reverse {
                (cell(2, left), cell(0, right))
            } else {
                (cell(0, left), cell(2, right))
            };
            let mut session = selected(&source, a, b);
            let before = session.view().point.position;
            let mark = session.buffer().unwrap().mark();
            kill(&mut session);
            assert_eq!(session.rectangle().unwrap().rows()[1], retained);
            assert_eq!(text(&session).split("\r\n").nth(1), Some(remaining));
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source);
            assert_eq!(session.view().point.position, before);
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert!(!session.buffer().unwrap().is_modified());
        }
    }
}

// The top-left can be at EOL on a short row with no edit. That row contributes
// an empty retained fragment, while later changes still share one history group.
#[test]
fn short_rows() {
    let source = "ab\n012345\nx\n012345";
    let mut session = selected(source, cell(0, 2), cell(3, 5));
    let before = session.view().point.position;
    kill(&mut session);
    assert_eq!(text(&session), "ab\n015\nx\n015");
    assert_eq!(session.rectangle().unwrap().rows(), ["", "234", "", "234"]);
    assert_eq!(session.view().point.position, Scalar(2));
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()
            .len(),
        2
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, before);
}

// All live views map through all row changes. Undo from a passive pane restores
// the original initiating cursor, while other points map from their current state.
#[test]
fn shared_mapping() {
    use rev::command::Split;
    let source = "abcdef\nabcdef\nabcdef\nend";
    let mut session = selected(source, cell(0, 2), cell(2, 5));
    let origin = session.windows().selected_id();
    session.execute(Command::Split(Split::Right));
    let inside = session.windows().order()[1];
    session.select_view(inside).unwrap();
    super::at(&mut session, cell(1, 3));
    session.execute(Command::Split(Split::Below));
    let outside = session.windows().order()[2];
    session.select_view(outside).unwrap();
    super::at(&mut session, cell(3, 2));
    session.frame().unwrap();
    session.select_view(origin).unwrap();
    kill(&mut session);
    assert_eq!(text(&session), "abf\nabf\nabf\nend");
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        Scalar(2)
    );
    assert_eq!(
        session.windows().get(inside).unwrap().point.position,
        Scalar(6)
    );
    assert_eq!(
        session.windows().get(outside).unwrap().point.position,
        Scalar(14)
    );
    session.select_view(outside).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        Scalar(19)
    );
    assert_eq!(
        session.windows().get(inside).unwrap().point.position,
        Scalar(12)
    );
    assert_eq!(
        session.windows().get(outside).unwrap().point.position,
        Scalar(23)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(2)));
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), "abf\nabf\nabf\nend");
    let coordinates = Coordinates::new(session.buffer().unwrap()).unwrap();
    for id in session.windows().order() {
        let point = session.windows().get(id).unwrap().point;
        assert_eq!(point.stamp, session.buffer().unwrap().stamp());
        assert!(coordinates.is_boundary(point.position).unwrap());
    }
    session.frame().unwrap();
}

// Read-only, inactive, zero-width and prompt-owned kills retain every source
// row and the previous block, even when the refused selection differs from it.
#[test]
fn refused_kills() {
    let source = "012345\na\t界z\n012345";
    let mut session = super::editor(source, cell(0, 2), cell(2, 5), true);
    super::copy(&mut session);
    super::at(&mut session, cell(2, 4));
    let before = session.view().point;
    let mark = session.buffer().unwrap().mark();
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.echo(), "buffer is read-only");
    assert!(session.buffer().unwrap().rectangle_active());
    assert_eq!(session.view().point, before);
    assert_eq!(session.buffer().unwrap().mark(), mark);
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.echo(), "Rectangle is not active");
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.echo(), "Rectangle is empty");
    session.execute(Command::FindFile);
    session.execute(Command::Rectangle(RectangleCommand::Kill));
    assert_eq!(session.echo(), "a prompt owns input focus");
    assert_eq!(session.rectangle().unwrap().width(), 3);
    assert_eq!(session.rectangle().unwrap().rows(), ["234", "  界", "234"]);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.kill_ring().len(), 0);
}

// Rejection of the final base row cannot publish preceding valid replacements.
// Publication also rechecks selection/read-only authority after preparation.
#[test]
fn invalid_rows() {
    use rev::buffer::Buffers;
    use rev::buffer::edit::{Error, Point, Replacement};
    let source = "abcdef\nabcdef\nabcdef";
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let foreign = buffers.get(buffers.initial()).unwrap().stamp();
    let buffer = buffers.get_mut(id).unwrap();
    let point = Point {
        stamp: buffer.stamp(),
        position: Scalar(19),
    };
    buffer
        .toggle_rectangle(Point {
            position: Scalar(1),
            ..point
        })
        .unwrap();
    for fault in ["foreign", "stale", "range", "order"] {
        let mut edits: Vec<_> = [1..4, 8..11, 15..18]
            .into_iter()
            .map(|range| Replacement {
                source: buffer.span(range).unwrap(),
                text: String::new(),
            })
            .collect();
        match fault {
            "foreign" => edits[2].source.stamp = foreign,
            "stale" => edits[2].source.stamp.revision += 1,
            "range" => edits[2].source.range.end = 99,
            "order" => edits[2].source.range = 8..9,
            _ => unreachable!(),
        }
        assert!(buffer.prepare(&edits, &[point], None).is_err(), "{fault}");
        assert_eq!(
            buffer
                .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                .unwrap(),
            source
        );
        assert_eq!(buffer.stamp(), point.stamp);
        assert_eq!(buffer.mark(), Some(Scalar(1)));
        assert!(buffer.rectangle_active());
        assert!(buffer.history().is_empty());
    }
    let edit = Replacement {
        source: buffer.span(1..4).unwrap(),
        text: String::new(),
    };
    let prepared = buffer
        .prepare(std::slice::from_ref(&edit), &[point], None)
        .unwrap();
    buffer.set_readonly(true);
    assert!(matches!(
        buffer.publish(prepared, &mut [point]),
        Err(Error::ReadOnly)
    ));
    buffer.set_readonly(false);
    let prepared = buffer.prepare(&[edit], &[point], None).unwrap();
    buffer.toggle_rectangle(point).unwrap();
    assert!(matches!(
        buffer.publish(prepared, &mut [point]),
        Err(Error::ChangedPositions)
    ));
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap(),
        source
    );
    assert!(buffer.history().is_empty());
}

// Thousands of mixed-width rows create one checked publication and one undo.
// Copy/history retain only their fragments; scrolling cannot limit affected rows.
#[test]
fn large_rows() {
    use crossterm::event::Event;
    let source = format!(
        "0123456789\n{}\n0123456789",
        vec!["a\t界e\u{301}👩\u{200d}💻Z"; 4096].join("\n")
    );
    let mut session = selected(&source, cell(0, 2), cell(4097, 8));
    session.handle(Event::Resize(5, 4));
    session.frame().unwrap();
    assert!(session.view().top > 0);
    let original = session.view().point.position;
    let before = session.buffer().unwrap().stamp().revision;
    kill(&mut session);
    let expected = format!("0189\n{}\n0189", vec!["a Z"; 4096].join("\n"));
    assert_eq!(text(&session), expected);
    assert_eq!(session.buffer().unwrap().stamp().revision, before + 1);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(
        session
            .buffer()
            .unwrap()
            .history()
            .back()
            .unwrap()
            .changes()
            .len(),
        4098
    );
    assert_eq!(session.rectangle().unwrap().rows().len(), 4098);
    assert_eq!(session.rectangle().unwrap().width(), 6);
    assert!(
        session.rectangle().unwrap().rows()[1..4097]
            .iter()
            .all(|row| row == "  界e\u{301}👩\u{200d}💻")
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert_eq!(session.view().point.position, original);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert!(session.frame().unwrap().cursor().is_some());
}

// A held/released nested suffix cannot run the multi-row edit a second time or
// insert k into its result. A later independent insertion gets its own undo group.
#[test]
fn terminal_phases() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    let mut session = selected("abcdef\nabcdef", cell(0, 1), cell(1, 4));
    for (code, modifiers) in [
        (KeyCode::Char('x'), KeyModifiers::CONTROL),
        (KeyCode::Char('r'), KeyModifiers::NONE),
        (KeyCode::Char('k'), KeyModifiers::NONE),
    ] {
        session.handle(Event::Key(KeyEvent::new(code, modifiers)));
    }
    for phase in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        key.kind = phase;
        session.handle(Event::Key(key));
    }
    assert_eq!(text(&session), "aef\naef");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.insert('!');
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "aef\naef");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "abcdef\nabcdef");
}
