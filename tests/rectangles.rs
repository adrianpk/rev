#![forbid(unsafe_code)]

#[path = "rectangles/kill.rs"]
mod kill;
#[path = "rectangles/yank.rs"]
mod yank;

use rev::buffer::Buffers;
use rev::buffer::edit::Point;
use rev::command::{Command, Motion, RectangleCommand, RegionCommand};
use rev::coordinates::{Cell, Coordinates, Scalar};
use rev::display::Size;
use rev::session::{Action, Session};
use rev::view::View;

fn selected(text: &str, a: Cell, b: Cell) -> Session {
    editor(text, a, b, false)
}

fn editor(text: &str, a: Cell, b: Cell, read_only: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(text).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_cell(a).unwrap();
    let end = coordinates.from_cell(b).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: start,
        })
        .unwrap();
    buffer.set_read_only(read_only);
    let mut view = View::new(buffer);
    view.point.position = end;
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

fn cell(row: usize, column: usize) -> Cell {
    Cell { row, column }
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn copy(session: &mut Session) {
    assert_eq!(
        session.execute(Command::Rectangle(RectangleCommand::Copy)),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Rectangle copied");
}

// Rectangular extraction selects existing cells on every inclusive source row:
// partial tabs become spaces, while touched wide and combining clusters stay whole.
#[test]
fn cell_fragments() {
    let source = "012345\na\t界z\nQ👩\u{200d}💻e\u{301}X\nx\n\n012345";
    let mut session = selected(source, cell(0, 2), cell(5, 5));
    let stamp = session.buffer().unwrap().stamp();
    let point = session.view().point;
    copy(&mut session);
    let block = session.rectangle().unwrap();
    assert_eq!(
        block.rows(),
        ["234", "  界", "👩\u{200d}💻e\u{301}X", "", "", "234"]
    );
    assert_eq!(block.width(), 3);
    assert_eq!(text(&session), source);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.view().point, point);
    assert!(session.buffer().unwrap().history().is_empty());
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.kill_ring().len(), 0);
}

// Toggle starts at point or reuses an already active mark; it changes only
// selection mode. Normal mark and C-g restore the linear/inactive interactions.
#[test]
fn mark_lifetime() {
    let mut session = selected("abcdef\nsecond", cell(0, 1), cell(1, 4));
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert!(!session.buffer().unwrap().mark_active());
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(11)));
    session.execute(Command::Motion(Motion::LineStart));
    session.execute(Command::Region(RegionCommand::Set));
    assert!(!session.buffer().unwrap().rectangle_active());
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(7)));
    assert!(session.buffer().unwrap().rectangle_active());
    session.execute(Command::KeyboardQuit);
    assert!(!session.buffer().unwrap().rectangle_active());
    assert_eq!(session.buffer().unwrap().mark(), Some(Scalar(7)));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Bound and named commands resolve to the same handlers; a nested Meta copy
// cannot escape to linear M-w, and a repeated prefix suffix is never inserted.
#[test]
fn command_routes() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::input::{Input, Outcome};
    let event = |code, modifiers| Event::Key(KeyEvent::new(code, modifiers));
    for meta in [KeyModifiers::ALT, KeyModifiers::NONE] {
        let mut input = Input::default();
        input.handle(event(KeyCode::Char('x'), KeyModifiers::CONTROL));
        assert_eq!(
            input.handle(event(KeyCode::Char(' '), KeyModifiers::NONE)),
            Outcome::Command(Command::Rectangle(RectangleCommand::Mark))
        );
        input.handle(event(KeyCode::Char('x'), KeyModifiers::CONTROL));
        input.handle(event(KeyCode::Char('r'), KeyModifiers::NONE));
        if meta.is_empty() {
            input.handle(event(KeyCode::Esc, KeyModifiers::NONE));
        }
        assert_eq!(
            input.handle(event(KeyCode::Char('w'), meta)),
            Outcome::Command(Command::Rectangle(RectangleCommand::Copy))
        );
    }
    assert_eq!(
        Command::named("rectangle-mark-mode"),
        Some(Command::Rectangle(RectangleCommand::Mark))
    );
    assert_eq!(
        Command::named("copy-rectangle-as-kill"),
        Some(Command::Rectangle(RectangleCommand::Copy))
    );
}

