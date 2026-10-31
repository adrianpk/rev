#![forbid(unsafe_code)]

#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "bookmarks/list.rs"]
mod list;
#[path = "bookmarks/locations.rs"]
mod locations;
#[path = "bookmarks/persistence.rs"]
mod persistence;
#[path = "bookmarks/reopening.rs"]
mod reopening;

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
        let before = (
            text(&s),
            s.buffer().unwrap().state(),
            s.view().clone(),
            s.buffer().unwrap().mark(),
        );
        argument(&mut s, BookmarkCommand::Set, "here");
        assert_eq!(text(&s), before.0, "{label}");
        assert_eq!(s.buffer().unwrap().state(), before.1, "{label}");
        assert_eq!(s.view(), &before.2, "{label}");
        assert_eq!(s.buffer().unwrap().mark(), before.3, "{label}");
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
    s.replace(0..2, "").unwrap();
    s.load(&f.0.join("other")).unwrap();
    let origin = s.view().clone();
    argument(&mut s, BookmarkCommand::Jump, "source");
    assert!(s.echo().contains("stale"));
    assert_eq!(s.view(), &origin);
    assert_eq!(
        s.bookmarks().get("source").unwrap().resolution(),
        rev::bookmark::Resolution::Stale
    );
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

// The session cap rejects only new identities: replacements still work at the
// limit, and deletion frees one slot without recycling the deleted identity.
#[test]
fn bounded_collection() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "")).unwrap();
    for index in 0..rev::bookmark::MAX_ENTRIES {
        argument(&mut s, BookmarkCommand::Set, &format!("name-{index}"));
    }
    let original = s.bookmarks().get("name-0").unwrap().id();
    argument(&mut s, BookmarkCommand::Set, "overflow");
    assert!(s.echo().contains("full"));
    assert_eq!(s.bookmarks().len(), rev::bookmark::MAX_ENTRIES);
    cancel(&mut s);
    argument(&mut s, BookmarkCommand::Set, "name-0");
    assert_eq!(s.bookmarks().get("name-0").unwrap().id(), original);
    argument(&mut s, BookmarkCommand::Delete, "name-0");
    argument(&mut s, BookmarkCommand::Set, "overflow");
    assert_eq!(s.bookmarks().len(), rev::bookmark::MAX_ENTRIES);
    assert_ne!(s.bookmarks().get("overflow").unwrap().id(), original);
}

// Read-only and externally removed live sources remain navigable. Once closed,
// unavailable targets retain the origin and the stored record without guessing.
#[test]
fn visit_failures() {
    use std::os::unix::fs::PermissionsExt;
    let f = fixture::Fixture::new();
    let path = f.write("readonly", "retained");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o444)).unwrap();
    let mut s = session();
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "readonly");
    let point = s.view().point;
    s.load(&f.write("origin", "origin")).unwrap();
    argument(&mut s, BookmarkCommand::Jump, "readonly");
    assert_eq!(s.view().point, point);
    std::fs::remove_file(&path).unwrap();
    s.load(&f.0.join("origin")).unwrap();
    argument(&mut s, BookmarkCommand::Jump, "readonly");
    assert_eq!(s.view().point, point);
    assert_eq!(text(&s), "retained");
    s.execute(Command::Buffer(rev::command::BufferCommand::Kill));
    key(&mut s, KeyCode::Enter);
    s.load(&f.0.join("origin")).unwrap();
    let origin = s.view().clone();
    argument(&mut s, BookmarkCommand::Jump, "readonly");
    assert_eq!(s.view(), &origin);
    assert!(s.echo().contains("unavailable"));
    cancel(&mut s);
    assert_eq!(s.bookmarks().get("readonly").unwrap().point(), point);
}

// Completion is vertical, lexical and literal; explicit navigation accepts the
// selected identity, Tab only fills it, and cancel/resize preserve the source.
#[test]
fn vertical_names() {
    use rev::display::Band;
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "a\nb\n")).unwrap();
    argument(&mut s, BookmarkCommand::Set, "alpha");
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "alpine");
    let end = s.view().point;
    s.execute(Command::Motion(Motion::BufferStart));
    let origin = s.view().clone();
    s.execute(Command::Bookmark(BookmarkCommand::Jump));
    s.handle(Event::Paste("al".into()));
    let frame = s.frame().unwrap();
    let labels: Vec<_> = frame
        .rows()
        .iter()
        .filter(|row| row.band() == Band::Completion)
        .map(|row| row.text().trim())
        .filter(|name| !name.is_empty())
        .collect();
    assert_eq!(labels, ["alpha", "alpine"]);
    key(&mut s, KeyCode::Down);
    key(&mut s, KeyCode::Tab);
    assert_eq!(s.view(), &origin);
    s.handle(Event::Resize(0, 0));
    s.frame().unwrap();
    s.handle(Event::Resize(80, 24));
    s.frame().unwrap();
    key(&mut s, KeyCode::Enter);
    assert_eq!(s.view().point, end);
    s.execute(Command::Bookmark(BookmarkCommand::Delete));
    key(&mut s, KeyCode::Down);
    cancel(&mut s);
    assert_eq!(s.bookmarks().len(), 2);
    assert_eq!(s.view().point, end);
    s.execute(Command::Bookmark(BookmarkCommand::Delete));
    key(&mut s, KeyCode::Down);
    key(&mut s, KeyCode::Enter);
    assert!(s.bookmarks().get("alpine").is_none());
    assert!(s.bookmarks().get("alpha").is_some());
}

