use std::fs;
use std::os::unix::fs::PermissionsExt;

use rev::buffer::Buffers;
use rev::config::prepare;
use rev::file::completion::{ENTRY_LIMIT, NAME_BYTE_LIMIT};
use rev::view::View;

use super::*;

fn configured(fixture: &config_fixture::Fixture, visible: bool) -> Session {
    let file = fixture.write("root/main.txt", "retained");
    fixture.write("root/child/deep.txt", "hidden descendant");
    let settings = prepare(&fixture.write("config.toml", &format!("tree.visible = {visible}")));
    assert!(settings.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let id = buffers.visit(&file).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap()
}

// First manual/configured presentation exposes one level only. A child with
// denied read permission proves initialization does not enumerate descendants.
#[test]
fn first_presentation() {
    for visible in [false, true] {
        let fixture = config_fixture::Fixture::new();
        let mut session = configured(&fixture, false);
        let child = fixture.0.join("root/child");
        let permissions = fs::metadata(&child).unwrap().permissions();
        fs::set_permissions(&child, fs::Permissions::from_mode(0o000)).unwrap();
        if visible {
            let b = session.buffer().unwrap();
            let settings = prepare(&fixture.write("visible.toml", "tree.visible = true"));
            let mut buffers = Buffers::with_settings(settings.snapshot).unwrap();
            let id = buffers.visit(b.file().unwrap().path()).unwrap();
            let view = View::new(buffers.get(id).unwrap());
            session = Session::new(
                buffers,
                view,
                Size {
                    width: 80,
                    height: 12,
                },
            )
            .unwrap();
        } else {
            tree(&mut session, TreeCommand::Toggle);
        }
        fs::set_permissions(&child, permissions).unwrap();
        assert!(session.tree().entries()[0].expanded());
        assert_eq!(session.tree().entries().len(), 3);
        assert!(!session.tree().entries()[1].expanded());
        assert_eq!(text(&session), "retained");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Deliberate collapse survives hide/show without another listing, even when
// the root becomes unreadable. An actual root change initializes once again.
#[test]
fn retained_initialization() {
    let fixture = config_fixture::Fixture::new();
    let mut session = configured(&fixture, false);
    let root = fixture.0.join("root");
    let permissions = fs::metadata(&root).unwrap().permissions();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o000)).unwrap();
    let failed = session.execute(Command::Tree(TreeCommand::Focus));
    fs::set_permissions(&root, permissions).unwrap();
    assert_eq!(failed, Action::Redraw);
    assert!(session.echo().contains("Permission denied"));
    assert!(!session.tree().visible());
    assert!(!session.tree().entries()[0].expanded());
    tree(&mut session, TreeCommand::Focus);
    tree(&mut session, TreeCommand::Collapse);
    tree(&mut session, TreeCommand::Toggle);
    let permissions = fs::metadata(&root).unwrap().permissions();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o000)).unwrap();
    let action = session.execute(Command::Tree(TreeCommand::Toggle));
    let frame = session.frame();
    fs::set_permissions(&root, permissions).unwrap();
    assert_eq!(action, Action::Redraw);
    frame.unwrap();
    assert_eq!(session.tree().entries().len(), 1);
    assert!(!session.tree().entries()[0].expanded());
    let other = fixture.write("other/main.txt", "other");
    session.load(&other).unwrap();
    assert_eq!(session.tree().root(), other.parent());
    assert!(session.tree().entries()[0].expanded());
    assert_eq!(session.tree().entries().len(), 2);
}

// A narrow directory opening retains initialized browsing/focus intent. A
// later cancel releases that intent, so resizing cannot steal editing focus.
#[test]
fn narrow_intent() {
    let fixture = config_fixture::Fixture::new();
    fixture.write("root/main.txt", "retained");
    let mut session = Session::empty(Size {
        width: 20,
        height: 8,
    })
    .unwrap();
    let point = session.view().point;
    session.load(&fixture.0.join("root")).unwrap();
    assert!(session.tree().visible() && !session.tree().focused());
    assert!(session.tree().entries()[0].expanded());
    session.handle(Event::Resize(80, 12));
    assert!(session.tree().focused());
    session.handle(Event::Resize(20, 8));
    session.execute(Command::KeyboardQuit);
    session.handle(Event::Resize(80, 12));
    assert!(!session.tree().focused());
    assert_eq!(session.view().point, point);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Explicit directory targets may follow aliases, but a tree symlink remains a
// leaf action. Opening a directory referent must not replace the chosen root.
#[test]
fn directory_link_leaf() {
    let fixture = config_fixture::Fixture::new();
    let mut session = configured(&fixture, false);
    let root = fixture.0.join("root");
    let link = root.join("link");
    std::os::unix::fs::symlink("child", &link).unwrap();
    tree(&mut session, TreeCommand::Focus);
    for _ in 0..session.tree().entries().len() {
        if session.tree().selected().unwrap().path() == link {
            break;
        }
        tree(&mut session, TreeCommand::Next);
    }
    assert_eq!(session.tree().selected().unwrap().path(), link);
    let point = session.view().point;
    tree(&mut session, TreeCommand::Open);
    assert!(session.echo().contains("expected a regular file"));
    assert_eq!(session.tree().root(), Some(root.as_path()));
    assert!(session.tree().focused());
    assert_eq!(session.view().point, point);
    assert_eq!(text(&session), "retained");
    assert!(session.buffer().unwrap().history().is_empty());
}

// Tree initialization uses finite native path-observation limits. Exact-limit
// and larger listings are visibly incomplete, never empty or silently complete.
#[test]
fn listing_limits() {
    for (count, length) in [
        (ENTRY_LIMIT - 1, 8),
        (ENTRY_LIMIT, 8),
        (ENTRY_LIMIT + 1, 8),
        (2200, 245),
    ] {
        let fixture = config_fixture::Fixture::new();
        let root = fixture.0.join("root");
        fs::create_dir(&root).unwrap();
        for index in 0..count {
            fs::write(
                root.join(format!("{index:04}{}", "x".repeat(length - 4))),
                b"",
            )
            .unwrap();
        }
        let mut session = editor();
        session.load(&root).unwrap();
        let entries = session.tree().entries();
        assert!(entries.len() <= ENTRY_LIMIT + 1);
        let bytes: usize = entries[1..]
            .iter()
            .map(|entry| entry.path().file_name().unwrap().as_encoded_bytes().len())
            .sum();
        assert!(bytes <= NAME_BYTE_LIMIT);
        let incomplete = count >= ENTRY_LIMIT || count * length > NAME_BYTE_LIMIT;
        assert_eq!(session.tree().diagnostic().is_some(), incomplete);
        assert_eq!(
            session.echo().contains("Incomplete tree listing"),
            incomplete
        );
        assert!(entries[0].expanded());
        assert!(entries.len() > 1);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
