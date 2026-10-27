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
    buffers.get_mut(second).unwrap().set_readonly(true);
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

// Switching among modified unnamed/file owners retains each canonical history;
// an externally removed path remains editable and selectable without disk ingress.
#[test]
fn retained_owners() {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let unnamed = buffers.create("").unwrap();
    let name = buffers.get(unnamed).unwrap().name().to_owned();
    let file = buffers.visit(&path).unwrap();
    for id in [initial, unnamed, file] {
        insert_text(buffers.get_mut(id).unwrap(), "!");
    }
    let mut session = editor(buffers, initial);
    let original = snapshot(&session);
    command(&mut session, BufferCommand::Switch, &name);
    assert_eq!(text(session.buffer().unwrap()), "!");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    fs::remove_file(&path).unwrap();
    command(&mut session, BufferCommand::Switch, "document.txt");
    assert_eq!(session.buffer().unwrap().id(), file);
    assert_eq!(text(session.buffer().unwrap()), "!original");
    session.execute(Command::History(rev::command::History::Undo));
    assert_eq!(text(session.buffer().unwrap()), "original");
    assert!(!session.buffer().unwrap().is_modified());
    assert!(!path.exists());
    command(&mut session, BufferCommand::Switch, "*scratch*");
    assert_eq!(snapshot(&session).bytes, original.bytes);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert!(session.buffer().unwrap().is_modified());
    command(&mut session, BufferCommand::Switch, &name);
    assert_eq!(session.buffer().unwrap().id(), unnamed);
    assert!(session.buffer().unwrap().is_modified());
}

// Saving an inactive chosen owner must write that buffer, close only it, and
// retain the active view/text/history instead of accidentally saving the selection.
#[test]
fn inactive_close() {
    let fixture = Fixture::new();
    let path = fixture.file("inactive.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    let inactive = buffers.visit(&path).unwrap();
    insert_text(buffers.get_mut(initial).unwrap(), "active");
    insert_text(buffers.get_mut(inactive).unwrap(), "!");
    let mut session = editor(buffers, initial);
    let before = snapshot(&session);
    command(&mut session, BufferCommand::Kill, "inactive.txt");
    session.handle(press(KeyCode::Char('y')));
    assert_eq!(session.echo(), "Buffer closed");
    assert_eq!(snapshot(&session), before);
    assert_eq!(fs::read(&path).unwrap(), b"!original");
    session.load(&path).unwrap();
    assert_ne!(session.buffer().unwrap().id(), inactive);
    assert_eq!(text(session.buffer().unwrap()), "!original");
}

// Denied access, missing parents and native read-only state cannot satisfy a
// required save. The same dirty owner remains usable after each refusal.
#[test]
fn failed_close_save() {
    use std::os::unix::fs::PermissionsExt;
    for failure in ["denied", "missing", "readonly"] {
        let fixture = Fixture::new();
        let path = if failure == "missing" {
            fixture.0.join("missing/new.txt")
        } else {
            fixture.file("document.txt", b"original")
        };
        if failure == "denied" {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
        }
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        insert_text(buffers.get_mut(id).unwrap(), "!");
        if failure == "readonly" {
            buffers.get_mut(id).unwrap().set_readonly(true);
        }
        let mut session = editor(buffers, id);
        let before = snapshot(&session);
        command(&mut session, BufferCommand::Kill, "");
        session.handle(press(KeyCode::Char('y')));
        assert!(session.echo().starts_with("save: "), "{}", session.echo());
        assert_eq!(snapshot(&session), before, "{failure}");
        assert!(session.buffer().unwrap().is_modified());
        session.handle(press(KeyCode::Char('?')));
        assert_eq!(session.buffer().unwrap().id(), id);
        assert_eq!(
            session.buffer().unwrap().is_readonly(),
            failure == "readonly"
        );
        assert_ne!(session.echo(), "Buffer closed");
    }
}

// Required save preserves its close target across unnamed and overwrite prompts.
// Cancellation at either prompt drops close authority; a later successful save
// of the final owner installs a fresh scratch with no stale text/history.
#[test]
fn nested_close_save() {
    let fixture = Fixture::new();
    let path = fixture.file("occupied.txt", b"keep");
    let buffers = Buffers::new().unwrap();
    let original = buffers.initial();
    let mut session = editor(buffers, original);
    session.insert('界');
    for cancel in ["argument", "decision", "complete"] {
        let before = snapshot(&session);
        command(&mut session, BufferCommand::Kill, "");
        session.handle(press(KeyCode::Char('y')));
        if cancel == "argument" {
            session.handle(chord('g'));
        } else {
            argument(&mut session, path.to_str().unwrap());
            assert!(session.echo().contains("Overwrite occupied destination"));
            session.handle(press(KeyCode::Char(if cancel == "decision" {
                'n'
            } else {
                'y'
            })));
        }
        if cancel == "complete" {
            assert_ne!(session.buffer().unwrap().id(), original);
            assert_eq!(session.buffer().unwrap().name(), "*scratch*");
            assert_eq!(text(session.buffer().unwrap()), "");
            assert!(session.buffer().unwrap().history().is_empty());
            assert!(!session.buffer().unwrap().is_modified());
            assert_eq!(fs::read(&path).unwrap(), "界".as_bytes());
        } else {
            assert_eq!(snapshot(&session), before);
            assert_eq!(fs::read(&path).unwrap(), b"keep");
        }
    }
    assert_eq!(session.execute(Command::Quit), rev::session::Action::Exit);
    session.insert('!');
    assert_eq!(text(session.buffer().unwrap()), "!");
}

// A stale close request cannot discard later edits or invalidate live indexes;
// the same guard applies to native callers and session confirmation completion.
#[test]
fn stale_close() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.initial();
    let kill = buffers.prepare_kill(id).unwrap();
    insert_text(buffers.get_mut(id).unwrap(), "!");
    assert!(buffers.kill(kill).is_err());
    assert_eq!(buffers.iter().len(), 1);
    let mut session = editor(buffers, id);
    command(&mut session, BufferCommand::Kill, "");
    session.insert('?');
    session.handle(press(KeyCode::Char('n')));
    assert!(session.echo().contains("revision"));
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(text(session.buffer().unwrap()), "?!");
    assert!(session.buffer().unwrap().is_modified());
}

