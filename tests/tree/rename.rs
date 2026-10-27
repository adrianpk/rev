use std::fs;

use rev::buffer::edit::{Point, Replacement};
use rev::buffer::{Buffer, Buffers, SaveRequest};
use rev::coordinates::Scalar;
use rev::file::Rename;

use super::{Fixture, choose};

#[path = "rename_policy.rs"]
mod policy;

pub(super) fn insert(buffer: &mut Buffer, text: &str) {
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: text.into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
}
pub(super) fn text(buffer: &Buffer) -> String {
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Only disk success relocates the canonical owner. Rename keeps dirty text,
// mark/history/revision, fixes the path index and makes a later save use new name.
#[test]
fn renamed_file() {
    let fixture = Fixture::new();
    fixture.file("old.txt");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old.txt")).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    insert(buffer, "界");
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(1),
        })
        .unwrap();
    let stamp = buffer.stamp();
    let state = buffer.state();
    let mark = buffer.mark();
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    tree.move_selection(true);
    let renamed = buffers
        .rename(tree.prepare_rename().unwrap(), "new.txt")
        .unwrap();
    tree.renamed(&renamed).unwrap();
    let buffer = buffers.get(id).unwrap();
    assert_eq!(buffer.file().unwrap().path(), fixture.path("new.txt"));
    assert_eq!(buffer.name(), "new.txt");
    assert_eq!(buffer.stamp(), stamp);
    assert_eq!(buffer.state(), state);
    assert_eq!(buffer.mark(), mark);
    assert!(buffer.mark_active());
    assert_eq!(buffer.history().len(), 1);
    assert!(buffer.is_modified());
    assert_eq!(tree.selected().unwrap().path(), fixture.path("new.txt"));
    assert!(!fixture.path("old.txt").exists());
    assert_eq!(buffers.visit(&fixture.path("new.txt")).unwrap(), id);
    assert_ne!(buffers.visit(&fixture.path("old.txt")).unwrap(), id);
    assert!(matches!(
        buffers.request_save(id, None).unwrap(),
        SaveRequest::Ready(_)
    ));
    assert!(buffers.save(id).unwrap().is_complete());
    assert_eq!(
        fs::read(fixture.path("new.txt")).unwrap(),
        "界retained".as_bytes()
    );
    assert!(!fixture.path("old.txt").exists());
}

// Directory mapping includes loaded descendants and intended missing files,
// while path-component boundaries exclude similarly prefixed sibling directories.
#[test]
fn renamed_directory() {
    let fixture = Fixture::new();
    fixture.directory("old/sub");
    fixture.directory("old-other");
    fixture.file("old/a.txt");
    fixture.file("old/sub/b.txt");
    fixture.file("old-other/a.txt");
    let mut buffers = Buffers::new().unwrap();
    let a = buffers.visit(&fixture.path("old/a.txt")).unwrap();
    let b = buffers.visit(&fixture.path("old/sub/b.txt")).unwrap();
    let pending = buffers.visit(&fixture.path("old/sub/pending.txt")).unwrap();
    let untouched = buffers.visit(&fixture.path("old-other/a.txt")).unwrap();
    insert(buffers.get_mut(a).unwrap(), "!");
    let stamp = buffers.get(a).unwrap().stamp();
    let mut tree = fixture.tree();
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("old"));
    let rename = tree.prepare_rename().unwrap();
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("old/sub"));
    tree.expand().unwrap();
    choose(&mut tree, &fixture.path("old/sub/b.txt"));
    let renamed = buffers.rename(rename, "moved").unwrap();
    tree.renamed(&renamed).unwrap();
    for (id, path) in [
        (a, "moved/a.txt"),
        (b, "moved/sub/b.txt"),
        (pending, "moved/sub/pending.txt"),
        (untouched, "old-other/a.txt"),
    ] {
        assert_eq!(
            buffers.get(id).unwrap().file().unwrap().path(),
            fixture.path(path)
        );
        assert_eq!(buffers.visit(&fixture.path(path)).unwrap(), id);
    }
    assert_eq!(buffers.get(a).unwrap().stamp(), stamp);
    assert!(!buffers.get(pending).unwrap().file().unwrap().is_present());
    assert_eq!(
        tree.selected().unwrap().path(),
        fixture.path("moved/sub/b.txt")
    );
    assert!(
        tree.entries()
            .iter()
            .any(|entry| entry.path() == fixture.path("moved/sub") && entry.expanded())
    );
    assert!(matches!(
        buffers.request_save(a, None).unwrap(),
        SaveRequest::Ready(_)
    ));
    assert!(buffers.save(a).unwrap().is_complete());
    assert_eq!(fs::read(fixture.path("moved/a.txt")).unwrap(), b"!retained");
    assert!(!fixture.path("old").exists());
}

// Occupied disk entries and live intended destinations are rejected before
// rename. Exact disk bytes, dirty text and canonical source association survive.
#[test]
fn rename_conflicts() {
    use std::io::ErrorKind;
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fixture.file("old");
    fixture.file("occupied");
    fixture.directory("directory");
    symlink(fixture.path("absent"), fixture.path("link")).unwrap();
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&fixture.path("old")).unwrap();
    insert(buffers.get_mut(id).unwrap(), "!");
    let stamp = buffers.get(id).unwrap().stamp();
    for target in ["occupied", "directory", "link", "old"] {
        let error = buffers
            .rename(Rename::capture(&fixture.path("old")).unwrap(), target)
            .err()
            .unwrap();
        assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    }
    let reserved = buffers.visit(&fixture.path("reserved")).unwrap();
    assert_eq!(
        buffers
            .rename(Rename::capture(&fixture.path("old")).unwrap(), "reserved")
            .err()
            .unwrap()
            .kind(),
        ErrorKind::AlreadyExists
    );
    assert_eq!(buffers.get(id).unwrap().stamp(), stamp);
    assert_eq!(text(buffers.get(id).unwrap()), "!retained");
    assert_eq!(
        buffers.get(id).unwrap().file().unwrap().path(),
        fixture.path("old")
    );
    assert_eq!(buffers.visit(&fixture.path("reserved")).unwrap(), reserved);
    assert_eq!(fs::read(fixture.path("old")).unwrap(), b"retained");
    assert!(!fixture.path("reserved").exists());
}

// Key and M-x rename routes capture a selected path but C-g consumes the name
// stage without mutation. The dirty editor remains unchanged after cancellation.
#[test]
fn rename_prompts() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Command, TreeCommand};
    let mut session = crate::editor();
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    crate::tree(&mut session, TreeCommand::Rename);
    assert_eq!(session.echo(), "File tree is not focused");
    for named in [false, true] {
        crate::tree(&mut session, TreeCommand::Focus);
        if named {
            crate::named(&mut session, "tree-rename");
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(crate::key(KeyCode::Char('t')));
            session.handle(crate::key(KeyCode::Char('R')));
        }
        session.handle(Event::Paste("canceled-name".into()));
        session.handle(crate::key(KeyCode::Tab));
        assert_eq!(session.echo(), "Enter one name; C-g cancels");
        session.execute(Command::KeyboardQuit);
        assert!(session.tree().focused());
        assert_eq!(session.echo(), "Quit");
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(crate::text(&session), "!");
    assert_eq!(
        Command::named("tree-rename"),
        Some(Command::Tree(TreeCommand::Rename))
    );
}

#[path = "rename_failures.rs"]
mod failures;
