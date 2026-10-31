use super::*;
use rev::bookmark::Resolution;

fn position(s: &mut Session, at: usize) {
    s.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..at {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
}

// The checked publication uses left affinity and half-open deletion boundaries.
// Source undo/redo restores bookmark outcomes without a separate edit history.
#[test]
fn edit_outcomes() {
    for (name, point, range, replacement, expected) in [
        ("at insertion", 3, 3..3, "XX", Some(3)),
        ("before insertion", 3, 0..0, "XX", Some(5)),
        ("end boundary", 3, 1..3, "X", Some(2)),
        ("removed start", 3, 3..5, "X", None),
        ("removed interior", 3, 2..5, "", None),
        ("after deletion", 7, 2..5, "", Some(4)),
        ("EOF insertion", 10, 10..10, "X", Some(10)),
    ] {
        let f = fixture::Fixture::new();
        let mut s = session();
        s.load(&f.write("source", "0123456789")).unwrap();
        position(&mut s, point);
        argument(&mut s, BookmarkCommand::Set, "here");
        s.replace(range, replacement).unwrap();
        let record = s.bookmarks().get("here").unwrap().clone();
        assert_eq!(
            record.resolution(),
            if expected.is_some() {
                Resolution::Resolved
            } else {
                Resolution::Stale
            },
            "{name}"
        );
        assert_eq!(record.point().stamp, s.buffer().unwrap().stamp(), "{name}");
        if let Some(at) = expected {
            assert_eq!(record.point().position.0, at, "{name}");
        }
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), "0123456789", "{name}");
        assert_eq!(
            s.bookmarks().get("here").unwrap().resolution(),
            Resolution::Resolved,
            "{name}"
        );
        assert_eq!(
            s.bookmarks().get("here").unwrap().point().position.0,
            point,
            "{name}"
        );
        s.execute(Command::History(History::Redo));
        assert_eq!(
            s.bookmarks().get("here").unwrap().resolution(),
            record.resolution(),
            "{name}"
        );
        assert_eq!(
            s.bookmarks().get("here").unwrap().point().position,
            record.point().position,
            "{name}"
        );
    }
}

// A newly joined grapheme has no original insertion boundary. It becomes stale
// rather than moving to a neighbouring cursor; undo recovers the actual boundary.
#[test]
fn grapheme_join() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "aX")).unwrap();
    position(&mut s, 1);
    argument(&mut s, BookmarkCommand::Set, "here");
    s.replace(1..1, "\u{301}").unwrap();
    assert_eq!(
        s.bookmarks().get("here").unwrap().resolution(),
        Resolution::Stale
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(
        s.bookmarks().get("here").unwrap().resolution(),
        Resolution::Resolved
    );
    assert_eq!(s.bookmarks().get("here").unwrap().point().position.0, 1);
}

// Old source history cannot resurrect deleted names or restore a replaced
// target generation. Later records use their own mapping through the inverse.
#[test]
fn history_identity() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "abcdef")).unwrap();
    position(&mut s, 3);
    argument(&mut s, BookmarkCommand::Set, "old");
    argument(&mut s, BookmarkCommand::Set, "replace");
    let old = s.bookmarks().get("old").unwrap().id();
    s.replace(0..0, "XX").unwrap();
    argument(&mut s, BookmarkCommand::Delete, "old");
    position(&mut s, 6);
    argument(&mut s, BookmarkCommand::Set, "old");
    argument(&mut s, BookmarkCommand::Set, "replace");
    argument(&mut s, BookmarkCommand::Set, "new");
    assert_ne!(s.bookmarks().get("old").unwrap().id(), old);
    s.execute(Command::History(History::Undo));
    for name in ["old", "replace", "new"] {
        assert_eq!(
            s.bookmarks().get(name).unwrap().point().position.0,
            4,
            "{name}"
        );
    }
    argument(&mut s, BookmarkCommand::Delete, "new");
    s.execute(Command::History(History::Redo));
    assert!(s.bookmarks().get("new").is_none());
    for name in ["old", "replace"] {
        assert_eq!(s.bookmarks().get(name).unwrap().point().position.0, 6);
    }
}

// Every split observes the same native anchor; adjacent typing remains one undo
// group and bookmark observations remain coherent when older groups are evicted.
#[test]
fn splits_and_grouping() {
    let f = fixture::Fixture::new();
    let startup = rev::config::prepare(&f.write("config.toml", "history.group_limit=1"));
    let mut buffers = rev::buffer::Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&f.write("source", "abcdef")).unwrap();
    let view = rev::view::View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    position(&mut s, 3);
    argument(&mut s, BookmarkCommand::Set, "here");
    s.execute(Command::Split(rev::command::Split::Right));
    position(&mut s, 0);
    for character in ['X', 'Y', 'Z'] {
        s.insert(character);
    }
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    assert_eq!(s.bookmarks().get("here").unwrap().point().position.0, 6);
    s.execute(Command::History(History::Undo));
    assert_eq!(s.bookmarks().get("here").unwrap().point().position.0, 3);
    s.execute(Command::History(History::Redo));
    s.replace(0..0, "Q").unwrap();
    assert_eq!(s.buffer().unwrap().history().len(), 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(s.bookmarks().get("here").unwrap().point().position.0, 6);
    argument(&mut s, BookmarkCommand::Jump, "here");
    assert_eq!(s.view().point.position.0, 6);
    s.execute(Command::Window(rev::command::WindowCommand::Other));
    argument(&mut s, BookmarkCommand::Jump, "here");
    assert_eq!(s.view().point.position.0, 6);
}

fn settle(s: &mut Session) {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(std::time::Instant::now()).is_none() {
            return;
        }
        assert!(
            std::time::Instant::now() < until,
            "native work did not settle"
        );
        s.expire_input(std::time::Instant::now() + std::time::Duration::from_millis(10));
        std::thread::yield_now();
    }
}

// Native structural movement retains the actual source fragment through the
// canonical permutation, including source undo/redo, instead of marking it lost.
#[test]
fn retained_subtree() {
    let f = fixture::Fixture::new();
    let source = "* A\nbody-A\n* B\nbody-B\n";
    let mut s = session();
    s.load(&f.write("source.org", source)).unwrap();
    let old = source.find("body-B").unwrap() + 2;
    position(&mut s, old);
    argument(&mut s, BookmarkCommand::Set, "body");
    position(&mut s, source.find("* B").unwrap());
    s.execute(Command::Org(rev::command::OrgCommand::MoveSubtreeUp));
    settle(&mut s);
    let moved = text(&s);
    assert_eq!(moved, "* B\nbody-B\n* A\nbody-A\n");
    assert_eq!(
        s.bookmarks().get("body").unwrap().resolution(),
        Resolution::Resolved
    );
    assert_eq!(
        s.bookmarks().get("body").unwrap().point().position.0,
        moved.find("body-B").unwrap() + 2
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(s.bookmarks().get("body").unwrap().point().position.0, old);
    s.execute(Command::History(History::Redo));
    assert_eq!(
        s.bookmarks().get("body").unwrap().point().position.0,
        moved.find("body-B").unwrap() + 2
    );
}