// Killing a clean final owner is immediate and cannot reuse its identity. A
// previously captured save cannot publish through a later owner of the same path.
#[test]
fn closed_identity() {
    let fixture = Fixture::new();
    let path = fixture.file("document.txt", b"original");
    let mut buffers = Buffers::new().unwrap();
    let old = buffers.visit(&path).unwrap();
    let save = buffers.prepare_save(old, None).unwrap();
    let kill = buffers.prepare_kill(old).unwrap();
    buffers.kill(kill).unwrap();
    let new = buffers.visit(&path).unwrap();
    assert_ne!(old, new);
    assert_eq!(buffers.get(new).unwrap().name(), "document.txt");
    let result = buffers.publish_save(save);
    assert_eq!(result.publication(), rev::file::Publication::NotPublished);
    assert_eq!(fs::read(&path).unwrap(), b"original");
    let buffers = Buffers::new().unwrap();
    let old = buffers.initial();
    let mut session = editor(buffers, old);
    command(&mut session, BufferCommand::Kill, "");
    assert_ne!(session.buffer().unwrap().id(), old);
    assert_eq!(session.echo(), "Buffer closed");
    session.insert('x');
    assert_eq!(text(session.buffer().unwrap()), "x");
}

// A long inventory draws only visible rows, safely escapes unusual names, and
// retains identity selection through scrolling and Enter-repeat suppression.
#[test]
fn inventory_limits() {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let fixture = Fixture::new();
    let mut buffers = Buffers::new().unwrap();
    let initial = buffers.initial();
    for _ in 0..40 {
        buffers.create("text").unwrap();
    }
    let path = fixture.file("last\n\x1b.txt", b"raw");
    let last = buffers.visit(&path).unwrap();
    let mut session = editor(buffers, initial);
    session.execute(Command::Buffer(BufferCommand::List));
    session.handle(press(KeyCode::End));
    let frame = session.frame().unwrap();
    assert_eq!(frame.rows().len(), 8);
    assert_eq!(
        frame
            .rows()
            .iter()
            .filter(|row| row.band() == Band::Document)
            .count(),
        6
    );
    for row in frame.rows() {
        assert!(!row.text().contains('\x1b'));
        assert!(!row.text().contains('\n'));
    }
    let repeated = Event::Key(KeyEvent::new_with_kind(
        KeyCode::Enter,
        KeyModifiers::NONE,
        KeyEventKind::Repeat,
    ));
    assert_eq!(session.handle(repeated.clone()), rev::session::Action::Wait);
    assert_eq!(session.buffer().unwrap().id(), initial);
    session.handle(press(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), last);
    assert_eq!(session.handle(repeated), rev::session::Action::Wait);
    assert_eq!(text(session.buffer().unwrap()), "raw");
}
