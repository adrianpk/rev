use std::fs;

use crossterm::event::{Event, KeyCode};
use rev::buffer::{BufferId, Buffers};
use rev::command::{BufferCommand, Command};
use rev::display::Band;
use rev::session::Session;
use rev::view::{Size, View};

use super::{Fixture, chord, insert_text, press, snapshot, text};

fn editor(buffers: Buffers, id: BufferId) -> Session {
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 160,
            height: 8,
        },
    )
    .unwrap()
}

fn argument(session: &mut Session, name: &str) {
    for character in name.chars() {
        session.handle(press(KeyCode::Char(character)));
    }
    session.handle(press(KeyCode::Enter));
}

fn command(session: &mut Session, action: BufferCommand, name: &str) {
    session.execute(Command::Buffer(action));
    argument(session, name);
}

// Closing releases the sole canonical path index and repairs initial/replacement
// identities; the last owner becomes a fresh, clean, editable scratch buffer.
#[test]
fn registry_lifetime() {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let file = buffers.visit(&path).unwrap();
    let kill = buffers.prepare_kill(initial).unwrap();
    assert_eq!(buffers.kill(kill).unwrap(), file);
    assert_eq!(buffers.initial(), file);
    assert!(buffers.get(initial).is_err());
    assert_eq!(buffers.named("document.txt"), Some(file));
    let kill = buffers.prepare_kill(file).unwrap();
    let replacement = buffers.kill(kill).unwrap();
    assert_ne!(replacement, file);
    assert_eq!(buffers.iter().len(), 1);
    assert_eq!(buffers.initial(), replacement);
    assert_eq!(buffers.get(replacement).unwrap().name(), "*scratch*");
    assert!(!buffers.get(replacement).unwrap().is_modified());
    assert!(buffers.get(file).is_err());
    let reopened = buffers.visit(&path).unwrap();
    assert_ne!(reopened, file);
    assert_eq!(buffers.get(reopened).unwrap().name(), "document.txt");
    assert_eq!(text(buffers.get(reopened).unwrap()), "original");
}

// Tab completes a unique live name and a shared Unicode prefix; Enter selects
// an exact name, while unknown names and cancellation preserve the active owner.
#[test]
fn buffer_completion() {
    let fixture = Fixture::new();
    let first = fixture.file("界alpha", b"first");
    let second = fixture.file("界alpine", b"second");
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let first_id = buffers.visit(&first).unwrap();
    buffers.visit(&second).unwrap();
    let mut session = editor(buffers, initial);
    session.handle(chord('x'));
    session.handle(press(KeyCode::Char('b')));
    session.handle(press(KeyCode::Char('界')));
    session.handle(press(KeyCode::Tab));
    assert!(session.echo().contains("2 matching"));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Switch to buffer: 界alp")
    );
    argument(&mut session, "ha");
    assert_eq!(session.buffer().unwrap().id(), first_id);
    let before = snapshot(&session);
    command(&mut session, BufferCommand::Switch, "missing");
    assert_eq!(session.echo(), "No buffer with that name");
    assert_eq!(snapshot(&session), before);
    session.execute(Command::Buffer(BufferCommand::Switch));
    argument(&mut session, "");
    assert_eq!(snapshot(&session), before);
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(press(KeyCode::Char('*')));
    session.handle(press(KeyCode::Tab));
    session.handle(chord('g'));
    assert_eq!(snapshot(&session), before);
    session.execute(Command::Buffer(BufferCommand::Switch));
    session.handle(press(KeyCode::Char('*')));
    session.handle(press(KeyCode::Tab));
    session.handle(press(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), initial);
}

// Inventory projection is bounded to the frame and never modifies canonical
// text. End/Home/arrows select by identity, including duplicate file basenames.
#[test]
fn buffer_inventory() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("one")).unwrap();
    fs::create_dir(fixture.0.join("two")).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let first = buffers
        .visit(&fixture.file("one/note.txt", b"first"))
        .unwrap();
    let second = buffers
        .visit(&fixture.file("two/note.txt", b"second"))
        .unwrap();
    insert_text(buffers.get_mut(first).unwrap(), "!");
    buffers.get_mut(second).unwrap().set_read_only(true);
    let mut session = editor(buffers, first);
    let before = snapshot(&session);
    session.handle(chord('x'));
    session.handle(chord('b'));
    let frame = session.frame().unwrap();
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().contains("note.txt ** RW"))
    );
    assert!(
        frame
            .rows()
            .iter()
            .any(|row| row.text().contains("note.txt<2> -- RO"))
    );
    session.handle(press(KeyCode::Char('x')));
    session.handle(press(KeyCode::End));
    session.handle(chord('g'));
    assert_eq!(snapshot(&session), before);
    session.execute(Command::Buffer(BufferCommand::List));
    session.handle(press(KeyCode::Down));
    session.handle(press(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), second);
    session.execute(Command::Buffer(BufferCommand::List));
    session.handle(press(KeyCode::Home));
    session.handle(press(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), initial);
    session.execute(Command::Buffer(BufferCommand::List));
    for (width, height) in [(0, 0), (1, 1), (3, 2), (20, 4), (160, 8)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        assert_eq!(frame.rows().len(), usize::from(height));
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height);
        }
    }
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.band() == Band::Status)
    );
}

// A modified buffer closes only after the requested choice. Cancel retains its
// edits; discard preserves disk bytes and deterministically selects the first owner.
#[test]
fn close_decisions() {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let id = buffers.visit(&path).unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "!");
    let mut session = editor(buffers, id);
    let before = snapshot(&session);
    command(&mut session, BufferCommand::Kill, "");
    assert!(session.echo().contains("y=save, n=discard, C-g=cancel"));
    session.handle(press(KeyCode::Enter));
    assert!(session.echo().contains("Answer y or n"));
    session.handle(chord('g'));
    assert_eq!(snapshot(&session), before);
    command(&mut session, BufferCommand::Kill, "");
    session.handle(press(KeyCode::Char('n')));
    assert_eq!(session.buffer().unwrap().id(), initial);
    assert_eq!(fs::read(&path).unwrap(), b"original");
    command(&mut session, BufferCommand::Switch, "document.txt");
    assert_eq!(session.echo(), "No buffer with that name");
    session.load(&path).unwrap();
    assert_ne!(session.buffer().unwrap().id(), id);
    assert_eq!(text(session.buffer().unwrap()), "original");
}
