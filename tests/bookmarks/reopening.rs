use super::*;
use rev::bookmark::Resolution;
use rev::command::{BufferCommand, OrgCommand};

fn at(s: &mut Session, position: usize) {
    s.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..position {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
}
fn close(s: &mut Session) {
    let dirty = s.buffer().unwrap().is_modified();
    s.execute(Command::Buffer(BufferCommand::Kill));
    key(s, KeyCode::Enter);
    if dirty {
        key(s, KeyCode::Char('n'));
    }
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
            "bookmark work did not settle"
        );
        s.expire_input(std::time::Instant::now());
        std::thread::yield_now();
    }
}
fn sample() -> String {
    format!("{}unique-target{}", "a".repeat(300), "b".repeat(300))
}

// Checked context reopens unchanged files and relocates one unique match after
// a small external insertion, using scalar coordinates for Unicode text.
#[test]
fn contextual_reopen() {
    let f = fixture::Fixture::new();
    for prefix in ["", "界 inserted\n"] {
        let original = sample();
        let path = f.write("source", &original);
        let mut s = session();
        s.load(&path).unwrap();
        at(&mut s, 307);
        argument(&mut s, BookmarkCommand::Set, "here");
        let id = s.bookmarks().get("here").unwrap().id();
        close(&mut s);
        assert_eq!(
            s.bookmarks().get("here").unwrap().resolution(),
            Resolution::Unresolved
        );
        std::fs::write(&path, format!("{prefix}{original}")).unwrap();
        s.load(&f.write("origin", "origin")).unwrap();
        argument(&mut s, BookmarkCommand::Jump, "here");
        settle(&mut s);
        assert_eq!(s.buffer().unwrap().file().unwrap().path(), path);
        assert_eq!(s.view().point.position.0, 307 + prefix.chars().count());
        assert_eq!(s.bookmarks().get("here").unwrap().id(), id);
        assert_eq!(
            s.bookmarks().get("here").unwrap().resolution(),
            Resolution::Resolved
        );
        assert!(s.echo().contains("checked context"));
        assert_eq!(text(&s), format!("{prefix}{original}"));
        assert!(!s.buffer().unwrap().is_modified());
    }
}

// Repeated, removed and missing context preserve the origin and record. Neither
// an ambiguous match nor a missing destination supplies guessed source content.
#[test]
fn unresolved_targets() {
    for variant in ["ambiguous", "removed", "missing"] {
        let f = fixture::Fixture::new();
        let original = sample();
        let path = f.write("source", &original);
        let mut s = session();
        s.load(&path).unwrap();
        at(&mut s, 307);
        argument(&mut s, BookmarkCommand::Set, "here");
        close(&mut s);
        let retained = s.bookmarks().get("here").unwrap().clone();
        match variant {
            "ambiguous" => std::fs::write(&path, format!("prefix{original}{original}")).unwrap(),
            "removed" => std::fs::write(&path, "unrelated").unwrap(),
            _ => std::fs::remove_file(&path).unwrap(),
        }
        s.load(&f.write("origin", "origin")).unwrap();
        let before = s.view().clone();
        argument(&mut s, BookmarkCommand::Jump, "here");
        settle(&mut s);
        assert_eq!(s.view(), &before, "{variant}");
        assert!(
            s.echo().contains(match variant {
                "ambiguous" => "ambiguous",
                "missing" => "unavailable",
                _ => "stale",
            }),
            "{variant}: {}",
            s.echo()
        );
        assert_eq!(
            s.bookmarks().get("here").unwrap().context(),
            retained.context(),
            "{variant}"
        );
        assert_eq!(
            s.bookmarks().get("here").unwrap().id(),
            retained.id(),
            "{variant}"
        );
        if variant == "missing" {
            assert!(!path.exists());
        }
    }
}

