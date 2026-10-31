use super::*;
use rev::{buffer::Buffers, config::prepare, session::Action, view::View};
use std::fs;
use std::path::Path;

fn configured(f: &fixture::Fixture, mode: &str, extra: &str) -> Session {
    let path = f.0.join("data/bookmarks.toml");
    let config = f.write(
        "config.toml",
        &format!(
            "[bookmarks]\nfile = '{}'\nsave = '{mode}'\n{extra}",
            path.display()
        ),
    );
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let view = View::new(buffers.get(buffers.initial()).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}
fn answer(s: &mut Session, value: &str) -> Action {
    s.handle(Event::Paste(value.into()));
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )))
}
fn yesno(s: &mut Session, value: char) -> Action {
    s.handle(Event::Key(KeyEvent::new(
        KeyCode::Char(value),
        KeyModifiers::NONE,
    )))
}
fn disk(f: &fixture::Fixture) -> std::path::PathBuf {
    f.0.join("data/bookmarks.toml")
}
fn document(path: &Path, name: &str) -> String {
    format!(
        "version = 1\n[[bookmarks]]\nname = '{name}'\npath = '{}'\nposition = 0\nbefore = ''\nafter = 'saved'\nat_start = true\nat_end = true\n",
        path.display()
    )
}

// Store discovery belongs to first native use, never construction or painting.
// Restart restores checked durable evidence without saving or changing sources.
#[test]
fn lazy_restart() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    let mut s = configured(&f, "manual", "");
    f.write("data/bookmarks.toml", &document(&path, "restored"));
    assert!(s.bookmarks().is_empty());
    s.frame().unwrap();
    assert!(s.bookmarks().is_empty());
    argument(&mut s, BookmarkCommand::Jump, "restored");
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), path);
    assert_eq!(
        s.bookmarks().get("restored").unwrap().resolution(),
        rev::bookmark::Resolution::Resolved
    );
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "EOF");
    s.execute(Command::Bookmark(BookmarkCommand::Save));
    assert_eq!(s.echo(), "Bookmarks saved");
    assert!(!s.bookmarks().is_dirty());
    let mut restarted = configured(&f, "manual", "");
    argument(&mut restarted, BookmarkCommand::Jump, "EOF");
    assert_eq!(restarted.view().point.position.0, 5);
    assert_eq!(text(&restarted), "saved");
    assert!(!restarted.buffer().unwrap().is_modified());
    assert_eq!(fs::read_to_string(path).unwrap(), "saved");
    let stored = fs::read_to_string(disk(&f)).unwrap();
    assert!(stored.find("name = \"EOF\"").unwrap() < stored.find("name = \"restored\"").unwrap());
}

// Each automatic policy publishes at its documented boundary; explicit save
// works in all modes, and no mode writes source buffers or changes undo state.
#[test]
fn save_modes() {
    for mode in ["on-exit", "on-change", "manual"] {
        let f = fixture::Fixture::new();
        let path = f.write("source", "saved");
        let mut s = configured(&f, mode, "");
        s.load(&path).unwrap();
        argument(&mut s, BookmarkCommand::Set, "A");
        assert_eq!(disk(&f).exists(), mode == "on-change", "{mode}");
        s.insert('X');
        let state = s.buffer().unwrap().state();
        let bytes = fs::read(disk(&f)).ok();
        assert!(s.bookmarks().is_dirty());
        assert_eq!(fs::read(disk(&f)).ok(), bytes);
        assert_eq!(s.execute(Command::Quit), Action::Redraw);
        assert_eq!(yesno(&mut s, 'n'), Action::Exit, "{mode}");
        assert_eq!(s.buffer().unwrap().state(), state);
        assert_eq!(text(&s), "Xsaved");
        assert!(s.buffer().unwrap().is_modified());
        assert_eq!(fs::read_to_string(&path).unwrap(), "saved");
        assert_eq!(disk(&f).exists(), mode != "manual", "{mode}");
        if mode == "manual" {
            assert!(s.bookmarks().is_dirty());
        }
        s.execute(Command::Bookmark(BookmarkCommand::Save));
        assert_eq!(s.echo(), "Bookmarks saved");
        assert!(!s.bookmarks().is_dirty());
        assert_eq!(s.buffer().unwrap().state(), state);
    }
}

