#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, RegionCommand, Split};
use rev::coordinates::{Coordinates, Scalar};
use rev::display::Size;
use rev::search::Pattern;
use rev::session::{Action, Session};
use rev::view::View;

#[path = "search/control.rs"]
mod control;
#[path = "search/edges.rs"]
mod edges;
#[path = "search/replace.rs"]
mod replace;

fn editor(source: &str, position: usize) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.point.position = Scalar(position);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 10,
        },
    )
    .unwrap()
}

fn key(session: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    assert_eq!(
        session.handle(Event::Key(KeyEvent::new(code, modifiers))),
        Action::Redraw
    );
}

fn control(session: &mut Session, value: char) {
    key(session, KeyCode::Char(value), KeyModifiers::CONTROL);
}

fn query(session: &mut Session, text: &str) {
    assert_eq!(session.handle(Event::Paste(text.into())), Action::Redraw);
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// A real rope seam falls inside a literal spanning Unicode and LF; the matcher
// returns original scalar coordinates and freshness rather than flattened bytes.
#[test]
fn chunk_match() {
    for unit in ["é界\n", "e\u{301}👩\u{200d}💻\r\n", "abababac界\n"] {
        let source = unit.repeat(1500);
        let session = editor(&source, 0);
        let buffer = session.buffer().unwrap();
        let whole = buffer.span(0..buffer.len_chars()).unwrap();
        let seam = buffer
            .chunks(&whole)
            .unwrap()
            .next()
            .unwrap()
            .chars()
            .count();
        let expected = buffer.span(seam - 12..seam + 12).unwrap();
        let literal = buffer.slice(&expected).unwrap().to_string();
        assert!(literal.contains('\n'));
        let found = Pattern::new(&literal)
            .unwrap()
            .find(buffer, Scalar(seam - 12))
            .unwrap()
            .unwrap();
        assert_eq!(found, expected);
        assert_eq!(buffer.slice(&found).unwrap().to_string(), literal);
        assert_eq!(text(&session), source);
    }
}

// Query changes rescan from the current search anchor; repetition advances and a
// miss retains a valid point, visible query and failure without any text history.
#[test]
fn incremental_query() {
    let mut session = editor("zero cat café cat", 0);
    let stamp = session.buffer().unwrap().stamp();
    let state = session.buffer().unwrap().state();
    control(&mut session, 's');
    key(&mut session, KeyCode::Char('c'), KeyModifiers::NONE);
    assert_eq!(session.search_match().unwrap().range, 5..6);
    query(&mut session, "at");
    assert_eq!(session.search_match().unwrap().range, 5..8);
    control(&mut session, 's');
    assert_eq!(session.search_match().unwrap().range, 14..17);
    query(&mut session, "x");
    assert!(session.search_match().is_none());
    assert_eq!(session.view().point.position, Scalar(17));
    assert_eq!(session.echo(), "No match");
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .last()
            .unwrap()
            .text()
            .contains("I-search: catx")
    );
    assert!(frame.rows().last().unwrap().text().contains("No match"));
    key(&mut session, KeyCode::Backspace, KeyModifiers::NONE);
    assert_eq!(session.search_match().unwrap().range, 14..17);
    assert_eq!(session.echo(), "");
    let buffer = session.buffer().unwrap();
    assert_eq!((buffer.stamp(), buffer.state()), (stamp, state));
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert_eq!(text(&session), "zero cat café cat");
    Coordinates::new(buffer)
        .unwrap()
        .to_cell(session.view().point.position)
        .unwrap();
}

// Only the selected pane underlines the match. Region reverse video remains
// independent, and accepting removes search styling without changing the rope.
#[test]
fn selected_surface() {
    let mut session = editor("a\t界e\u{301}z", 1);
    session.execute(Command::Region(RegionCommand::Set));
    session.execute(Command::Split(Split::Right));
    let other = session.windows().order()[1];
    let other_view = session.windows().get(other).unwrap().clone();
    session.execute(Command::SearchForward);
    query(&mut session, "界e");
    assert_eq!(session.search_match().unwrap().range, 2..4);
    assert_eq!(session.view().point.position, Scalar(5));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows()[0].search(), Some(4..7));
    assert_eq!(frame.rows()[0].selection(), Some(1..7));
    assert!(
        frame
            .rows()
            .iter()
            .skip(1)
            .all(|row| row.search().is_none())
    );
    let mut wire = Vec::new();
    frame.write(&mut wire).unwrap();
    assert!(wire.windows(4).any(|part| part == b"\x1b[4m"));
    assert!(wire.windows(5).any(|part| part == b"\x1b[24m"));
    assert!(wire.windows(4).any(|part| part == b"\x1b[7m"));
    assert_eq!(session.windows().get(other).unwrap(), &other_view);
    key(&mut session, KeyCode::Enter, KeyModifiers::NONE);
    assert!(session.search_match().is_none());
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .all(|row| row.search().is_none())
    );
    assert_eq!(text(&session), "a\t界e\u{301}z");
}