// Empty files and source endpoints retain explicit BOF/EOF evidence; a changed
// empty target cannot be resolved simply because its former position was zero.
#[test]
fn boundary_context() {
    for (name, source, end) in [
        ("empty", "", false),
        ("BOF", "界ab", false),
        ("EOF", "界ab", true),
    ] {
        let f = fixture::Fixture::new();
        let path = f.write(name, source);
        let mut s = session();
        s.load(&path).unwrap();
        if end {
            s.execute(Command::Motion(Motion::BufferEnd));
        }
        argument(&mut s, BookmarkCommand::Set, "here");
        close(&mut s);
        argument(&mut s, BookmarkCommand::Jump, "here");
        settle(&mut s);
        assert_eq!(
            s.view().point.position.0,
            if end { source.chars().count() } else { 0 },
            "{name}"
        );
        assert_eq!(text(&s), source);
        if source.is_empty() {
            close(&mut s);
            std::fs::write(&path, "new").unwrap();
            let origin = s.view().clone();
            argument(&mut s, BookmarkCommand::Jump, "here");
            assert_eq!(s.view(), &origin);
            assert!(s.echo().contains("evidence"));
        }
    }
}

// Normal visits can reopen a target before bookmark use. Resolution prefers its
// unsaved live text and never rereads disk over that canonical buffer.
#[test]
fn live_reopen_preference() {
    let f = fixture::Fixture::new();
    let original = sample();
    let path = f.write("source", &original);
    let mut s = session();
    s.load(&path).unwrap();
    at(&mut s, 307);
    argument(&mut s, BookmarkCommand::Set, "here");
    close(&mut s);
    s.load(&path).unwrap();
    s.replace(0..0, "dirty\n").unwrap();
    let live = s.buffer().unwrap().id();
    s.load(&f.write("origin", "origin")).unwrap();
    argument(&mut s, BookmarkCommand::Jump, "here");
    settle(&mut s);
    assert_eq!(s.buffer().unwrap().id(), live);
    assert_eq!(s.view().point.position.0, 313);
    assert!(s.buffer().unwrap().is_modified());
    assert_eq!(text(&s), format!("dirty\n{original}"));
    assert_eq!(std::fs::read_to_string(path).unwrap(), original);
}

// Closing after discard removes the live anchor. Unsaved content is not invented
// on disk; its context must be independently available in the reopened source.
#[test]
fn discarded_content() {
    for existing in [true, false] {
        let f = fixture::Fixture::new();
        let path = if existing {
            f.write("source", "saved")
        } else {
            f.0.join("source")
        };
        let mut s = session();
        s.load(&path).unwrap();
        s.replace(0..0, "unsaved unique ").unwrap();
        at(&mut s, 8);
        argument(&mut s, BookmarkCommand::Set, "here");
        close(&mut s);
        let before = s.view().clone();
        argument(&mut s, BookmarkCommand::Jump, "here");
        settle(&mut s);
        assert_eq!(s.view(), &before);
        assert_ne!(
            s.bookmarks().get("here").unwrap().resolution(),
            Resolution::Resolved
        );
        if existing {
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "saved");
        } else {
            assert!(!path.exists());
        }
    }
}

// Successful save-as follows the canonical destination, refreshes resolved
// context and keeps source undo attached to the original native buffer identity.
#[test]
fn save_as_lifetime() {
    let f = fixture::Fixture::new();
    let original = f.write("original", "source");
    let destination = f.0.join("destination");
    let mut s = session();
    s.load(&original).unwrap();
    at(&mut s, 3);
    argument(&mut s, BookmarkCommand::Set, "here");
    s.replace(0..0, "edited ").unwrap();
    let before = s.view().point;
    s.execute(Command::WriteFile);
    s.handle(Event::Paste(destination.to_str().unwrap().into()));
    key(&mut s, KeyCode::Enter);
    assert_eq!(s.bookmarks().get("here").unwrap().path(), destination);
    assert_eq!(
        s.bookmarks().get("here").unwrap().point().stamp.buffer,
        before.stamp.buffer
    );
    s.execute(Command::History(History::Undo));
    assert_eq!(s.bookmarks().get("here").unwrap().point().position.0, 3);
    s.execute(Command::History(History::Redo));
    s.execute(Command::SaveBuffer);
    close(&mut s);
    argument(&mut s, BookmarkCommand::Jump, "here");
    settle(&mut s);
    assert_eq!(s.buffer().unwrap().file().unwrap().path(), destination);
    assert_eq!(s.view().point.position.0, 10);
    assert_eq!(std::fs::read_to_string(original).unwrap(), "source");
    assert_eq!(text(&s), "edited source");
}