// On-change checkpoints explicit mutations, while source edits only dirty the
// in-memory evidence until a selected checkpoint. Intended files stay missing.
#[test]
fn intended_and_dirty() {
    let f = fixture::Fixture::new();
    let missing = f.0.join("intended");
    let mut s = configured(&f, "on-change", "context_chars = 16\n");
    s.load(&missing).unwrap();
    s.handle(Event::Paste("a".repeat(100)));
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "new");
    let old = fs::read(disk(&f)).unwrap();
    assert_eq!(
        s.bookmarks()
            .get("new")
            .unwrap()
            .context()
            .before
            .chars()
            .count(),
        16
    );
    s.insert('Z');
    assert!(s.bookmarks().is_dirty());
    assert_eq!(fs::read(disk(&f)).unwrap(), old);
    s.execute(Command::Bookmark(BookmarkCommand::Save));
    assert_ne!(fs::read(disk(&f)).unwrap(), old);
    assert!(!missing.exists());
    let mut restarted = configured(&f, "manual", "");
    let origin = restarted.view().clone();
    argument(&mut restarted, BookmarkCommand::Jump, "new");
    assert!(restarted.echo().contains("unavailable"));
    assert_eq!(restarted.view(), &origin);
    assert!(!missing.exists());
}

// Bad disk input blocks only persistence. Native memory operations remain
// usable, but explicit save alone never authorizes replacing unknown bytes.
#[test]
fn malformed_preserved() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    f.write("data/bookmarks.toml", "version = 99\n");
    let mut s = configured(&f, "on-change", "");
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "memory");
    assert!(s.echo().contains("blocked"));
    assert!(s.bookmarks().get("memory").is_some());
    s.execute(Command::Bookmark(BookmarkCommand::Save));
    assert!(s.echo().contains("blocked"));
    assert_eq!(fs::read_to_string(disk(&f)).unwrap(), "version = 99\n");
    f.write("data/bookmarks.toml", &document(&path, "disk"));
    s.execute(Command::Bookmark(BookmarkCommand::Reload));
    assert_eq!(yesno(&mut s, 'y'), Action::Redraw);
    assert!(s.bookmarks().get("memory").is_none());
    assert!(s.bookmarks().get("disk").is_some());
    assert!(!s.bookmarks().is_dirty());
    argument(&mut s, BookmarkCommand::Set, "new");
    assert!(!s.bookmarks().is_dirty());
}

// Reload validates first, confirms dirty replacement, rechecks the candidate
// after confirmation, and uses fresh identities so old source undo stays safe.
#[test]
fn reload_decisions() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    let mut s = configured(&f, "manual", "");
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "memory");
    let original = s.bookmarks().get("memory").unwrap().clone();
    let candidate = document(&path, "disk");
    f.write("data/bookmarks.toml", &candidate);
    for accept in [false, true] {
        s.execute(Command::Bookmark(BookmarkCommand::Reload));
        if accept {
            cancel(&mut s);
        } else {
            yesno(&mut s, 'n');
        }
        assert_eq!(s.bookmarks().get("memory"), Some(&original));
        assert_eq!(fs::read_to_string(disk(&f)).unwrap(), candidate);
    }
    s.execute(Command::Bookmark(BookmarkCommand::Reload));
    f.write("data/bookmarks.toml", "version = 2\n");
    yesno(&mut s, 'y');
    assert!(s.echo().contains("changed externally"));
    assert_eq!(s.bookmarks().get("memory"), Some(&original));
    s.execute(Command::Bookmark(BookmarkCommand::Reload));
    assert!(s.echo().contains("version"));
    assert_eq!(s.bookmarks().get("memory"), Some(&original));
    f.write("data/bookmarks.toml", &document(&path, "memory"));
    s.execute(Command::Bookmark(BookmarkCommand::Reload));
    yesno(&mut s, 'y');
    let record = s.bookmarks().get("memory").unwrap().clone();
    assert_ne!(record.id(), original.id());
    assert_eq!(record.resolution(), rev::bookmark::Resolution::Unresolved);
}

// Failed quit storage retains the session after source decisions. Cancel
// preserves all unsaved text; retry does not replay earlier source decisions.
#[test]
fn quit_recovery() {
    for choice in ["retry", "discard", "cancel", "C-g"] {
        let f = fixture::Fixture::new();
        let path = f.write("source", "saved");
        let mut s = configured(&f, "on-exit", "");
        s.load(&path).unwrap();
        argument(&mut s, BookmarkCommand::Set, "A");
        s.insert('X');
        f.write("data", "not a directory");
        let state = s.buffer().unwrap().state();
        assert_eq!(s.execute(Command::Quit), Action::Redraw);
        assert_eq!(yesno(&mut s, 'n'), Action::Redraw);
        assert!(
            s.frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().contains("retry/discard/cancel")),
            "{}",
            s.echo()
        );
        let action = match choice {
            "retry" => {
                fs::remove_file(f.0.join("data")).unwrap();
                answer(&mut s, "retry")
            }
            "C-g" => s.handle(Event::Key(KeyEvent::new(
                KeyCode::Char('g'),
                KeyModifiers::CONTROL,
            ))),
            _ => answer(&mut s, choice),
        };
        assert_eq!(
            action,
            if matches!(choice, "retry" | "discard") {
                Action::Exit
            } else {
                Action::Redraw
            },
            "{choice}"
        );
        assert_eq!(s.buffer().unwrap().state(), state);
        assert_eq!(text(&s), "Xsaved");
        assert_eq!(fs::read_to_string(&path).unwrap(), "saved");
        assert_eq!(disk(&f).exists(), choice == "retry");
        assert_eq!(s.bookmarks().is_dirty(), choice != "retry");
    }
}

