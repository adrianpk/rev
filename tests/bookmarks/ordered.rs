use super::*;
use rev::command::{BufferCommand, Split, WindowCommand};

fn at(s: &mut Session, position: usize) {
    s.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..position {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
}
fn step(s: &mut Session, backward: bool, name: &str, point: usize) {
    s.execute(Command::Bookmark(if backward {
        BookmarkCommand::Previous
    } else {
        BookmarkCommand::Next
    }));
    assert_eq!(s.echo(), format!("Bookmark {name:?}"));
    assert_eq!(s.view().point.position.0, point);
}

// Source position wins over name ordering; same-position records are distinct
// stable steps in both directions, including wrap and repeated M-x dispatch.
#[test]
fn ordered_ties() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "abcdef")).unwrap();
    for (position, name) in [(4, "middle"), (1, "B"), (6, "end"), (1, "A")] {
        at(&mut s, position);
        argument(&mut s, BookmarkCommand::Set, name);
    }
    at(&mut s, 1);
    for (name, point) in [("A", 1), ("B", 1), ("middle", 4), ("end", 6), ("A", 1)] {
        step(&mut s, false, name, point);
    }
    step(&mut s, true, "end", 6);
    step(&mut s, true, "middle", 4);
    step(&mut s, true, "B", 1);
    step(&mut s, true, "A", 1);
    step(&mut s, true, "end", 6);
    at(&mut s, 2);
    step(&mut s, false, "middle", 4);
    at(&mut s, 2);
    step(&mut s, true, "B", 1);
    at(&mut s, 1);
    for name in ["A", "B", "middle"] {
        s.execute(Command::ExecuteExtended);
        s.handle(Event::Paste("bookmark-next".into()));
        key(&mut s, KeyCode::Enter);
        assert_eq!(s.echo(), format!("Bookmark {name:?}"));
    }
    assert_eq!(text(&s), "abcdef");
    assert!(!s.buffer().unwrap().is_modified());
}

// Point movement and revisions reset same-point repetition even after returning
// to the old scalar. Collection mutation and pane focus do the same.
#[test]
fn iteration_invalidation() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "abcdef")).unwrap();
    for name in ["A", "B"] {
        argument(&mut s, BookmarkCommand::Set, name);
    }
    step(&mut s, false, "A", 0);
    step(&mut s, false, "B", 0);
    s.execute(Command::Motion(Motion::ForwardChar));
    s.execute(Command::Motion(Motion::BackwardChar));
    step(&mut s, false, "A", 0);
    step(&mut s, false, "B", 0);
    s.replace(4..4, "X").unwrap();
    step(&mut s, false, "A", 0);
    step(&mut s, false, "B", 0);
    argument(&mut s, BookmarkCommand::Set, "C");
    step(&mut s, false, "A", 0);
    s.execute(Command::Split(Split::Right));
    step(&mut s, false, "A", 0);
    step(&mut s, false, "B", 0);
    s.execute(Command::Window(WindowCommand::Other));
    step(&mut s, false, "A", 0);
    assert_eq!(text(&s), "abcdXef");
}

// Traversal never opens another file or guesses unresolved/stale evidence.
// Current-file aliases retain the canonical live owner and unsaved text.
#[test]
fn eligible_file() {
    let f = fixture::Fixture::new();
    let a = f.write("A", "abcdef");
    let b = f.write("B", "other");
    let alias = f.0.join("alias");
    std::os::unix::fs::symlink(&a, &alias).unwrap();
    let mut s = session();
    s.load(&a).unwrap();
    argument(&mut s, BookmarkCommand::Set, "local");
    at(&mut s, 3);
    argument(&mut s, BookmarkCommand::Set, "stale");
    s.replace(2..4, "").unwrap();
    s.load(&b).unwrap();
    argument(&mut s, BookmarkCommand::Set, "other");
    s.load(&alias).unwrap();
    let id = s.buffer().unwrap().id();
    step(&mut s, false, "local", 0);
    step(&mut s, true, "local", 0);
    assert_eq!(s.buffer().unwrap().id(), id);
    assert_eq!(text(&s), "abef");
    assert!(s.bookmarks().get("other").is_some());
    argument(&mut s, BookmarkCommand::Delete, "local");
    let before = s.view().clone();
    s.execute(Command::Bookmark(BookmarkCommand::Next));
    assert!(s.echo().contains("No resolved bookmarks"));
    assert_eq!(s.view(), &before);
    assert!(s.bookmarks().get("stale").is_some());
    assert_eq!(std::fs::read_to_string(a).unwrap(), "abcdef");
    s.execute(Command::Buffer(BufferCommand::Kill));
    key(&mut s, KeyCode::Enter);
    key(&mut s, KeyCode::Char('n'));
    s.load(&b).unwrap();
    s.execute(Command::Buffer(BufferCommand::Kill));
    key(&mut s, KeyCode::Enter);
    let before = s.view().clone();
    s.execute(Command::Bookmark(BookmarkCommand::Next));
    assert!(s.echo().contains("file-backed"));
    assert_eq!(s.view(), &before);
}

// Loaded locations are retained but not guessed into current-file order. An
// explicit checked jump can resolve one, after which traversal admits it.
#[test]
fn unresolved_order() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "abcdef");
    let store=f.write("bookmarks.toml",&format!("version=1\n[[bookmarks]]\nname='A'\npath='{}'\nposition=2\nbefore='ab'\nafter='cdef'\nat_start=true\nat_end=true\n",path.display()));
    let startup = rev::config::prepare(&f.write(
        "config.toml",
        &format!(
            "bookmarks.file='{}'\nbookmarks.save='manual'\n",
            store.display()
        ),
    ));
    assert!(startup.diagnostic.is_none());
    let buffers = rev::buffer::Buffers::with_settings(startup.snapshot).unwrap();
    let view = rev::view::View::new(buffers.get(buffers.initial()).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    s.load(&path).unwrap();
    let before = s.view().clone();
    s.execute(Command::Bookmark(BookmarkCommand::Next));
    assert!(s.echo().contains("1 retained unresolved"));
    assert_eq!(s.view(), &before);
    assert_eq!(
        s.bookmarks().get("A").unwrap().resolution(),
        rev::bookmark::Resolution::Unresolved
    );
    argument(&mut s, BookmarkCommand::Jump, "A");
    step(&mut s, false, "A", 2);
    assert_eq!(text(&s), "abcdef");
}
