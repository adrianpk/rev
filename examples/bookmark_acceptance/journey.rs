use std::path::Path;
use std::time::Instant;

use crossterm::event::{Event, KeyCode};
use rev::bookmark::{Record, Resolution};
use rev::command::{
    BookmarkCommand as B, Command as C, GutterCommand, History, Motion, OrgCommand, Split,
};
use rev::session::{Action, Session};

use super::support::*;

pub(super) fn run(root: &Path) -> Result<()> {
    let started = Instant::now();
    let path = root.join("workspace/alpha.txt");
    let original = format!(
        "{}\nTARGET here {}\nlast\n",
        "first line ".repeat(12),
        "after words ".repeat(12)
    );
    std::fs::write(&path, &original)?;
    std::os::unix::fs::symlink(&path, root.join("workspace/alias.txt"))?;
    let target = original.find("TARGET").unwrap();
    let saved = first_session(root, &path, &original, target)?;
    let store = root.join("native/data/rev/bookmarks.toml");
    persisted(&store, target)?;
    reopened(root, target, &saved)?;
    relocated(&path, target, &saved)?;
    folded(root)?;
    recovery(root, &path, &store, &original)?;
    println!(
        "PASS native bookmark journey: {}ms; set/refuse, edit/undo/save, alias, splits, list/delete, restart/context, fold reveal, unavailable/malformed protection and idle",
        started.elapsed().as_millis()
    );
    Ok(())
}

fn first_session(root: &Path, path: &Path, original: &str, target: usize) -> Result<String> {
    let mut s = Session::startup(Some(path), SIZE)?;
    let record = set_record(&mut s, target)?;
    let saved = edit_source(&mut s, path, original, target)?;
    alias_markers(&mut s, root, &record, target)?;
    list_and_exit(&mut s, &saved)?;
    drop(s);
    Ok(saved)
}

fn set_record(s: &mut Session, target: usize) -> Result<Record> {
    goto(s, 2);
    assert_eq!(s.view().point.position.0, target);
    let saved_state = s.buffer()?.state();
    bookmark(s, B::Set, "place");
    bookmark(s, B::Set, "tied");
    let record = s.bookmarks().get("place").unwrap().clone();
    bookmark(s, B::SetNoOverwrite, "place");
    assert!(s.echo().contains("already exists"));
    s.execute(C::KeyboardQuit);
    assert_eq!(s.bookmarks().get("place"), Some(&record));
    assert_eq!(s.buffer()?.state(), saved_state);
    assert!(s.buffer()?.history().is_empty());
    assert_eq!(markers(s)?, 1);

    Ok(record)
}

fn edit_source(s: &mut Session, path: &Path, original: &str, target: usize) -> Result<String> {
    s.execute(C::Motion(Motion::BufferStart));
    s.handle(Event::Paste("edit\n".into()));
    assert_eq!(
        s.bookmarks().get("place").unwrap().point().position.0,
        target + 5
    );
    assert!(s.buffer()?.is_modified());
    s.execute(C::History(History::Undo));
    assert_eq!(source(s), original.as_bytes());
    assert_eq!(
        s.bookmarks().get("place").unwrap().point().position.0,
        target
    );
    s.execute(C::History(History::Redo));
    s.execute(C::SaveBuffer);
    let saved = format!("edit\n{original}");
    assert_eq!(std::fs::read(path)?, saved.as_bytes());
    Ok(saved)
}

fn alias_markers(s: &mut Session, root: &Path, record: &Record, target: usize) -> Result<()> {
    let id = s.view().point.stamp.buffer;
    s.load(&root.join("workspace/alias.txt"))?;
    assert_eq!(s.view().point.stamp.buffer, id);
    s.execute(C::Split(Split::Right));
    let peer = s.windows().next_view().unwrap();
    s.execute(C::Bookmark(B::Markers));
    assert!(!s.view().bookmark_markers && s.windows().get(peer)?.bookmark_markers);
    s.execute(C::Gutter(GutterCommand::Relative));
    s.execute(C::Gutter(GutterCommand::ToggleSide));
    s.execute(C::Gutter(GutterCommand::Toggle));
    s.execute(C::Bookmark(B::Markers));
    assert_eq!(s.bookmarks().get("place").unwrap().id(), record.id());
    s.execute(C::Bookmark(B::Next));
    assert!(s.echo().contains("place"));
    s.execute(C::Bookmark(B::Next));
    assert!(s.echo().contains("tied"));
    assert_eq!(s.view().point.position.0, target + 5);
    Ok(())
}

fn list_and_exit(s: &mut Session, saved: &str) -> Result<()> {
    s.execute(C::Bookmark(B::List));
    assert!(
        s.frame()?
            .rows()
            .iter()
            .any(|row| row.text().contains("*Bookmarks*"))
    );
    s.handle(Event::Paste("must not edit source".into()));
    assert_eq!(source(s), saved.as_bytes());
    key(s, KeyCode::Down);
    key(s, KeyCode::Char('d'));
    key(s, KeyCode::Char('x'));
    assert_eq!(s.bookmarks().len(), 1);
    assert!(s.bookmarks().get("place").is_some());
    key(s, KeyCode::Char('q'));
    settle(s)?;
    idle(s);
    assert_eq!(s.execute(C::Quit), Action::Exit);
    Ok(())
}

