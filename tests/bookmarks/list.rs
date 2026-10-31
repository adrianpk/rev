use super::*;
use rev::command::{BufferCommand, Split, WindowCommand};

fn open(s: &mut Session) {
    s.execute(Command::Bookmark(BookmarkCommand::List));
}
fn surface(s: &mut Session) -> String {
    s.frame()
        .unwrap()
        .rows()
        .iter()
        .map(|r| r.text())
        .collect::<Vec<_>>()
        .join("\n")
}
fn selected(s: &mut Session, name: &str) {
    assert!(
        surface(s).contains(&format!(">  {name:?}")),
        "{}",
        surface(s)
    );
}
fn kill(s: &mut Session) {
    s.execute(Command::Buffer(BufferCommand::Kill));
    key(s, KeyCode::Enter);
}
fn configured(f: &fixture::Fixture, text: &str) -> Session {
    let startup = rev::config::prepare(&f.write("config.toml", text));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = rev::buffer::Buffers::with_settings(startup.snapshot).unwrap();
    let view = rev::view::View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 16,
        },
    )
    .unwrap()
}

// A read-only native result view preserves source text, point, modified state
// and source undo while marks and row navigation act on stable collection rows.
#[test]
fn rows_and_deletions() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    let mut s = session();
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "Z");
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "A");
    s.insert('X');
    let before = (s.view().clone(), s.buffer().unwrap().state(), text(&s));
    open(&mut s);
    assert!(surface(&mut s).contains("*Bookmarks* [read-only] 2 entries"));
    selected(&mut s, "A");
    assert!(surface(&mut s).find("\"A\"").unwrap() < surface(&mut s).find("\"Z\"").unwrap());
    s.handle(Event::Paste("forbidden".into()));
    s.insert('!');
    assert!(s.replace(0..0, "no").is_err());
    assert_eq!(text(&s), before.2);
    key(&mut s, KeyCode::Char('d'));
    selected(&mut s, "Z");
    assert!(surface(&mut s).contains(" D \"A\""));
    key(&mut s, KeyCode::Up);
    key(&mut s, KeyCode::Char('u'));
    assert!(!surface(&mut s).contains(" D \"A\""));
    key(&mut s, KeyCode::Up);
    key(&mut s, KeyCode::Char('d'));
    key(&mut s, KeyCode::Char('x'));
    assert!(s.bookmarks().get("A").is_none());
    assert!(s.bookmarks().get("Z").is_some());
    selected(&mut s, "Z");
    key(&mut s, KeyCode::Char('q'));
    assert_eq!(s.view(), &before.0);
    assert_eq!(s.buffer().unwrap().state(), before.1);
    assert_eq!(text(&s), before.2);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "saved");
    assert_eq!(fs_read(&path), "saved");
}
fn fs_read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

// Cancellation and ordinary quit discard unexecuted marks, including the empty
// list path. Marking never deletes before the explicit execute action.
#[test]
fn cancel_marks() {
    for exit in ["q", "C-g", "quit"] {
        let f = fixture::Fixture::new();
        let mut s = session();
        s.load(&f.write("source", "saved")).unwrap();
        argument(&mut s, BookmarkCommand::Set, "A");
        open(&mut s);
        key(&mut s, KeyCode::Char('d'));
        match exit {
            "q" => key(&mut s, KeyCode::Char('q')),
            "C-g" => cancel(&mut s),
            _ => {
                assert_eq!(s.execute(Command::Quit), rev::session::Action::Exit);
            }
        }
        assert!(s.bookmarks().get("A").is_some());
        open(&mut s);
        assert!(!surface(&mut s).contains(" D \"A\""));
        key(&mut s, KeyCode::Char('x'));
        assert!(s.echo().contains("No bookmarks marked"));
    }
    let mut s = session();
    let before = s.view().clone();
    open(&mut s);
    key(&mut s, KeyCode::Enter);
    assert!(s.echo().contains("No bookmarks"));
    cancel(&mut s);
    assert_eq!(s.view(), &before);
}

// Mutation in another pane invalidates all staged row observations. Recreating
// a name cannot make an old mark delete its new identity or redirect a visit.
#[test]
fn stable_rows() {
    let f = fixture::Fixture::new();
    let mut s = session();
    s.load(&f.write("source", "abcdef")).unwrap();
    argument(&mut s, BookmarkCommand::Set, "A");
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "B");
    let original = s.bookmarks().get("A").unwrap().id();
    s.execute(Command::Split(Split::Right));
    let origin = s.windows().selected_id();
    open(&mut s);
    key(&mut s, KeyCode::Char('d'));
    s.execute(Command::Window(WindowCommand::Other));
    argument(&mut s, BookmarkCommand::Delete, "A");
    argument(&mut s, BookmarkCommand::Set, "A");
    let replacement = s.bookmarks().get("A").unwrap().id();
    assert_ne!(replacement, original);
    s.select_view(origin).unwrap();
    key(&mut s, KeyCode::Char('x'));
    assert!(s.echo().contains("collection changed"));
    assert_eq!(s.bookmarks().get("A").unwrap().id(), replacement);
    selected(&mut s, "B");
    key(&mut s, KeyCode::Char('x'));
    assert!(s.echo().contains("No bookmarks marked"));
    key(&mut s, KeyCode::Enter);
    assert_eq!(s.view().point.position.0, 6);
    assert!(!surface(&mut s).contains("*Bookmarks*"));
}

