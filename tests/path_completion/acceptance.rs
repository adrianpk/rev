use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::command::Command;
use rev::display::{Band, Frame};
use rev::session::Session;
use rev::view::Size;

use super::fixture::Fixture;

pub(super) fn editor(fixture: &Fixture) -> Session {
    let file = fixture.write("root/source.txt", "retained\ntext");
    fixture.write("root/alpha.txt", "alpha");
    fixture.write("root/child/nested.txt", "nested");
    let mut session = Session::empty(Size {
        width: 80,
        height: 14,
    })
    .unwrap();
    session.load(&file).unwrap();
    session
}

pub(super) fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

pub(super) fn query(frame: &Frame) -> &str {
    frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Echo)
        .unwrap()
        .text()
        .trim_end()
}

pub(super) fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn start(session: &mut Session, extended: bool) {
    if extended {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::ALT,
        )));
        session.handle(Event::Paste("find-file".into()));
        session.handle(key(KeyCode::Enter));
    } else {
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
        )));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('f'),
            KeyModifiers::CONTROL,
        )));
    }
}

// Both native entry points accept a navigated identity rather than its blank
// query or escaped label. A fresh Return opens exactly the chosen file.
#[test]
fn selected_file() {
    for extended in [false, true] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        start(&mut session, extended);
        session.handle(key(KeyCode::Down));
        session.handle(key(KeyCode::Enter));
        assert_eq!(
            session.buffer().unwrap().file().unwrap().path(),
            fixture.0.join("root/alpha.txt")
        );
        assert_eq!(text(&session), "alpha");
        assert!(!query(&session.frame().unwrap()).starts_with("Find file:"));
    }
}

// The default highlight does not override a freely typed draft, including a
// prefix of an existing suggestion. Missing paths become clean in-memory text.
#[test]
fn typed_file() {
    for (draft, present) in [("alpha.txt", true), ("al", false), ("new 界.txt", false)] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        session.execute(Command::FindFile);
        session.handle(Event::Paste(draft.into()));
        session.handle(key(KeyCode::Enter));
        let buffer = session.buffer().unwrap();
        assert_eq!(
            buffer.file().unwrap().path(),
            fixture.0.join("root").join(draft)
        );
        assert_eq!(buffer.file().unwrap().is_present(), present);
        assert_eq!(text(&session), if present { "alpha" } else { "" });
        assert!(!buffer.is_modified());
        assert_eq!(fixture.0.join("root").join(draft).exists(), present);
    }
}

// Selected and typed directories descend without visiting a buffer. Held
// Return cannot open a nested suggestion; only a new choice/press may continue.
#[test]
fn directory_return() {
    for selected in [false, true] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        session.execute(Command::FindFile);
        if selected {
            session.handle(key(KeyCode::Up));
        } else {
            session.handle(Event::Paste("./child".into()));
        }
        session.handle(key(KeyCode::Enter));
        let draft = if selected { "child/" } else { "./child/" };
        assert_eq!(
            query(&session.frame().unwrap()),
            format!("Find file: {draft}")
        );
        session.handle(Event::Key(KeyEvent::new_with_kind(
            KeyCode::Enter,
            KeyModifiers::NONE,
            KeyEventKind::Repeat,
        )));
        assert_eq!(*session.view(), view);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        session.handle(key(KeyCode::Up));
        session.handle(key(KeyCode::Enter));
        assert_eq!(text(&session), "nested");
    }
}

// Ingress errors retain the draft, selection context and original buffer.
// Correcting the path retries within the same prompt; C-g still cancels safely.
#[test]
fn retained_error() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let view = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("alpha.txt/child".into()));
    session.handle(key(KeyCode::Enter));
    assert!(!session.echo().is_empty());
    assert!(query(&session.frame().unwrap()).starts_with("Find file: alpha.txt/child  "));
    assert_eq!(*session.view(), view);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    for _ in 0..6 {
        session.handle(key(KeyCode::Backspace));
    }
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), "alpha");
}
