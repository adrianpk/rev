use std::fs;

use rev::buffer::{Buffers, SaveRequest};
use rev::file::{DeleteDecision, Deletion};

use super::rename::{insert, text};
use super::{Fixture, choose};

fn confirmed(deletion: Deletion) -> Deletion {
    let DeleteDecision::Ready(deletion) = deletion.confirm().unwrap() else {
        panic!("unexpected recursive confirmation");
    };
    deletion
}

// Both clean and dirty visiting text survive deletion with their exact history,
// state and canonical reservation. Only an explicit later save recreates disk.
#[test]
fn retained_file() {
    for dirty in [false, true] {
        let fixture = Fixture::new();
        fixture.file("file.txt");
        let path = fixture.path("file.txt");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        if dirty {
            insert(buffers.get_mut(id).unwrap(), "界");
        }
        let buffer = buffers.get_mut(id).unwrap();
        buffer
            .activate_mark(rev::buffer::edit::Point {
                stamp: buffer.stamp(),
                position: rev::coordinates::Scalar(0),
            })
            .unwrap();
        let mark = buffer.mark();
        let stamp = buffer.stamp();
        let state = buffer.state();
        let expected = text(buffer);
        let history = buffer.history().len();
        let stale = buffers.prepare_save(id, None).unwrap();
        let mut tree = fixture.tree();
        tree.expand().unwrap();
        choose(&mut tree, &path);
        let deleted = buffers.delete(confirmed(tree.prepare_delete().unwrap()));
        assert!(deleted.is_complete());
        tree.refresh().unwrap();
        assert_eq!(tree.selected().unwrap().path(), fixture.0);
        let buffer = buffers.get(id).unwrap();
        assert_eq!(text(buffer), expected);
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.state(), state);
        assert_eq!(buffer.mark(), mark);
        assert!(buffer.mark_active());
        assert_eq!(buffer.history().len(), history);
        assert!(!buffer.file().unwrap().is_present());
        assert!(buffer.file().unwrap().observation().is_none());
        assert!(buffer.is_modified());
        assert!(buffer.save_warning().unwrap().contains("explicit save"));
        assert_eq!(buffers.visit(&path).unwrap(), id);
        assert!(!path.exists());
        assert!(!buffers.publish_save(stale).is_complete());
        assert!(!path.exists());
        assert!(matches!(
            buffers.request_save(id, None).unwrap(),
            SaveRequest::Ready(_)
        ));
        assert!(buffers.save(id).unwrap().is_complete());
        assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
        assert!(!buffers.get(id).unwrap().is_modified());
    }
}

// Nonempty removal consumes two separate decisions. Dropping either stage
// changes nothing; successful recursion marks only actual descendant owners.
#[test]
fn recursive_decision() {
    let fixture = Fixture::new();
    fixture.directory("dir/sub");
    fixture.directory("dir-other");
    fixture.file("dir/first");
    fixture.file("dir/sub/second");
    fixture.file("dir-other/first");
    let mut buffers = Buffers::new().unwrap();
    let a = buffers.visit(&fixture.path("dir/first")).unwrap();
    let b = buffers.visit(&fixture.path("dir/sub/second")).unwrap();
    let other = buffers.visit(&fixture.path("dir-other/first")).unwrap();
    let absent = buffers.visit(&fixture.path("dir/sub/absent")).unwrap();
    insert(buffers.get_mut(b).unwrap(), "!");
    for _ in 0..2 {
        let deletion = Deletion::capture(&fixture.path("dir")).unwrap();
        assert!(deletion.message().starts_with("Delete "));
        let DeleteDecision::Recursive(deletion) = deletion.confirm().unwrap() else {
            panic!("missing second confirmation");
        };
        assert!(
            deletion
                .message()
                .contains("contains children; recursively delete")
        );
        drop(deletion);
        assert_eq!(fs::read(fixture.path("dir/first")).unwrap(), b"retained");
    }
    let DeleteDecision::Recursive(deletion) = Deletion::capture(&fixture.path("dir"))
        .unwrap()
        .confirm()
        .unwrap()
    else {
        panic!();
    };
    assert!(buffers.delete(confirmed(deletion)).is_complete());
    for id in [a, b] {
        assert!(!buffers.get(id).unwrap().file().unwrap().is_present());
        assert!(buffers.get(id).unwrap().is_modified());
    }
    assert_eq!(text(buffers.get(b).unwrap()), "!retained");
    assert!(!buffers.get(absent).unwrap().is_modified());
    assert!(buffers.get(other).unwrap().file().unwrap().is_present());
    assert!(!buffers.get(other).unwrap().is_modified());
    assert!(!fixture.path("dir").exists());
    assert_eq!(buffers.visit(&fixture.path("dir/first")).unwrap(), a);
    assert!(!buffers.save(b).unwrap().is_complete());
    assert!(!fixture.path("dir").exists());
}

// Empty directories need one decision. Children appearing after that decision
// are protected by nonrecursive removal, not silently granted recursive consent.
#[test]
fn empty_directory() {
    let fixture = Fixture::new();
    fixture.directory("empty");
    let mut buffers = Buffers::new().unwrap();
    assert!(
        buffers
            .delete(confirmed(
                Deletion::capture(&fixture.path("empty")).unwrap()
            ))
            .is_complete()
    );
    fixture.directory("changed");
    let deletion = confirmed(Deletion::capture(&fixture.path("changed")).unwrap());
    fixture.file("changed/late");
    let deleted = buffers.delete(deletion);
    assert!(!deleted.is_complete());
    assert!(deleted.error().unwrap().to_string().contains("changed"));
    assert_eq!(fs::read(fixture.path("changed/late")).unwrap(), b"retained");
}

// Named and bound deletion share focused cancellation. Prompt drafts and key
// repeats cannot become editing text; decision frames place the cursor in the echo area.
#[test]
fn cancel_delete() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Command, TreeCommand};
    let mut session = crate::editor();
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    crate::tree(&mut session, TreeCommand::Delete);
    assert_eq!(session.echo(), "File tree is not focused");
    for named in [false, true] {
        crate::tree(&mut session, TreeCommand::Focus);
        if named {
            crate::named(&mut session, "tree-delete");
        } else {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('x'),
                KeyModifiers::CONTROL,
            )));
            session.handle(crate::key(KeyCode::Char('t')));
            session.handle(crate::key(KeyCode::Char('D')));
        }
        assert!(session.echo().starts_with("Delete "));
        assert_eq!(session.frame().unwrap().cursor().unwrap().row, 7);
        session.handle(Event::Paste("y".into()));
        assert!(session.echo().contains("Delete "));
        session.execute(Command::KeyboardQuit);
        assert!(session.tree().focused());
        assert_eq!(session.echo(), "Quit");
    }
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(crate::text(&session), "!");
    assert_eq!(
        Command::named("tree-delete"),
        Some(Command::Tree(TreeCommand::Delete))
    );
}

#[path = "delete_failures.rs"]
mod failures;