fn at(session: &mut Session, target: Cell) {
    let position = Coordinates::new(session.buffer().unwrap())
        .unwrap()
        .from_cell(target)
        .unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    while session.view().point.position < position {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    assert_eq!(session.view().point.position, position);
}

// Ordering each axis separately gives the same block for reversed point/mark
// or crossed columns; selection and copy share row-local Unicode edge expansion.
#[test]
fn reversed_edges() {
    let source = "012345\na\t界z\nQ👩\u{200d}💻e\u{301}X\nx\n\n012345";
    let cases = [
        (cell(0, 2), cell(5, 5)),
        (cell(5, 5), cell(0, 2)),
        (cell(0, 5), cell(5, 2)),
        (cell(5, 2), cell(0, 5)),
    ];
    for (a, b) in cases {
        let mut session = selected(source, a, b);
        copy(&mut session);
        assert_eq!(
            session.rectangle().unwrap().rows(),
            ["234", "  界", "👩\u{200d}💻e\u{301}X", "", "", "234"]
        );
        let frame = session.frame().unwrap();
        let ranges: Vec<_> = frame.rows().iter().map(|row| row.selection()).collect();
        assert_eq!(
            ranges,
            [
                Some(2..5),
                Some(2..6),
                Some(1..5),
                None,
                None,
                Some(2..5),
                None,
                None
            ]
        );
        assert_eq!(text(&session), source);
    }
}

// Tabs follow absolute four-cell stops. Only partially selected tabs expand;
// an unselected tab and complete selected tab retain their source bytes.
#[test]
fn tab_edges() {
    let source = "0123456789\na\tb\tc\n0123456789";
    let cases = [
        (1, 4, "\t"),
        (2, 3, " "),
        (0, 2, "a "),
        (2, 5, "  b"),
        (4, 8, "b\t"),
        (5, 7, "  "),
        (7, 9, " c"),
        (8, 10, "c"),
    ];
    for (left, right, expected) in cases {
        let mut session = selected(source, cell(0, left), cell(2, right));
        copy(&mut session);
        assert_eq!(
            session.rectangle().unwrap().rows()[1],
            expected,
            "{left}..{right}"
        );
        assert_eq!(session.rectangle().unwrap().width(), right - left);
        assert_eq!(text(&session), source);
    }
}

// Both edges can touch the interiors of different wide clusters. Controls are
// copied as source characters, CRLF is excluded, and detached marks stay intact.
#[test]
fn unicode_edges() {
    let cases = [
        ("e\u{301}界👩\u{200d}💻z", 2, 4, "界👩\u{200d}💻"),
        ("e\u{301}界👩\u{200d}💻z", 0, 1, "e\u{301}"),
        ("a🇵🇱b", 2, 3, "🇵🇱"),
        ("a\r\x1bb", 2, 4, "\r\x1b"),
        ("\u{301}界", 0, 2, "\u{301}界"),
    ];
    for (middle, left, right, expected) in cases {
        let source = format!("0123456789\r\n{middle}\r\n0123456789");
        let mut session = selected(&source, cell(0, left), cell(2, right));
        copy(&mut session);
        assert_eq!(
            session.rectangle().unwrap().rows()[1],
            expected,
            "{middle:?}"
        );
        assert_eq!(text(&session), source);
        let frame = session.frame().unwrap();
        assert!(frame.rows().iter().all(|row| !row.text().contains('\x1b')));
    }
}

// Selection projects only existing cells, including clipped edge fragments;
// blank short rows, status and echo never acquire a rectangle background.
#[test]
fn clipped_surface() {
    use rev::display::Frame;
    let mut session = selected(
        "012345\na\t界z\nQ👩\u{200d}💻e\u{301}X\nx\n\n012345",
        cell(0, 2),
        cell(5, 5),
    );
    let mut view = session.view().clone();
    view.top = 1;
    view.left = 3;
    let frame = Frame::project(
        session.buffer().unwrap(),
        &view,
        Size {
            width: 3,
            height: 5,
        },
        "notice",
    )
    .unwrap();
    assert_eq!(frame.rows()[0].selection(), Some(0..3));
    assert_eq!(frame.rows()[1].selection(), Some(0..2));
    assert!(
        frame
            .rows()
            .iter()
            .skip(2)
            .all(|row| row.selection().is_none())
    );
    let mut wire = Vec::new();
    frame.write(&mut wire).unwrap();
    assert!(wire.windows(4).any(|part| part == b"\x1b[7m"));
    assert!(wire.windows(5).any(|part| part == b"\x1b[27m"));
    copy(&mut session);
    assert_eq!(session.rectangle().unwrap().rows()[1], "  界");
}

// Copy scans the selected rope rows rather than the visible surface or a
// whole-file presentation copy; clusters crossing storage chunks stay whole.
#[test]
fn scrolled_chunks() {
    use crossterm::event::Event;
    let anchor = "a".repeat(1030);
    let middle = format!("{}e\u{301}界👩\u{200d}💻tail", "a".repeat(1020));
    let source = format!("{anchor}\n{}\n{anchor}", vec![middle; 80].join("\n"));
    let mut session = selected(&source, cell(0, 1021), cell(81, 1024));
    session.handle(Event::Resize(5, 4));
    let frame = session.frame().unwrap();
    assert!(session.view().top > 0 && session.view().left > 0);
    assert!(frame.cursor().is_some());
    copy(&mut session);
    let block = session.rectangle().unwrap();
    assert_eq!(block.width(), 3);
    assert_eq!(block.rows().len(), 82);
    assert_eq!(block.rows()[0], "aaa");
    assert_eq!(block.rows()[81], "aaa");
    assert!(
        block.rows()[1..81]
            .iter()
            .all(|row| row == "界👩\u{200d}💻")
    );
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// The rectangle replaces only its own store. Rejected empty/inactive copies
// retain it; linear copying/yanking still uses the prior independent ring.
#[test]
fn separate_store() {
    use rev::command::{History, LinearCommand};
    let source = "012345\na\t界z\n012345";
    let mut session = selected(source, cell(0, 2), cell(2, 5));
    copy(&mut session);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::Linear(LinearCommand::Copy));
    assert_eq!(session.kill_ring().next().unwrap().to_string(), "5");
    session.execute(Command::KeyboardQuit);
    at(&mut session, cell(0, 2));
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    at(&mut session, cell(2, 5));
    copy(&mut session);
    assert_eq!(session.kill_ring().next().unwrap().to_string(), "5");
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Rectangle(RectangleCommand::Mark));
    session.execute(Command::Rectangle(RectangleCommand::Copy));
    assert_eq!(session.echo(), "Rectangle is empty");
    assert_eq!(session.rectangle().unwrap().rows(), ["234", "  界", "234"]);
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Rectangle(RectangleCommand::Copy));
    assert_eq!(session.echo(), "Rectangle is not active");
    session.execute(Command::Motion(Motion::LineEnd));
    session.execute(Command::Linear(LinearCommand::Yank));
    assert_eq!(text(&session), format!("{source}5"));
    assert_eq!(session.rectangle().unwrap().rows(), ["234", "  界", "234"]);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    assert!(!session.buffer().unwrap().is_modified());
}

