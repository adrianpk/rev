use std::fs;
use std::path::Path;

use rev::buffer::{BufferId, Buffers, Confirmation, SaveRequest};
use rev::command::Command;
use rev::file::Publication;
use rev::session::{Action, Session};
use rev::view::Size;

use super::{Fixture, chord, insert_text, press, snapshot};
use crossterm::event::KeyCode;

fn confirmation(buffers: &Buffers, id: BufferId, target: Option<&Path>) -> Confirmation {
    match buffers.request_save(id, target).unwrap() {
        SaveRequest::Confirm(confirmation) => confirmation,
        SaveRequest::Ready(_) => panic!("target requires explicit confirmation"),
    }
}

fn enter_path(session: &mut Session, path: &Path) {
    for character in path.to_str().unwrap().chars() {
        session.handle(press(KeyCode::Char(character)));
    }
    session.handle(press(KeyCode::Enter));
}

fn session() -> Session {
    Session::empty(Size {
        width: 240,
        height: 8,
    })
    .unwrap()
}

// Occupied targets and removed/changed visited files are captured without
// publication. Only their consumed explicit decision can acknowledge a save.
#[test]
fn observed_decisions() {
    for name in ["occupied", "modified", "removed"] {
        let fixture = Fixture::new();
        let path = fixture.file("target.txt", b"original");
        let mut buffers = Buffers::new().unwrap();
        let id = if name == "occupied" {
            buffers.create("").unwrap()
        } else {
            buffers.visit(&path).unwrap()
        };
        insert_text(buffers.get_mut(id).unwrap(), "new");
        match name {
            "modified" => fs::write(&path, b"external").unwrap(),
            "removed" => fs::remove_file(&path).unwrap(),
            _ => {}
        }
        let target = (name == "occupied").then_some(path.as_path());
        let confirmation = confirmation(&buffers, id, target);
        assert_eq!(confirmation.target(), path);
        assert!(confirmation.message().contains(path.to_str().unwrap()));
        let result = buffers.confirm_save(confirmation);
        assert_eq!(result.publication(), Publication::Complete, "{result:?}");
        assert!(!buffers.get(id).unwrap().is_modified());
        assert!(fs::read(&path).unwrap().starts_with(b"new"));
        assert_eq!(buffers.visit(&path).unwrap(), id);
    }
}

// Save-buffer prompts for unnamed text; write-file shares the focused argument
// owner and publishes only after Enter, preserving the edited view and history.
#[test]
fn destination_prompt() {
    for command in [Command::SaveBuffer, Command::WriteFile] {
        let fixture = Fixture::new();
        let path = fixture.0.join("new.txt");
        let mut session = session();
        session.insert('界');
        let before = snapshot(&session);
        session.execute(command);
        enter_path(&mut session, &path);
        assert!(session.echo().starts_with("Saved "), "{}", session.echo());
        let after = snapshot(&session);
        assert_eq!(after.view, before.view);
        assert_eq!(after.stamp, before.stamp);
        assert_eq!(after.state, before.state);
        assert_eq!(after.history, before.history);
        assert_eq!(fs::read(&path).unwrap(), "界".as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(session.buffer().unwrap().file().unwrap().path(), path);
        assert_eq!(session.execute(Command::Quit), Action::Exit);
    }
}

// No/keyboard-quit and invalid decision keys never reach document text or disk.
// The target question remains visible after invalid input and a resize.
#[test]
fn focused_consent() {
    let fixture = Fixture::new();
    let path = fixture.file("occupied.txt", b"keep");
    let mut session = session();
    session.insert('!');
    for answer in [KeyCode::Char('n'), KeyCode::Char('g'), KeyCode::Char('y')] {
        let before = snapshot(&session);
        session.handle(chord('x'));
        session.handle(chord('w'));
        enter_path(&mut session, &path);
        assert!(session.echo().contains("Overwrite occupied destination"));
        assert_eq!(session.frame().unwrap().cursor().unwrap().row, 7);
        session.handle(press(KeyCode::Enter));
        assert!(session.echo().contains("Answer y or n"));
        assert!(session.echo().contains(path.to_str().unwrap()));
        assert_eq!(snapshot(&session), before);
        let event = if answer == KeyCode::Char('g') {
            chord('g')
        } else {
            press(answer)
        };
        session.handle(event);
        if answer == KeyCode::Char('y') {
            assert_eq!(fs::read(&path).unwrap(), b"!");
            assert!(!session.buffer().unwrap().is_modified());
        } else {
            assert_eq!(fs::read(&path).unwrap(), b"keep");
            assert_eq!(snapshot(&session), before);
            assert!(session.frame().unwrap().cursor().is_some());
        }
    }
}

// Destination arguments cancel without side effects, even with a repeated Enter.
// An invalid destination consumes only its argument and returns to editing.
#[test]
fn argument_exit() {
    let fixture = Fixture::new();
    let mut session = session();
    session.insert('!');
    let before = snapshot(&session);
    session.execute(Command::WriteFile);
    for character in "cancel界".chars() {
        session.handle(press(KeyCode::Char(character)));
    }
    session.handle(chord('g'));
    let mut repeated =
        crossterm::event::KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
    repeated.kind = crossterm::event::KeyEventKind::Repeat;
    session.handle(crossterm::event::Event::Key(repeated));
    assert_eq!(snapshot(&session), before);
    for path in [Path::new(""), fixture.0.as_path()] {
        session.execute(Command::WriteFile);
        enter_path(&mut session, path);
        assert!(session.echo().starts_with("save: "));
        assert_eq!(snapshot(&session), before);
        assert_eq!(session.frame().unwrap().cursor().unwrap().row, 0);
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
}

// A visited target owned by another buffer is refused without a question, even
// through a symlink; consent cannot override canonical ownership.
#[test]
fn live_target() {
    let fixture = Fixture::new();
    let active = fixture.file("active.txt", b"active");
    let other = fixture.file("other.txt", b"other");
    let alias = fixture.0.join("alias.txt");
    std::os::unix::fs::symlink(&other, &alias).unwrap();
    let mut session = session();
    session.load(&other).unwrap();
    session.load(&active).unwrap();
    session.insert('!');
    let before = snapshot(&session);
    session.execute(Command::WriteFile);
    enter_path(&mut session, &alias);
    assert!(session.echo().contains("another live buffer"));
    assert_eq!(snapshot(&session), before);
    assert_eq!(fs::read(other).unwrap(), b"other");
    assert_eq!(fs::read(active).unwrap(), b"active");
    assert_eq!(session.frame().unwrap().cursor().unwrap().row, 0);
}

// A displayed question remains focused and keeps document point/scroll through
// tiny/empty geometry; cancel after resize restores the unchanged editing view.
#[test]
fn decision_geometry() {
    let fixture = Fixture::new();
    let path = fixture.file("occupied.txt", b"keep");
    let mut session = session();
    session.insert('!');
    session.execute(Command::WriteFile);
    enter_path(&mut session, &path);
    for (width, height) in [(0, 0), (1, 1), (3, 2), (240, 8)] {
        session.handle(crossterm::event::Event::Resize(width, height));
        let before = snapshot(&session);
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows().len(), usize::from(height));
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height);
            assert_eq!(cursor.row, height - 1);
        }
        assert_eq!(snapshot(&session), before);
        assert!(session.echo().contains(path.to_str().unwrap()));
    }
    let before = snapshot(&session);
    session.handle(chord('g'));
    assert_eq!(snapshot(&session), before);
    assert_eq!(fs::read(path).unwrap(), b"keep");
    assert_eq!(session.frame().unwrap().cursor().unwrap().row, 0);
}
