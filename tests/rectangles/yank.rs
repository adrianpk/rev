use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::{Buffers, edit::Point};
use rev::command::{BufferCommand, Command, History, RectangleCommand};
use rev::coordinates::{Cell, Coordinates};
use rev::display::Size;
use rev::session::{Action, Session};
use rev::view::View;

use super::{cell, text};

const SOURCE: &str = "0AB9\nx界z\n\n0e\u{301}X9";

fn retained(source: &str, endpoints: (Cell, Cell), target: &str, read_only: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let source_id = buffers.create(source).unwrap();
    let target_id = buffers.create(target).unwrap();
    let target_buffer = buffers.get_mut(target_id).unwrap();
    target_buffer.set_readonly(read_only);
    let name = target_buffer.name().to_owned();
    let buffer = buffers.get_mut(source_id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let a = coordinates.from_cell(endpoints.0).unwrap();
    let b = coordinates.from_cell(endpoints.1).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: a,
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = b;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap();
    super::copy(&mut session);
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(Event::Paste(name));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(text(&session), target);
    session
}

fn yank(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Yank)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle yanked");
}

// One transaction inserts unequal row fragments, adds missing lines and only
// the necessary prefixes. Empty copied rows advance without synthetic padding.
#[test]
fn append_rows() {
    let mut session = retained(SOURCE, (cell(0, 1), cell(3, 3)), "....", false);
    super::at(&mut session, cell(0, 3));
    let before = session.view().point.position;
    yank(&mut session);
    let expected = "...AB.\n   界\n\n   e\u{301}X";
    assert_eq!(text(&session), expected);
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["AB", "界", "", "e\u{301}X"]
    );
    assert_eq!(session.rectangle().unwrap().width(), 2);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        cell(3, 5)
    );
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
        2
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "....");
    assert_eq!(session.view().point.position, before);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert_eq!(session.kill_ring().len(), 0);
}

// A target inside a wide/tab glyph snaps to its source start. Existing short
// rows pad only for nonempty fragments, while empty fragments leave rows intact.
#[test]
fn target_edges() {
    for (middle, inserted) in [("界Z", "界界Z"), ("\tZ", "界\tZ")] {
        let target = format!("abcde\n{middle}\n\tx\nq");
        let mut session = retained(SOURCE, (cell(0, 1), cell(3, 3)), &target, false);
        super::at(&mut session, cell(0, 1));
        yank(&mut session);
        assert_eq!(
            text(&session),
            format!("aABbcde\n{inserted}\n\tx\nqe\u{301}X")
        );
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            cell(3, 3)
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), target);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Bound and named entry share a real multi-row insertion. The separate retained
// block survives replay, and command suffixes cannot become document characters.
#[test]
fn command_routes() {
    for named in [false, true] {
        let mut session = retained("0AB9\n0CD9", (cell(0, 1), cell(1, 3)), "x\ny", false);
        if named {
            session.execute(Command::ExecuteExtended);
            session.handle(Event::Paste("yank-rectangle".into()));
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
                KeyCode::Char('y'),
                KeyModifiers::NONE,
            )));
        }
        assert_eq!(session.echo(), "Rectangle yanked");
        assert_eq!(text(&session), "ABx\nCDy");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "x\ny");
        assert_eq!(session.rectangle().unwrap().rows(), ["AB", "CD"]);
    }
    assert_eq!(
        Command::named("yank-rectangle"),
        Some(Command::Rectangle(RectangleCommand::Yank))
    );
}

// Existing separators stay exact; new rows follow the destination's prevailing
// style. Insertions sharing EOF merge instead of creating duplicate base ranges.
#[test]
fn eof_offsets() {
    let cases = [
        ("ab", cell(0, 2), "abAB\n  CD\n  EF"),
        ("ab\n", cell(0, 1), "aABb\n CD\n EF"),
        ("\r\n", cell(0, 0), "AB\r\nCD\r\nEF"),
        ("aa\r\nb\r\n", cell(1, 1), "aa\r\nbAB\r\n CD\r\n EF"),
        ("aa\r\nb\n", cell(1, 1), "aa\r\nbAB\n CD\n EF"),
        ("abc\nxyz\nend", cell(0, 1), "aABbc\nxCDyz\neEFnd"),
    ];
    for (target, point, expected) in cases {
        let mut session = retained("0AB9\n0CD9\n0EF9", (cell(0, 1), cell(2, 3)), target, false);
        super::at(&mut session, point);
        let before = session.view().point.position;
        yank(&mut session);
        assert_eq!(text(&session), expected, "{target:?}");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), target);
        assert_eq!(session.view().point.position, before);
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(text(&session), expected);
    }
}

