use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;

use rev::tree::CreateKind;

use super::Fixture;

// Preparation creates nothing. Exclusive publication produces an empty file or
// directory, and reconciliation reveals/selects it without opening a buffer.
#[test]
fn creation() {
    for (kind, name) in [
        (CreateKind::File, "new.unknown"),
        (CreateKind::Directory, "new folder"),
    ] {
        let fixture = Fixture::new();
        let mut tree = fixture.tree();
        let creation = tree.prepare_create(kind).unwrap();
        assert_eq!(creation.directory(), fixture.0);
        assert_eq!(creation.kind(), kind);
        assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
        let path = creation.publish(name).unwrap();
        assert_eq!(path, fixture.path(name));
        assert_eq!(
            fs::metadata(&path).unwrap().is_dir(),
            kind == CreateKind::Directory
        );
        if kind == CreateKind::File {
            assert_eq!(fs::read(&path).unwrap(), b"");
        }
        tree.reveal_created(&path).unwrap();
        assert_eq!(tree.entries().len(), 2);
        assert_eq!(tree.selected().unwrap().path(), path);
        assert!(tree.entries()[0].expanded());
    }
}

// Only one literal filename component is admitted, before any disk mutation.
// UTF-8 and spaces remain valid names rather than paths or command syntax.
#[test]
fn creation_names() {
    let fixture = Fixture::new();
    let tree = fixture.tree();
    for kind in [CreateKind::File, CreateKind::Directory] {
        for name in [
            "",
            " ",
            ".",
            "..",
            "/absolute",
            "a/b",
            "a/",
            "./a",
            "a//b",
            "bad\0name",
            "bad\nname",
        ] {
            let error = tree
                .prepare_create(kind)
                .unwrap()
                .publish(name)
                .unwrap_err();
            assert_eq!(error.kind(), ErrorKind::InvalidInput, "{kind:?} {name:?}");
        }
    }
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
    let name = "界 e\u{301}.other";
    let path = tree
        .prepare_create(CreateKind::File)
        .unwrap()
        .publish(name)
        .unwrap();
    assert_eq!(path.file_name().unwrap(), name);
}

// Conflicts include regular files, directories and dangling symlinks. Both
// create kinds preserve exact existing data and never follow a target link.
#[test]
fn creation_conflicts() {
    let fixture = Fixture::new();
    fixture.file("keep");
    fixture.directory("directory");
    symlink(fixture.path("keep"), fixture.path("link")).unwrap();
    symlink(fixture.path("missing"), fixture.path("dangling")).unwrap();
    let tree = fixture.tree();
    for kind in [CreateKind::File, CreateKind::Directory] {
        for name in ["keep", "directory", "link", "dangling"] {
            assert_eq!(
                tree.prepare_create(kind)
                    .unwrap()
                    .publish(name)
                    .unwrap_err()
                    .kind(),
                ErrorKind::AlreadyExists
            );
        }
    }
    assert_eq!(fs::read(fixture.path("keep")).unwrap(), b"retained");
    assert_eq!(
        fs::read_link(fixture.path("dangling")).unwrap(),
        fixture.path("missing")
    );
    assert!(!fixture.path("missing").exists());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 4);
}

// Bound/named creation use focused name prompts. C-g cancels without creating
// entries or deactivating the dirty editor's point/history; Tab is only a hint.
#[test]
fn creation_prompts() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Command, TreeCommand};
    let mut session = crate::editor();
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    let point = session.view().point;
    for command in [TreeCommand::CreateFile, TreeCommand::CreateDirectory] {
        crate::tree(&mut session, command);
        assert_eq!(session.echo(), "File tree is not focused");
    }
    for named in [false, true] {
        crate::tree(&mut session, TreeCommand::Focus);
        if named {
            crate::named(&mut session, "tree-create-directory");
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(crate::key(KeyCode::Char('t')));
            session.handle(crate::key(KeyCode::Char('n')));
        }
        session.handle(Event::Paste("never-created".into()));
        session.handle(crate::key(KeyCode::Tab));
        assert_eq!(session.echo(), "Enter one name; C-g cancels");
        session.execute(Command::KeyboardQuit);
        assert!(session.tree().focused());
        assert_eq!(session.echo(), "Quit");
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.view().point, point);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(crate::text(&session), "!");
}

#[path = "create_failures.rs"]
mod failures;