// A result view owns only its pane. Changed or killed origins cannot receive a
// stale return point; the current qualified replacement is retained on close.
#[test]
fn return_lifetime() {
    for change in ["edit", "close"] {
        let f = fixture::Fixture::new();
        let mut s = session();
        s.load(&f.write("source", "saved")).unwrap();
        argument(&mut s, BookmarkCommand::Set, "A");
        s.execute(Command::Split(Split::Right));
        let origin = s.windows().selected_id();
        open(&mut s);
        s.execute(Command::Window(WindowCommand::Other));
        if change == "edit" {
            s.insert('X');
        } else {
            kill(&mut s);
        }
        s.select_view(origin).unwrap();
        let current = s.view().point;
        key(&mut s, KeyCode::Char('q'));
        assert!(s.echo().contains("changed or closed"), "{}", s.echo());
        assert_eq!(s.view().point, current);
        assert_eq!(s.buffer().unwrap().stamp(), current.stamp);
    }
}

// The list context overrides factory keys independently of ordinary editing.
// M-x cancellation retains the list; a subsequent cancellation returns source.
#[test]
fn local_keymap() {
    let f = fixture::Fixture::new();
    let mut s = configured(
        &f,
        "[keys.bookmark_list]\n'M-j'='next-line'\n'd'='bookmark-bmenu-unmark'\n'M-d'='bookmark-bmenu-delete'\n",
    );
    s.load(&f.write("source", "saved")).unwrap();
    for name in ["A", "B"] {
        argument(&mut s, BookmarkCommand::Set, name);
    }
    open(&mut s);
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('j'),
        KeyModifiers::ALT,
    )));
    selected(&mut s, "B");
    key(&mut s, KeyCode::Up);
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('d'),
        KeyModifiers::ALT,
    )));
    assert!(surface(&mut s).contains(" D \"A\""));
    key(&mut s, KeyCode::Up);
    key(&mut s, KeyCode::Char('d'));
    assert!(!surface(&mut s).contains(" D \"A\""));
    s.execute(Command::ExecuteExtended);
    cancel(&mut s);
    assert!(surface(&mut s).contains("*Bookmarks*"));
    cancel(&mut s);
    assert!(!surface(&mut s).contains("*Bookmarks*"));
    let error = rev::config::parse(
        std::path::Path::new("config.toml"),
        b"[keys.bookmark_list]\n'd'='newline'",
    )
    .unwrap_err();
    assert_eq!(error.key, vec!["keys", "bookmark_list", "d"]);
}

// Loading a large list leaves all missing targets unresolved. Painting and
// resize use retained metadata even after the owned store becomes invalid.
#[test]
fn unloaded_targets() {
    let f = fixture::Fixture::new();
    let store = f.0.join("bookmarks.toml");
    let mut data = String::from("version=1\n");
    for index in 0..1024 {
        data.push_str(&format!("[[bookmarks]]\nname='B{index:04}'\npath='{}/missing-{index}'\nposition=0\nbefore=''\nafter=''\nat_start=true\nat_end=true\n",f.0.display()));
    }
    f.write("bookmarks.toml", &data);
    let mut s = configured(&f, &format!("bookmarks.file='{}'\n", store.display()));
    open(&mut s);
    assert_eq!(s.bookmarks().len(), 1024);
    assert!(
        s.bookmarks()
            .iter()
            .all(|r| r.resolution() == rev::bookmark::Resolution::Unresolved)
    );
    std::fs::write(&store, "version=99\n").unwrap();
    for (width, height) in [(1, 1), (0, 0), (100, 20), (45, 8)] {
        s.handle(Event::Resize(width, height));
        s.frame().unwrap();
    }
    assert!(surface(&mut s).contains("1024 entries"));
    assert!(
        s.bookmarks()
            .iter()
            .all(|r| r.resolution() == rev::bookmark::Resolution::Unresolved)
    );
    assert_eq!(text(&s), "");
    assert_eq!(fs_read(&store), "version=99\n");
}

// Failed row visits retain the list, source and record. Canceling bounded
// context work also returns safely without publishing a candidate destination.
#[test]
fn canceled_visits() {
    let f = fixture::Fixture::new();
    let path = f.write("source", &format!("{}unique-end", "a\n".repeat(100000)));
    let mut s = session();
    s.load(&path).unwrap();
    s.execute(Command::Motion(Motion::BufferEnd));
    settle(&mut s);
    argument(&mut s, BookmarkCommand::Set, "A");
    kill(&mut s);
    std::fs::write(&path, format!("prefix\n{}unique-end", "a\n".repeat(100000))).unwrap();
    s.load(&f.write("origin", "origin")).unwrap();
    let origin = s.view().clone();
    open(&mut s);
    key(&mut s, KeyCode::Enter);
    assert!(s.remaining_input(std::time::Instant::now()).is_some());
    assert!(surface(&mut s).contains("*Bookmarks*"));
    cancel(&mut s);
    settle(&mut s);
    assert_eq!(s.view(), &origin);
    assert_eq!(text(&s), "origin");
    std::fs::remove_file(&path).unwrap();
    open(&mut s);
    key(&mut s, KeyCode::Enter);
    assert!(s.echo().contains("unavailable"));
    assert!(surface(&mut s).contains("*Bookmarks*"));
    assert_eq!(s.view(), &origin);
    assert!(s.bookmarks().get("A").is_some());
    cancel(&mut s);
    assert_eq!(s.view(), &origin);
}
fn settle(s: &mut Session) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        s.frame().unwrap();
        if s.remaining_input(std::time::Instant::now()).is_none() {
            break;
        }
        assert!(std::time::Instant::now() < end, "navigation did not settle");
        s.expire_input(std::time::Instant::now());
        std::thread::yield_now();
    }
}