// Reopening a long source yields before publication. Releases and resize retain
// work, cancellation/input changes discard it, and disk races reject admission.
#[test]
fn pending_resolution() {
    for event in ["cancel", "edit", "disk"] {
        let f = fixture::Fixture::new();
        let original = format!("{}unique end", "a\n".repeat(100000));
        let path = f.write("source", &original);
        let mut s = session();
        s.load(&path).unwrap();
        s.execute(Command::Motion(Motion::BufferEnd));
        settle(&mut s);
        argument(&mut s, BookmarkCommand::Set, "here");
        close(&mut s);
        std::fs::write(&path, format!("prefix\n{original}")).unwrap();
        s.load(&f.write("origin", "origin")).unwrap();
        let before = s.view().clone();
        argument(&mut s, BookmarkCommand::Jump, "here");
        assert_eq!(s.view(), &before);
        assert!(s.remaining_input(std::time::Instant::now()).is_some());
        let mut release = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        release.kind = crossterm::event::KeyEventKind::Release;
        s.handle(Event::Key(release));
        assert!(s.remaining_input(std::time::Instant::now()).is_some());
        s.handle(Event::Resize(60, 18));
        match event {
            "cancel" => cancel(&mut s),
            "edit" => {
                s.replace(0..0, "changed ").unwrap();
            }
            _ => std::fs::write(&path, "raced replacement").unwrap(),
        }
        settle(&mut s);
        assert_eq!(
            s.buffer().unwrap().file().unwrap().path(),
            f.0.join("origin"),
            "{event}"
        );
        assert_ne!(
            s.bookmarks().get("here").unwrap().resolution(),
            Resolution::Resolved,
            "{event}"
        );
        assert!(
            s.remaining_input(std::time::Instant::now()).is_none(),
            "{event}"
        );
        if event == "disk" {
            assert!(
                s.echo().contains("changed during resolution"),
                "{}",
                s.echo()
            );
        }
    }
}

// Native reveal opens hidden ancestors and wrapped fragments in the selected
// view while another split retains its own fold state over the same source.
#[test]
fn folded_wrapped_reveal() {
    let f = fixture::Fixture::new();
    let source = "* Parent\n** Child\nbody with a needle and a long wrapped continuation for navigation\n* Other\n";
    let mut s = Session::empty(Size {
        width: 45,
        height: 18,
    })
    .unwrap();
    s.load(&f.write("source.org", source)).unwrap();
    let target = source.find("needle").unwrap();
    at(&mut s, target);
    argument(&mut s, BookmarkCommand::Set, "here");
    at(&mut s, 0);
    s.execute(Command::Org(OrgCommand::HideSubtree));
    settle(&mut s);
    s.execute(Command::Split(rev::command::Split::Below));
    let selected = s.windows().selected_id();
    let other = s
        .windows()
        .iter()
        .find(|(id, _)| *id != selected)
        .map(|(id, view)| (id, view.clone()))
        .unwrap();
    argument(&mut s, BookmarkCommand::Jump, "here");
    settle(&mut s);
    assert_eq!(s.view().point.position.0, target);
    assert_eq!(s.windows().get(other.0).unwrap(), &other.1);
    assert!(s.frame().unwrap().cursor().is_some());
    assert_eq!(text(&s), source);
}
