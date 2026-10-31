#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{BookmarkCommand, Command, History, Motion, RegionCommand};
use rev::display::Size;
use rev::session::Session;

fn session() -> Session {
    Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap()
}
fn key(session: &mut Session, code: KeyCode) {
    session.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}
fn argument(session: &mut Session, command: BookmarkCommand, name: &str) {
    session.execute(Command::Bookmark(command));
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter);
}
fn cancel(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
}
fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Replacement retains identity, refusal is atomic, and a recreated name cannot
// accidentally refer to the deleted record's historical identity.
#[test]
fn named_locations() {
    let f = fixture::Fixture::new();
    let path = f.write("界.rs", "first\nsecond\n");
    let mut s = session();
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "位置");
    let original = s.bookmarks().get("位置").unwrap().clone();
    s.execute(Command::Motion(Motion::BufferEnd));
    let end = s.view().point;
    argument(&mut s, BookmarkCommand::SetNoOverwrite, "位置");
    assert!(s.echo().contains("already exists"));
    assert_eq!(s.bookmarks().get("位置"), Some(&original));
    cancel(&mut s);
    argument(&mut s, BookmarkCommand::Set, "位置");
    assert_eq!(s.bookmarks().get("位置").unwrap().id(), original.id());
    assert_eq!(s.bookmarks().get("位置").unwrap().point(), end);
    argument(&mut s, BookmarkCommand::Set, "other");
    assert_eq!(s.bookmarks().len(), 2);
    argument(&mut s, BookmarkCommand::Delete, "位置");
    argument(&mut s, BookmarkCommand::Set, "位置");
    assert_ne!(s.bookmarks().get("位置").unwrap().id(), original.id());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "first\nsecond\n");
}

// Bookmarks operate on modified and intended new files without saving bytes,
// changing mark/dirty state or contributing a source undo transaction.
#[test]
fn source_preserved() {
    let f = fixture::Fixture::new();
    for (label, existing) in [("existing", true), ("intended", false)] {
        let path = if existing {
            f.write(label, "saved")
        } else {
            f.0.join(label)
        };
        let mut s = session();
        s.load(&path).unwrap();
        s.execute(Command::Region(RegionCommand::Set));
        s.insert('界');
        let before = (text(&s), s.buffer().unwrap().state(), s.view().clone());
        argument(&mut s, BookmarkCommand::Set, "here");
        assert_eq!(text(&s), before.0, "{label}");
        assert_eq!(s.buffer().unwrap().state(), before.1, "{label}");
        assert_eq!(s.view(), &before.2, "{label}");
        argument(&mut s, BookmarkCommand::Delete, "here");
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), if existing { "saved" } else { "" }, "{label}");
        if existing {
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "saved");
        } else {
            assert!(!path.exists());
        }
    }
}

// Canonical aliases select the existing live owner and cross-file jumps retain
// unsaved bytes. A stale revision is rejected before changing the origin view.
#[test]
fn checked_visits() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "source\n");
    let alias = f.0.join("alias");
    std::os::unix::fs::symlink(&path, &alias).unwrap();
    let mut s = session();
    s.load(&path).unwrap();
    s.insert('X');
    argument(&mut s, BookmarkCommand::Set, "source");
    let source = s.view().point;
    s.load(&alias).unwrap();
    assert_eq!(s.view().point.stamp.buffer, source.stamp.buffer);
    s.load(&f.write("other", "elsewhere")).unwrap();
    argument(&mut s, BookmarkCommand::Jump, "source");
    assert_eq!(s.view().point, source);
    assert_eq!(text(&s), "Xsource\n");
    assert!(s.buffer().unwrap().is_modified());
    s.insert('Y');
    s.load(&f.0.join("other")).unwrap();
    let origin = s.view().clone();
    argument(&mut s, BookmarkCommand::Jump, "source");
    assert!(s.echo().contains("stale"));
    assert_eq!(s.view(), &origin);
    assert_eq!(s.bookmarks().get("source").unwrap().point(), source);
    cancel(&mut s);
}

// Name admission is byte bounded, case sensitive and literal. Failed input and
// unnamed-source admission retain the collection and source unchanged.
#[test]
fn name_admission() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.execute(Command::Bookmark(BookmarkCommand::Set));
    assert!(s.echo().contains("file-backed"));
    assert!(s.bookmarks().is_empty());
    s.load(&f.write("source", "")).unwrap();
    for name in [" name ", "Name", "name", &"é".repeat(128)] {
        argument(&mut s, BookmarkCommand::Set, name);
        assert!(s.bookmarks().get(name).is_some(), "{name}");
    }
    let before: Vec<_> = s.bookmarks().iter().cloned().collect();
    argument(&mut s, BookmarkCommand::Set, &"é".repeat(129));
    assert!(s.echo().contains("1..256"));
    cancel(&mut s);
    assert_eq!(s.bookmarks().iter().cloned().collect::<Vec<_>>(), before);
    s.execute(Command::Bookmark(BookmarkCommand::Set));
    cancel(&mut s);
    assert_eq!(s.bookmarks().iter().cloned().collect::<Vec<_>>(), before);
}
