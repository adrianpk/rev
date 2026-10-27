use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::display::{Band, Frame};
use rev::session::Session;
use rev::view::Size;

use super::fixture::Fixture;

fn editor(fixture: &Fixture) -> Session {
    let file = fixture.write("root/document.txt", "retained\ntext");
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

fn labels(frame: &Frame) -> Vec<&str> {
    frame
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim())
        .collect()
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

// Find-file starts at the visiting file's parent and projects the same vertical
// panel contract as M-x, with a separate query cursor and directory markers.
#[test]
fn visiting_directory() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let view = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    let frame = session.frame().unwrap();
    assert_eq!(labels(&frame), ["child/", "alpha.txt", "document.txt"]);
    let query = frame
        .rows()
        .iter()
        .find(|row| row.band() == Band::Echo)
        .unwrap();
    assert!(query.text().starts_with("Find file: "));
    assert_eq!(frame.cursor().unwrap().row, query.row());
    assert_eq!(
        frame.rows()[usize::from(query.row()) + 1].selection(),
        Some(0..80)
    );
    assert_eq!(*session.view(), view);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.execute(Command::KeyboardQuit);
    assert!(labels(&session.frame().unwrap()).is_empty());
    assert_eq!(*session.view(), view);
}

// An unnamed buffer uses cwd rather than the last visited fixture or a global
// remembered path. Prefix filtering never changes its document or history.
#[test]
fn unnamed_directory() {
    let mut session = Session::empty(Size {
        width: 80,
        height: 14,
    })
    .unwrap();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("Cargo.tom".into()));
    assert_eq!(labels(&session.frame().unwrap()), ["Cargo.toml"]);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Query edits within one directory and repeated rendering keep the observation
// even after that directory moves. Cancel restores the retained workspace view.
#[test]
fn retained_observation() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    session.frame().unwrap();
    let view = session.view().clone();
    session.execute(Command::FindFile);
    std::fs::rename(fixture.0.join("root"), fixture.0.join("moved")).unwrap();
    session.handle(Event::Paste("al".into()));
    for _ in 0..3 {
        assert_eq!(labels(&session.frame().unwrap()), ["alpha.txt"]);
    }
    session.handle(key(KeyCode::Backspace));
    assert_eq!(labels(&session.frame().unwrap()), ["alpha.txt"]);
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
    assert_eq!(*session.view(), view);
}

// Native Tab descends through a completed directory without visiting a buffer.
// Deleting its separator reobserves the parent, preserving text and saved state.
#[test]
fn native_descent() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let stamp = session.buffer().unwrap().stamp();
    let file = session.buffer().unwrap().file().unwrap().path().to_owned();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("ch".into()));
    session.handle(key(KeyCode::Tab));
    let frame = session.frame().unwrap();
    assert_eq!(labels(&frame), ["nested.txt"]);
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().starts_with("Find file: child/"))
    );
    session.handle(key(KeyCode::Backspace));
    assert_eq!(labels(&session.frame().unwrap()), ["child/"]);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), file);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Both navigation pairs move the highlighted candidate without changing the
// draft. Tab fills that identity and neither opens it nor edits the document.
#[test]
fn native_selection() {
    for next in [
        key(KeyCode::Down),
        Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL)),
    ] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        let stamp = session.buffer().unwrap().stamp();
        session.execute(Command::FindFile);
        session.handle(next);
        let frame = session.frame().unwrap();
        let selected = frame
            .rows()
            .iter()
            .find(|row| row.band() == Band::Completion && row.selection().is_some())
            .unwrap();
        assert_eq!(selected.text().trim(), "alpha.txt");
        session.handle(key(KeyCode::Tab));
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().starts_with("Find file: alpha.txt"))
        );
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert!(
            session
                .buffer()
                .unwrap()
                .file()
                .unwrap()
                .path()
                .ends_with("document.txt")
        );
    }
}