// Copy-time short rows stay unpadded when inserted. Wide-cluster expansion and
// partial-tab spaces retain their actual lengths; complete tabs stay literal.
#[test]
fn copied_shapes() {
    let mut session = retained(
        "012345\nx\t界z\nq\n\n012345",
        (cell(0, 2), cell(4, 5)),
        "a\nb\nc\nd\ne",
        false,
    );
    super::at(&mut session, cell(0, 1));
    yank(&mut session);
    assert_eq!(text(&session), "a234\nb  界\nc\nd\ne234");
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["234", "  界", "", "", "234"]
    );
    assert_eq!(session.rectangle().unwrap().width(), 3);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "a\nb\nc\nd\ne");
    let mut session = retained(
        "012345\n\tZ\n012345",
        (cell(0, 0), cell(2, 4)),
        "a\nb\nc",
        false,
    );
    super::at(&mut session, cell(0, 1));
    yank(&mut session);
    assert_eq!(text(&session), "a0123\nb\t\nc0123");
    assert_eq!(session.rectangle().unwrap().rows()[1], "\t");
    assert_eq!(
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .to_cell(
                Coordinates::new(session.buffer().unwrap())
                    .unwrap()
                    .from_cell(cell(1, 99))
                    .unwrap()
            )
            .unwrap(),
        cell(1, 4)
    );
}

// A detached copied combining mark can join target text without changing its
// retained bytes. The final cursor is repaired at a complete grapheme boundary.
#[test]
fn unicode_joins() {
    let mut session = retained("XY\n\u{301}", (cell(0, 0), cell(1, 1)), "q\nab", false);
    super::at(&mut session, cell(0, 1));
    yank(&mut session);
    assert_eq!(text(&session), "qX\na\u{301}b");
    assert_eq!(session.rectangle().unwrap().rows(), ["X", "\u{301}"]);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        cell(1, 1)
    );
    assert!(
        Coordinates::new(session.buffer().unwrap())
            .unwrap()
            .is_boundary(session.view().point.position)
            .unwrap()
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "q\nab");
    assert!(!session.buffer().unwrap().is_modified());
}

// EOF points use right affinity across the complete appended tail; marks keep
// left affinity. Undo invoked in another pane restores the original initiator.
#[test]
fn shared_insertions() {
    use rev::command::{RegionCommand, Split};
    use rev::coordinates::Scalar;
    let mut session = retained("0AB9\n0CD9\n0EF9", (cell(0, 1), cell(2, 3)), "ab\nq", false);
    let origin = session.windows().selected_id();
    super::at(&mut session, cell(0, 2));
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    super::at(&mut session, cell(1, 1));
    session.select_view(origin).unwrap();
    yank(&mut session);
    assert_eq!(text(&session), "abAB\nq CD\n  EF");
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        Scalar(14)
    );
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        Scalar(14)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(2)));
    assert!(!session.buffer().unwrap().mark_active());
    session.select_view(peer).unwrap();
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "ab\nq");
    assert_eq!(
        session.windows().get(origin).unwrap().point.position,
        Scalar(2)
    );
    assert_eq!(
        session.windows().get(peer).unwrap().point.position,
        Scalar(4)
    );
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(2)));
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), "abAB\nq CD\n  EF");
    session.frame().unwrap();
    let coordinates = Coordinates::new(session.buffer().unwrap()).unwrap();
    for id in session.windows().order() {
        let point = session.windows().get(id).unwrap().point;
        assert_eq!(point.stamp, session.buffer().unwrap().stamp());
        assert!(coordinates.is_boundary(point.position).unwrap());
    }
}

// Empty store, read-only and focused-prompt admission cannot create rows or
// padding. A refused operation preserves point, text, history and retained data.
#[test]
fn refused_insertions() {
    let mut empty = Session::empty(Size {
        width: 48,
        height: 8,
    })
    .unwrap();
    empty.execute(Command::Rectangle(RectangleCommand::Yank));
    assert_eq!(empty.echo(), "No rectangle to yank");
    assert_eq!(text(&empty), "");
    assert!(empty.buffer().unwrap().history().is_empty());
    let mut session = retained(SOURCE, (cell(0, 1), cell(3, 3)), "x", true);
    let before = session.view().point;
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    assert_eq!(session.echo(), "buffer is read-only");
    assert_eq!(session.view().point, before);
    session.execute(Command::FindFile);
    session.execute(Command::Rectangle(RectangleCommand::Yank));
    assert_eq!(session.echo(), "a prompt owns input focus");
    assert_eq!(text(&session), "x");
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["AB", "界", "", "e\u{301}X"]
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.kill_ring().len(), 0);
}