// The configured cap covers fresh memory and whole disk candidates. Stored
// files are not truncated when a session lowers its admission limit.
#[test]
fn entry_cap() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    let mut s = configured(&f, "manual", "max_entries = 1\n");
    s.load(&path).unwrap();
    argument(&mut s, BookmarkCommand::Set, "A");
    let original = s.bookmarks().get("A").unwrap().clone();
    argument(&mut s, BookmarkCommand::Set, "B");
    assert!(s.echo().contains("full (1 entries)"));
    assert_eq!(s.bookmarks().get("A"), Some(&original));
    cancel(&mut s);
    s.execute(Command::Bookmark(BookmarkCommand::Save));
    let mut candidate = fs::read_to_string(disk(&f)).unwrap();
    let entry = candidate
        .split_once("\n[[bookmarks]]")
        .unwrap()
        .1
        .replace("name = \"A\"", "name = \"B\"");
    candidate.push_str(&format!("\n[[bookmarks]]{entry}"));
    fs::write(disk(&f), &candidate).unwrap();
    s.execute(Command::Bookmark(BookmarkCommand::Reload));
    assert!(s.echo().contains("exceeds 1 entries"));
    assert_eq!(s.bookmarks().get("A"), Some(&original));
    assert_eq!(fs::read_to_string(disk(&f)).unwrap(), candidate);
}

// A configured zero search budget checks the old position but cannot silently
// relocate; no candidate view or file association publishes on failure.
#[test]
fn relocation_disabled() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    f.write("data/bookmarks.toml", &document(&path, "A"));
    fs::write(&path, "prefixsaved").unwrap();
    let mut s = configured(&f, "manual", "search_chars = 0\n");
    let origin = s.view().clone();
    argument(&mut s, BookmarkCommand::Jump, "A");
    assert!(s.echo().contains("disabled"));
    assert_eq!(s.view(), &origin);
    assert_eq!(fs::read_to_string(path).unwrap(), "prefixsaved");
}

// Production startup resolves one user-wide XDG file independently of cwd.
// Child-only environment overrides keep the test harness and real user clean.
#[test]
fn xdg_restart() {
    let f = fixture::Fixture::new();
    let path = f.write("source", "saved");
    let config = "bookmarks.save='manual'\norg.clock.persist=false\n";
    f.write("config/rev/config.toml", config);
    for role in ["write", "read"] {
        let cwd = f.0.join(role);
        fs::create_dir(&cwd).unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "persistence::startup_child", "--nocapture"])
            .env("REV_BOOKMARK_STARTUP_SOURCE", &path)
            .env("REV_BOOKMARK_STARTUP_ROLE", role)
            .env("XDG_CONFIG_HOME", f.0.join("config"))
            .env("XDG_DATA_HOME", f.0.join("user-data"))
            .env("XDG_STATE_HOME", f.0.join("state"))
            .env("XDG_CACHE_HOME", f.0.join("cache"))
            .env("HOME", f.0.join("home"))
            .current_dir(&cwd)
            .status()
            .unwrap();
        assert!(result.success(), "{role}");
        assert_eq!(
            fs::read_to_string(f.0.join("config/rev/config.toml")).unwrap(),
            config
        );
    }
    assert!(f.0.join("user-data/rev/bookmarks.toml").is_file());
}

