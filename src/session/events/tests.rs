use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::command::{Command, Motion};
use crate::view::Size;

fn session() -> Session {
    Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap()
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// A busy source cannot starve drawing. All events left at the count boundary
// remain available to the next batch, with no change to text or history.
#[test]
fn event_limit() {
    let mut session = session();
    let original = "row\n".repeat(300);
    session.handle(Event::Paste(original.clone()));
    session.execute(Command::Motion(Motion::BufferStart));
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    let mut pending = VecDeque::from(vec![key(KeyCode::Down, KeyModifiers::NONE); 200]);
    while let Some(event) = pending.pop_front() {
        let before = pending.len() + 1;
        let started = Instant::now();
        let action = session.handle(event);
        assert_eq!(
            session
                .drain_input(action, started, || Ok(pending.pop_front()))
                .unwrap(),
            Action::Redraw
        );
        assert!(before - pending.len() <= INPUT_BATCH_EVENTS);
        let frame = session.frame().unwrap();
        assert!(frame.cursor().is_some());
    }
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        200
    );
    assert_eq!(text(&session), original);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
}

// Budget expiry and successful exit both leave the next event untouched. An
// ignored event must also retain any redraw already requested in this batch.
#[test]
fn batch_boundaries() {
    for (name, action, elapsed) in [
        ("time", Action::Redraw, INPUT_BATCH_TIME),
        ("exit", Action::Exit, Duration::ZERO),
    ] {
        let mut session = session();
        let mut reads = 0;
        let outcome = session
            .drain_input(action, Instant::now() - elapsed, || {
                reads += 1;
                Ok(Some(key(KeyCode::Char('X'), KeyModifiers::NONE)))
            })
            .unwrap();
        assert_eq!(outcome, action, "{name}");
        assert_eq!(reads, 0, "{name}");
        assert!(text(&session).is_empty(), "{name}");
    }
    let mut session = session();
    let mut pending = VecDeque::from([Event::FocusGained]);
    assert_eq!(
        session
            .drain_input(Action::Redraw, Instant::now(), || Ok(pending.pop_front()))
            .unwrap(),
        Action::Redraw
    );
}

// Batched handlers must match drawing after each event, including reversal,
// cancellation, resize and undo. Intermediate frames are the only omitted work.
#[test]
fn ordered_edit_history() {
    let mut reference = session();
    let mut batched = session();
    let original = "abc\n".repeat(120);
    let mut events = vec![
        Event::Paste(original.clone()),
        key(KeyCode::Char('<'), KeyModifiers::ALT),
    ];
    events.extend(vec![key(KeyCode::Down, KeyModifiers::NONE); 80]);
    events.extend(vec![key(KeyCode::Up, KeyModifiers::NONE); 30]);
    events.extend([
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('r'), KeyModifiers::NONE),
        key(KeyCode::Char('g'), KeyModifiers::CONTROL),
        key(KeyCode::Char('X'), KeyModifiers::NONE),
        Event::Resize(100, 30),
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('u'), KeyModifiers::NONE),
        Event::FocusGained,
    ]);
    for event in events.clone() {
        reference.handle(event);
        reference.frame().unwrap();
    }
    let mut pending = VecDeque::from(events);
    while let Some(event) = pending.pop_front() {
        let started = Instant::now();
        let action = batched.handle(event);
        batched
            .drain_input(action, started, || Ok(pending.pop_front()))
            .unwrap();
        batched.frame().unwrap();
    }
    assert_eq!(text(&batched), original);
    assert_eq!(text(&batched), text(&reference));
    assert_eq!(
        batched.view().point.position,
        reference.view().point.position
    );
    assert_eq!(batched.view().top, reference.view().top);
    assert_eq!(batched.view().left, reference.view().left);
    assert_eq!(batched.view().column, reference.view().column);
    assert_eq!(
        batched.frame().unwrap().cursor(),
        reference.frame().unwrap().cursor()
    );
    assert_eq!(batched.echo(), reference.echo());
    let buffer = batched.buffer().unwrap();
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.redo_history().len(), 1);
    assert_eq!(
        buffer.stamp().revision,
        reference.buffer().unwrap().stamp().revision
    );
}

// Quit is handled at its original ordered position. No later queued edit can
// run once the native handler accepts exit.
#[test]
fn exit_stops_reading() {
    let mut session = session();
    let mut pending = VecDeque::from([
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        key(KeyCode::Char('X'), KeyModifiers::NONE),
    ]);
    assert_eq!(
        session
            .drain_input(Action::Wait, Instant::now(), || Ok(pending.pop_front()))
            .unwrap(),
        Action::Exit
    );
    assert_eq!(pending.len(), 1);
    assert!(text(&session).is_empty());
}

// Each event still checks the Escape deadline. A late plain key is inserted;
// a timely suffix retains Meta routing rather than entering the document.
#[test]
fn escape_in_batch() {
    for (name, age, expected) in [
        ("timely", Duration::ZERO, ""),
        ("expired", crate::input::META_TIMEOUT, "f"),
    ] {
        let mut session = session();
        let action = session.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), Instant::now() - age);
        let mut pending = VecDeque::from([key(KeyCode::Char('f'), KeyModifiers::NONE)]);
        session
            .drain_input(action, Instant::now(), || Ok(pending.pop_front()))
            .unwrap();
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.input.remaining(Instant::now()), None, "{name}");
    }
}