fn persisted(store: &Path, target: usize) -> Result<()> {
    let persisted = std::fs::read(store)?;
    let document: toml::Value = toml::from_str(std::str::from_utf8(&persisted)?)?;
    assert_eq!(document["version"].as_integer(), Some(1));
    assert_eq!(document["bookmarks"].as_array().unwrap().len(), 1);
    assert_eq!(
        document["bookmarks"][0]["position"].as_integer(),
        Some(i64::try_from(target + 5)?)
    );
    Ok(())
}

fn reopened(root: &Path, target: usize, saved: &str) -> Result<()> {
    let mut reopened = Session::startup(None, SIZE)?;
    reopened.load(&root.join("workspace"))?;
    let tree_root = reopened.tree().root().unwrap().to_path_buf();
    reopened.execute(C::KeyboardQuit); // Leave tree focus; retain its explicit root.
    bookmark(&mut reopened, B::Jump, "place");
    settle(&mut reopened)?;
    assert_eq!(
        reopened.view().point.position.0,
        target + 5,
        "reopened jump: {}",
        reopened.echo()
    );
    assert_eq!(reopened.tree().root(), Some(tree_root.as_path()));
    assert_eq!(source(&reopened), saved.as_bytes());
    assert!(!reopened.buffer()?.is_modified() && reopened.buffer()?.history().is_empty());
    assert_eq!(markers(&mut reopened)?, 1);
    assert_eq!(reopened.execute(C::Quit), Action::Exit);
    drop(reopened);

    Ok(())
}

fn relocated(path: &Path, target: usize, saved: &str) -> Result<()> {
    std::fs::write(path, format!("outside\n{saved}"))?;
    let mut relocated = Session::startup(None, SIZE)?;
    bookmark(&mut relocated, B::Jump, "place");
    settle(&mut relocated)?;
    assert_eq!(relocated.view().point.position.0, target + 13);
    assert!(relocated.echo().contains("checked context"));
    assert_eq!(
        relocated.bookmarks().get("place").unwrap().resolution(),
        Resolution::Resolved
    );
    assert_eq!(relocated.execute(C::Quit), Action::Exit);
    drop(relocated);

    Ok(())
}

fn folded(root: &Path) -> Result<()> {
    // A folded body is never represented by its heading's marker. Native jump
    // reveals that actual source row and retains independent source history.
    let mut folded = Session::startup(Some(&root.join("workspace/notes.org")), SIZE)?;
    goto(&mut folded, 2);
    bookmark(&mut folded, B::Set, "folded");
    folded.execute(C::Motion(Motion::BufferStart));
    folded.execute(C::Org(OrgCommand::HideSubtree));
    settle(&mut folded)?;
    assert_eq!(markers(&mut folded)?, 0);
    bookmark(&mut folded, B::Jump, "folded");
    settle(&mut folded)?;
    assert_eq!(markers(&mut folded)?, 1);
    assert!(!folded.buffer()?.is_modified() && folded.buffer()?.history().is_empty());
    drop(folded);

    Ok(())
}

fn recovery(root: &Path, path: &Path, store: &Path, original: &str) -> Result<()> {
    let retained_store = std::fs::read(store)?;
    unavailable(root, path, store, &retained_store)?;
    malformed(root, store)?;
    std::fs::write(store, retained_store)?;
    std::fs::rename(root.join("workspace/unavailable.txt"), path)?;
    std::fs::write(path, original)?;
    Ok(())
}

fn unavailable(root: &Path, path: &Path, store: &Path, retained_store: &[u8]) -> Result<()> {
    std::fs::rename(path, root.join("workspace/unavailable.txt"))?;
    let mut failed = Session::startup(None, SIZE)?;
    let origin = failed.view().point;
    bookmark(&mut failed, B::Jump, "place");
    settle(&mut failed)?;
    assert_eq!(failed.view().point, origin);
    assert_eq!(
        failed.bookmarks().get("place").unwrap().resolution(),
        Resolution::Unavailable
    );
    assert!(!path.exists());
    assert_eq!(std::fs::read(store)?, retained_store);
    drop(failed);
    Ok(())
}

fn malformed(root: &Path, store: &Path) -> Result<()> {
    std::fs::write(store, "version=999\n")?;
    let mut blocked = Session::startup(Some(&root.join("workspace/origin.txt")), SIZE)?;
    bookmark(&mut blocked, B::Set, "memory");
    blocked.execute(C::Bookmark(B::Save));
    assert!(blocked.echo().contains("blocked"));
    assert_eq!(std::fs::read(store)?, b"version=999\n");
    blocked.execute(C::Quit);
    blocked.execute(C::KeyboardQuit);
    assert!(blocked.bookmarks().get("memory").is_some());
    assert!(!blocked.buffer()?.is_modified());
    drop(blocked);
    Ok(())
}