// Inert during the ordinary run; invoked only in isolated restart subprocesses.
#[test]
fn startup_child() {
    let Some(path) = std::env::var_os("REV_BOOKMARK_STARTUP_SOURCE") else {
        return;
    };
    let role = std::env::var("REV_BOOKMARK_STARTUP_ROLE").unwrap();
    let mut s = Session::startup(
        None,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap();
    assert!(s.bookmarks().is_empty());
    s.frame().unwrap();
    assert!(s.bookmarks().is_empty());
    if role == "write" {
        s.load(Path::new(&path)).unwrap();
        s.execute(Command::Motion(Motion::BufferEnd));
        argument(&mut s, BookmarkCommand::Set, "shared");
        s.execute(Command::Bookmark(BookmarkCommand::Save));
        assert_eq!(s.echo(), "Bookmarks saved");
    } else {
        argument(&mut s, BookmarkCommand::Jump, "shared");
        assert_eq!(s.buffer().unwrap().file().unwrap().path(), Path::new(&path));
        assert_eq!(s.view().point.position.0, 5);
    }
}

// Native file visits and restart consume the byte-array representation without
// a display-label round trip, including real source IO on a non-UTF-8 pathname.
#[test]
fn native_path_restart() {
    use std::os::unix::ffi::OsStringExt;
    let f = fixture::Fixture::new();
    let path =
        f.0.join(std::ffi::OsString::from_vec(b"source-\xff".to_vec()));
    fs::write(&path, "界saved").unwrap();
    let mut s = configured(&f, "manual", "");
    s.load(&path).unwrap();
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "native");
    s.execute(Command::Bookmark(BookmarkCommand::Save));
    assert_eq!(s.echo(), "Bookmarks saved");
    assert!(
        fs::read_to_string(disk(&f))
            .unwrap()
            .contains("path_bytes = [")
    );
    let mut restarted = configured(&f, "manual", "");
    argument(&mut restarted, BookmarkCommand::Jump, "native");
    assert_eq!(restarted.buffer().unwrap().file().unwrap().path(), path);
    assert_eq!(restarted.view().point.position.0, 6);
    assert_eq!(text(&restarted), "界saved");
}

// On-change follows successful source publication to its canonical save-as
// path and persists refreshed context; undo still belongs to the source owner.
#[test]
fn save_as_checkpoint() {
    let f = fixture::Fixture::new();
    let original = f.write("original", "saved");
    let destination = f.0.join("destination");
    let mut s = configured(&f, "on-change", "");
    s.load(&original).unwrap();
    s.execute(Command::Motion(Motion::BufferEnd));
    argument(&mut s, BookmarkCommand::Set, "A");
    s.insert('X');
    s.execute(Command::WriteFile);
    answer(&mut s, destination.to_str().unwrap());
    assert!(!s.bookmarks().is_dirty(), "{}", s.echo());
    assert_eq!(s.bookmarks().get("A").unwrap().path(), destination);
    let mut restarted = configured(&f, "manual", "");
    argument(&mut restarted, BookmarkCommand::Jump, "A");
    assert_eq!(
        restarted.buffer().unwrap().file().unwrap().path(),
        destination
    );
    assert_eq!(text(&restarted), "savedX");
    assert_eq!(fs::read_to_string(original).unwrap(), "saved");
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), "saved");
    assert!(s.bookmarks().is_dirty());
    s.execute(Command::SaveBuffer);
    assert!(!s.bookmarks().is_dirty());
    assert_eq!(fs::read_to_string(destination).unwrap(), "saved");
}

// A canonical path change is a first-use demand for durable associations even
// if no bookmark command has yet loaded them. Closed records follow both paths.
#[test]
fn lazy_file_lifetime() {
    use rev::command::TreeCommand;
    for operation in ["save-as", "rename"] {
        let f = fixture::Fixture::new();
        let original = f.write("directory/original", "saved");
        let destination = if operation == "save-as" {
            f.0.join("directory/destination")
        } else {
            // Focus initially selects the visible root directory itself.
            f.0.join("destination/original")
        };
        f.write("data/bookmarks.toml", &document(&original, "closed"));
        let mut s = configured(&f, "on-change", "");
        s.load(&original).unwrap();
        assert!(s.bookmarks().is_empty());
        if operation == "save-as" {
            s.execute(Command::WriteFile);
            answer(&mut s, destination.to_str().unwrap());
        } else {
            s.execute(Command::Tree(TreeCommand::Focus));
            s.execute(Command::Tree(TreeCommand::Rename));
            answer(&mut s, "destination");
        }
        assert!(destination.is_file(), "{operation}: {}", s.echo());
        assert_eq!(
            s.bookmarks().get("closed").unwrap().path(),
            destination,
            "{operation}"
        );
        assert!(!s.bookmarks().is_dirty(), "{operation}: {}", s.echo());
        assert_eq!(original.exists(), operation == "save-as");
        let mut restarted = configured(&f, "manual", "");
        argument(&mut restarted, BookmarkCommand::Jump, "closed");
        assert_eq!(
            restarted.buffer().unwrap().file().unwrap().path(),
            destination,
            "{operation}: {}",
            restarted.echo()
        );
        assert_eq!(text(&restarted), "saved");
    }
}