// Only implemented commands enter the native catalog; factory chords preserve
// uppercase no-overwrite and coexist with existing rectangle bindings.
#[test]
fn bookmark_bindings() {
    use rev::input::{Input, Outcome};
    for (suffix, command) in [
        ('m', BookmarkCommand::Set),
        ('M', BookmarkCommand::SetNoOverwrite),
        ('b', BookmarkCommand::Jump),
    ] {
        let mut input = Input::default();
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
        )));
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('r'),
            KeyModifiers::NONE,
        )));
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(suffix),
                KeyModifiers::NONE
            ))),
            Outcome::Command(Command::Bookmark(command))
        );
        assert_eq!(
            Command::named(command.name()),
            Some(Command::Bookmark(command))
        );
    }
    for command in [
        BookmarkCommand::Delete,
        BookmarkCommand::Save,
        BookmarkCommand::Reload,
        BookmarkCommand::List,
        BookmarkCommand::MenuDelete,
        BookmarkCommand::MenuUnmark,
        BookmarkCommand::MenuExecute,
        BookmarkCommand::MenuVisit,
    ] {
        assert_eq!(
            Command::named(command.name()),
            Some(Command::Bookmark(command))
        );
    }
    for name in [
        "bookmark-next",
        "bookmark-previous",
        "bookmark-markers-mode",
    ] {
        assert_eq!(Command::named(name), None);
    }
}

// An empty set draft uses the visiting basename, not a canonical alias basename
// or a unique buffer label. M-x and factory bindings share the same handler.
#[test]
fn basename_default() {
    let f = fixture::Fixture::new();
    let path = f.write("actual", "retained");
    let alias = f.0.join("alias");
    std::os::unix::fs::symlink(path, &alias).unwrap();
    let mut s = session();
    s.load(&alias).unwrap();
    s.execute(Command::ExecuteExtended);
    s.handle(Event::Paste("bookmark-set".into()));
    key(&mut s, KeyCode::Enter);
    key(&mut s, KeyCode::Enter);
    assert!(s.bookmarks().get("alias").is_some());
    assert!(s.bookmarks().get("actual").is_none());
    assert!(!s.buffer().unwrap().is_modified());
}

// Synthetic native result views cannot become bookmark sources even when the
// displayed rows describe file locations. Their text and collection stay intact.
#[test]
fn synthetic_source() {
    use rev::buffer::Buffers;
    use rev::command::OrgCommand;
    use rev::view::View;
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* TODO retained\n").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    s.execute(Command::Mode(rev::buffer::Mode::Org));
    s.execute(Command::Org(OrgCommand::TodoList));
    key(&mut s, KeyCode::Enter);
    s.handle(Event::Paste("buffers".into()));
    key(&mut s, KeyCode::Enter);
    let until = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(std::time::Instant::now()).is_none() {
            break;
        }
        assert!(std::time::Instant::now() < until);
        s.expire_input(std::time::Instant::now() + std::time::Duration::from_millis(10));
        std::thread::yield_now();
    }
    assert!(s.query_agenda().is_some());
    let before = text(&s);
    s.execute(Command::Bookmark(BookmarkCommand::Set));
    assert!(s.bookmarks().is_empty());
    assert_eq!(text(&s), before);
    assert!(s.echo().contains("Read-only query agenda"));
}

// Name matching and geometry honor the effective minibuffer settings; custom
// candidate/accept bindings target the name owner rather than editing source.
#[test]
fn configured_completion() {
    use rev::{buffer::Buffers, config::prepare, display::Band, view::View};
    for (mode, expected, deleted) in [
        ("prefix-then-substring", ["mid-a", "mid-z"], "mid-z"),
        ("prefix", ["mid-a", "mid-z"], "mid-z"),
        ("substring", ["amid", "mid-a"], "mid-a"),
    ] {
        let f = fixture::Fixture::new();
        let config = format!(
            "minibuffer.match = '{mode}'\nminibuffer.candidate_limit = 2\n[keys.minibuffer]\n'M-j' = 'minibuffer-next'\n'M-a' = 'minibuffer-accept'\n"
        );
        let startup = prepare(&f.write("config.toml", &config));
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&f.write("source", "retained")).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
        for name in ["mid-z", "amid", "mid-a"] {
            argument(&mut s, BookmarkCommand::Set, name);
        }
        let origin = s.view().clone();
        s.execute(Command::Bookmark(BookmarkCommand::Delete));
        s.handle(Event::Paste("mid".into()));
        let frame = s.frame().unwrap();
        let labels: Vec<_> = frame
            .rows()
            .iter()
            .filter(|row| row.band() == Band::Completion)
            .map(|row| row.text().trim())
            .collect();
        assert_eq!(labels, expected, "{mode}");
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::ALT,
        )));
        s.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT,
        )));
        assert!(s.bookmarks().get(deleted).is_none(), "{mode}");
        assert_eq!(s.view(), &origin, "{mode}");
        assert_eq!(text(&s), "retained");
    }
}