// Empty leading/trailing fragments preserve block height without synthesizing
// spaces. Undo includes any created separator while retaining the original point.
#[test]
fn empty_rows() {
    let cases = [
        (
            "012345\nab",
            (cell(0, 5), cell(1, 2)),
            "T",
            "T234\n",
            cell(1, 0),
        ),
        (
            "ab\n012345",
            (cell(0, 2), cell(1, 5)),
            "abc",
            "abc\n 234",
            cell(1, 4),
        ),
    ];
    for (source, endpoints, target, expected, cursor) in cases {
        let mut session = retained(source, endpoints, target, false);
        super::at(&mut session, cell(0, 1));
        let before = session.view().point.position;
        yank(&mut session);
        assert_eq!(text(&session), expected);
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap(),
            cursor
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), target);
        assert_eq!(session.view().point.position, before);
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Thousands of missing target rows form one EOF replacement and one history
// group, with exact prefix padding and no multiplication of per-row undo steps.
#[test]
fn large_rows() {
    let source = format!("0AB9\n{}\n0CD9", vec!["x界z"; 2048].join("\n"));
    let mut session = retained(&source, (cell(0, 1), cell(2049, 3)), "!", false);
    super::at(&mut session, cell(0, 1));
    let revision = session.buffer().unwrap().stamp().revision;
    yank(&mut session);
    let expected = format!("!AB\n{}\n CD", vec![" 界"; 2048].join("\n"));
    assert_eq!(text(&session), expected);
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
        1
    );
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        cell(2049, 3)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "!");
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    assert!(session.frame().unwrap().cursor().is_some());
}

// A held/released suffix cannot insert the block twice; a fresh invocation
// can reuse it and receives its own complete undo/redo group.
#[test]
fn terminal_phases() {
    use crossterm::event::KeyEventKind;
    let mut session = retained("0AB9\n0CD9", (cell(0, 1), cell(1, 3)), "", false);
    for (code, modifiers) in [
        (KeyCode::Char('x'), KeyModifiers::CONTROL),
        (KeyCode::Char('r'), KeyModifiers::NONE),
        (KeyCode::Char('y'), KeyModifiers::NONE),
    ] {
        session.handle(Event::Key(KeyEvent::new(code, modifiers)));
    }
    for phase in [KeyEventKind::Repeat, KeyEventKind::Release] {
        let mut key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        key.kind = phase;
        session.handle(Event::Key(key));
    }
    assert_eq!(text(&session), "AB\nCD");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    yank(&mut session);
    assert_eq!(text(&session), "AB\nCDAB\n  CD");
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "AB\nCD");
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "");
    assert!(!session.buffer().unwrap().is_modified());
}

// Two different grapheme joins occur in one CRLF rectangle transaction: ZWJ
// joins to a right-hand emoji and a detached accent joins its left neighbor.
// History from the peer must preserve separators and repair every live point.
#[test]
fn shared_seams() {
    use rev::command::Split;
    let original = "q💻z\r\nab";
    let expected = "q👩\u{200d}💻z\r\na\u{301}b";
    let mut session = retained(
        "👩\u{200d}X\r\n\u{301}",
        (cell(0, 0), cell(1, 1)),
        original,
        false,
    );
    assert_eq!(
        session.rectangle().unwrap().rows(),
        ["👩\u{200d}", "\u{301}"]
    );
    super::at(&mut session, cell(0, 1));
    session.execute(Command::Split(Split::Right));
    let origin = session.windows().selected_id();
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    super::at(&mut session, cell(1, 1));
    session.select_view(origin).unwrap();
    yank(&mut session);
    assert_eq!(text(&session), expected);
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
        2
    );
    session.select_view(peer).unwrap();
    for direction in [History::Undo, History::Redo, History::Undo, History::Redo] {
        session.execute(Command::History(direction));
        assert_eq!(
            text(&session),
            if direction == History::Undo {
                original
            } else {
                expected
            }
        );
        assert_eq!(
            session.buffer().unwrap().is_modified(),
            direction == History::Redo
        );
        let buffer = session.buffer().unwrap();
        let coordinates = Coordinates::new(buffer).unwrap();
        for (_, view) in session.windows().iter() {
            assert_eq!(view.point.stamp, buffer.stamp());
            assert!(coordinates.is_boundary(view.point.position).unwrap());
            coordinates.to_cell(view.point.position).unwrap();
        }
        session.frame().unwrap();
    }
    assert_eq!(session.kill_ring().len(), 0);
}