// Copy is allowed on read-only text and creates no history/dirty transition.
// Refused edits keep rectangle selection, while successful publication clears it.
#[test]
fn edit_lifetime() {
    use rev::command::History;
    for read_only in [false, true] {
        let mut session = editor("012345\na\t界z\n012345", cell(0, 2), cell(2, 5), read_only);
        copy(&mut session);
        let stamp = session.buffer().unwrap().stamp();
        session.insert('!');
        assert_eq!(session.buffer().unwrap().rectangle_active(), read_only);
        assert_eq!(session.buffer().unwrap().stamp() == stamp, read_only);
        assert_eq!(
            session.buffer().unwrap().history().len(),
            usize::from(!read_only)
        );
        assert_eq!(session.rectangle().unwrap().rows(), ["234", "  界", "234"]);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), "012345\na\t界z\n012345");
        assert!(!session.buffer().unwrap().is_modified());
    }
}

// Mode changes invalidate prepared publication just like mark movement. A
// foreign stamp or interior combining position cannot alter the active endpoint.
#[test]
fn qualified_mode() {
    use rev::buffer::edit::{Error, Replacement};
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("e\u{301}abc").unwrap();
    let foreign = buffers.get(buffers.initial()).unwrap().stamp();
    let buffer = buffers.get_mut(id).unwrap();
    let point = Point {
        stamp: buffer.stamp(),
        position: Scalar(2),
    };
    buffer.activate_mark(point).unwrap();
    let replacement = Replacement {
        source: buffer.span(2..2).unwrap(),
        text: "!".into(),
    };
    let prepared = buffer.prepare(&[replacement], &[point], None).unwrap();
    buffer.toggle_rectangle(point).unwrap();
    assert!(matches!(
        buffer.publish(prepared, &mut [point]),
        Err(Error::ChangedPositions)
    ));
    assert!(
        buffer
            .toggle_rectangle(Point {
                stamp: foreign,
                ..point
            })
            .is_err()
    );
    assert!(
        buffer
            .toggle_rectangle(Point {
                position: Scalar(1),
                ..point
            })
            .is_err()
    );
    assert_eq!(buffer.mark(), Some(Scalar(2)));
    assert!(buffer.rectangle_active());
    assert!(buffer.history().is_empty());
}

// A shared buffer keeps its mode/mark, but each selected pane supplies its own
// point. Prompt focus cannot run a copy; named entry and normal editing resume.
#[test]
fn shared_focus() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::Split;
    let mut session = selected("012345\na\t界z\n012345", cell(0, 2), cell(2, 5));
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    at(&mut session, cell(0, 4));
    copy(&mut session);
    assert_eq!(session.rectangle().unwrap().rows(), ["23"]);
    assert!(
        session.frame().unwrap().rows()[0]
            .selection()
            .unwrap()
            .start
            >= 24
    );
    session.execute(Command::FindFile);
    session.execute(Command::Rectangle(RectangleCommand::Copy));
    assert_eq!(session.echo(), "a prompt owns input focus");
    assert_eq!(session.rectangle().unwrap().rows(), ["23"]);
    session.execute(Command::KeyboardQuit);
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("rectangle-mark-mode".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert!(session.buffer().unwrap().rectangle_active());
    at(&mut session, cell(0, 5));
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("copy-rectangle-as-kill".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    assert_eq!(session.echo(), "Rectangle copied");
    assert_eq!(session.rectangle().unwrap().rows(), ["4"]);
    assert!(session.buffer().unwrap().history().is_empty());
}
