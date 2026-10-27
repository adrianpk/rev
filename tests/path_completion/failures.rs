use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{PermissionsExt, symlink};

use crossterm::event::{Event, KeyCode};
use rev::command::Command;

use super::acceptance::{editor, key, query, text};
use super::fixture::Fixture;

// Invalid contents, components and symlink destinations cannot consume the
// prompt or publish a visit. Cancel retains dirty text, point and saved truth.
#[test]
fn rejected_paths() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    std::fs::write(fixture.0.join("root/invalid"), b"bad\xff").unwrap();
    std::fs::write(fixture.0.join("root/nul"), b"bad\0text").unwrap();
    symlink("absent", fixture.0.join("root/dangling")).unwrap();
    symlink("loop", fixture.0.join("root/loop")).unwrap();
    session.handle(key(KeyCode::Char('!')));
    for draft in ["invalid", "nul", "alpha.txt/child", "dangling", "loop"] {
        session.execute(Command::FindFile);
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        session.handle(Event::Paste(draft.into()));
        session.handle(key(KeyCode::Enter));
        assert!(query(&session.frame().unwrap()).starts_with(&format!("Find file: {draft}  ")));
        assert!(!session.echo().is_empty());
        assert_eq!(*session.view(), view);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().history().len(), history);
        session.handle(key(KeyCode::Char('x')));
        assert!(query(&session.frame().unwrap()).starts_with(&format!("Find file: {draft}x")));
        session.execute(Command::KeyboardQuit);
        assert_eq!(text(&session), "!retained\ntext");
        assert!(session.buffer().unwrap().is_modified());
    }
    assert_eq!(
        std::fs::read(fixture.0.join("root/source.txt")).unwrap(),
        b"retained\ntext"
    );
    assert!(!fixture.0.join("root/absent").exists());
}

// Real OS permission errors retain both typed and selected drafts, including a
// directory that cannot be listed. Restore owned permissions before assertions.
#[test]
fn denied_paths() {
    for (draft, selected) in [
        ("alpha.txt", false),
        ("alpha.txt", true),
        ("child", true),
        ("child/nested.txt", false),
    ] {
        let fixture = Fixture::new();
        let mut session = editor(&fixture);
        session.execute(Command::FindFile);
        session.handle(Event::Paste(draft.into()));
        if selected {
            session.handle(key(KeyCode::Up));
        }
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let target = fixture.0.join("root").join(if draft.starts_with("child") {
            "child"
        } else {
            draft
        });
        let permissions = std::fs::metadata(&target).unwrap().permissions();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o000)).unwrap();
        session.handle(key(KeyCode::Enter));
        let echo = session.echo().to_owned();
        let frame = session.frame().unwrap();
        std::fs::set_permissions(&target, permissions).unwrap();
        assert!(echo.contains("Permission denied"), "{draft}: {echo}");
        assert!(query(&frame).starts_with(&format!("Find file: {draft}  ")));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(*session.view(), view);
        assert_eq!(text(&session), "retained\ntext");
        session.execute(Command::KeyboardQuit);
        assert_eq!(*session.view(), view);
    }
}

// Acceptance uses the cached chosen path, never a newly sorted row. A removed
// observed file is an error rather than a silently created in-memory new file.
#[test]
fn vanished_choice() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(key(KeyCode::Down));
    std::fs::remove_file(fixture.0.join("root/alpha.txt")).unwrap();
    fixture.write("root/aaa.txt", "wrong candidate");
    session.handle(key(KeyCode::Enter));
    assert!(session.echo().contains("alpha.txt"));
    assert!(query(&session.frame().unwrap()).starts_with("Find file:   "));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    fixture.write("root/alpha.txt", "restored identity");
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), "restored identity");
    assert_eq!(
        std::fs::read(fixture.0.join("root/aaa.txt")).unwrap(),
        b"wrong candidate"
    );
}

// A child symlink remains an untraversed list leaf until explicit Return.
// Accepting a directory referent descends under the editable alias spelling.
#[test]
fn symlink_directory() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    symlink("child", fixture.0.join("root/link")).unwrap();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("link".into()));
    assert!(
        !session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().trim() == "nested.txt")
    );
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Enter));
    assert_eq!(query(&session.frame().unwrap()), "Find file: link/");
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Enter));
    assert_eq!(text(&session), "nested");
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        fixture.0.join("root/child/nested.txt")
    );
}

// Escaped display controls are not filename bytes. The accepted UTF-8 identity
// opens its original name without inserting escaped text or rewriting disk.
#[test]
fn escaped_choice() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let file = fixture.write("root/line\n界.txt", "exact identity");
    session.execute(Command::FindFile);
    session.handle(Event::Paste("line".into()));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|row| row.text().trim() == "line\\n界.txt")
    );
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Enter));
    assert_eq!(session.buffer().unwrap().file().unwrap().path(), file);
    assert_eq!(text(&session), "exact identity");
    assert_eq!(std::fs::read(file).unwrap(), b"exact identity");
}

// Unsupported byte names stay visible but cannot open a replacement path.
// Their failed selection keeps the draft, original identity and cancellation.
#[test]
fn unsupported_choice() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let file = fixture
        .0
        .join("root")
        .join(OsString::from_vec(b"bad-\xff".to_vec()));
    std::fs::write(&file, "raw identity").unwrap();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("bad-".into()));
    session.handle(key(KeyCode::Up));
    session.handle(key(KeyCode::Enter));
    assert!(session.echo().contains("unsupported non-UTF-8 filename"));
    assert!(query(&session.frame().unwrap()).starts_with("Find file: bad-  "));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.execute(Command::KeyboardQuit);
    assert_eq!(text(&session), "retained\ntext");
    assert_eq!(std::fs::read(file).unwrap(), b"raw identity");
}
