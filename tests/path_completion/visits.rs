use std::os::unix::fs::symlink;

use crossterm::event::{Event, KeyCode};
use rev::buffer::Buffers;
use rev::command::Command;
use rev::config::{IndentStyle, prepare};
use rev::session::Session;
use rev::view::{Size, View};

use super::acceptance::{editor, key, text};
use super::fixture::Fixture;

// Canonical visits through literal, relative and symlink aliases must reuse the
// dirty identity and undo state; completion cannot reload it from disk or save.
#[test]
fn dirty_aliases() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    symlink("source.txt", fixture.0.join("root/alias.txt")).unwrap();
    session.handle(key(KeyCode::Char('!')));
    let id = session.buffer().unwrap().id();
    for draft in ["source.txt", "./source.txt", "alias.txt"] {
        session.execute(Command::FindFile);
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        session.handle(Event::Paste(draft.into()));
        session.handle(key(KeyCode::Enter));
        assert_eq!(session.buffer().unwrap().id(), id);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        assert_eq!(*session.view(), view);
        assert_eq!(text(&session), "!retained\ntext");
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(
            std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
            b"retained\ntext"
        );
    }
}

// Returning from another buffer through a symlink still finds the dirty live
// destination instead of publishing a duplicate or rereading replaced bytes.
#[test]
fn revisit_destination() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    symlink("source.txt", fixture.0.join("root/alias.txt")).unwrap();
    session.handle(key(KeyCode::Char('!')));
    let id = session.buffer().unwrap().id();
    session.load(&fixture.0.join("root/alpha.txt")).unwrap();
    std::fs::write(fixture.0.join("root/source.txt"), "external").unwrap();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("alias.txt".into()));
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(text(&session), "!retained\ntext");
    assert!(session.buffer().unwrap().is_modified());
    assert_eq!(
        std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
        b"external"
    );
}

fn configured(fixture: &Fixture) -> Session {
    let config = fixture.write("config.toml", "files.newline = 'crlf'\nfiles.bom = true\n[language.go]\ntab_width = 8\nindent_width = 2\nindent_style = 'tabs'");
    let settings = prepare(&config);
    assert!(settings.diagnostic.is_none());
    let buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 14,
        },
    )
    .unwrap();
    session
        .load(&fixture.write("root/source.txt", "retained\ntext"))
        .unwrap();
    std::fs::write(
        config,
        "files.bom = false\nfiles.newline = 'lf'\nlanguage.go.tab_width = 3",
    )
    .unwrap();
    session
}

// New paths, even with missing parents, use the frozen startup language/format
// snapshot in memory. Editing does not create parents, a file or saved output.
#[test]
fn new_file_policy() {
    let fixture = Fixture::new();
    let mut session = configured(&fixture);
    session.execute(Command::FindFile);
    session.handle(Event::Paste("missing/new.go".into()));
    session.handle(key(KeyCode::Enter));
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.policy().tab_width, 8);
    assert_eq!(buffer.policy().indent_width, 2);
    assert_eq!(buffer.policy().indent_style, IndentStyle::Tabs);
    assert!(buffer.has_bom());
    assert_eq!(buffer.newline(), "\r\n");
    assert!(!buffer.is_modified());
    session.handle(key(KeyCode::Char('x')));
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), "x\r\n");
    assert!(session.buffer().unwrap().is_modified());
    assert!(!fixture.0.join("root/missing").exists());
    assert_eq!(
        std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
        b"retained\ntext"
    );
}

// Existing file ingress preserves separators, BOM and final-newline state,
// while applying its language preferences without publishing a rewrite.
#[test]
fn existing_file_policy() {
    let fixture = Fixture::new();
    let mut session = configured(&fixture);
    let path = fixture.write("root/existing.go", "a\nb\nlast");
    session.execute(Command::FindFile);
    session.handle(Event::Paste("existing.go".into()));
    session.handle(key(KeyCode::Enter));
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.policy().tab_width, 8);
    assert!(!buffer.has_bom());
    assert_eq!(buffer.newline(), "\n");
    assert_eq!(text(&session), "a\nb\nlast");
    assert!(!buffer.is_modified());
    assert!(buffer.history().is_empty());
    assert_eq!(std::fs::read(path).unwrap(), b"a\nb\nlast");
}
