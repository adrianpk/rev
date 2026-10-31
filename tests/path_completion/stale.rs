use std::fs;

use crossterm::event::{Event, KeyCode};
use rev::command::{Command, Split};

use super::acceptance::{key, query, text};
use super::edges::configured;
use super::fixture::Fixture;

// UI03-UI04/CFG16: replacing cached file bytes with invalid ingress cannot
// consume the chosen path, dirty shared text, undo state or editable query.
#[test]
fn changed_contents() {
    for (name, bytes) in [("UTF-8", b"\xff".as_slice()), ("NUL", b"a\0b".as_slice())] {
        let fixture = Fixture::new();
        let mut session = configured(&fixture, "minibuffer.candidate_limit = 1");
        let target = fixture.write("root/alpha.txt", "observed");
        session.execute(Command::Split(Split::Right));
        session.insert('!');
        session.frame().unwrap();
        let before = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let state = session.buffer().unwrap().state();
        session.execute(Command::FindFile);
        session.handle(Event::Paste("al".into()));
        session.handle(key(KeyCode::Up));
        fs::write(&target, bytes).unwrap();
        fixture.write("root/aaa.txt", "unrelated");
        session.handle(key(KeyCode::Enter));
        assert!(!session.echo().is_empty(), "{name}");
        assert!(query(&session.frame().unwrap()).starts_with("Find file: al  "));
        assert_eq!(*session.view(), before);
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().state(), state);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        assert!(session.buffer().unwrap().is_modified());
        assert_eq!(text(&session), "!retained\ntext");
        assert_eq!(fs::read(&target).unwrap(), bytes);
        fs::write(&target, b"recovered selection").unwrap();
        session.handle(key(KeyCode::Enter));
        assert_eq!(text(&session), "recovered selection");
        assert_eq!(session.buffer().unwrap().file().unwrap().path(), target);
        assert_eq!(
            fs::read(fixture.0.join("root/document.txt")).unwrap(),
            b"retained\ntext"
        );
        assert_eq!(
            fs::read(fixture.0.join("root/aaa.txt")).unwrap(),
            b"unrelated"
        );
    }
}

// UI03-UI04: Return rechecks the selected path's current kind. Cached labels
// cannot force an obsolete target kind or visit a replacement list row.
#[test]
fn changed_kind() {
    for was_directory in [false, true] {
        let fixture = Fixture::new();
        let mut session = configured(&fixture, "minibuffer.candidate_limit = 1");
        let target = fixture.0.join("root/choice");
        if was_directory {
            fs::create_dir(&target).unwrap();
        } else {
            fs::write(&target, "observed").unwrap();
        }
        session.execute(Command::FindFile);
        session.handle(Event::Paste("cho".into()));
        session.handle(key(KeyCode::Up));
        replace_kind(&target, was_directory);
        fixture.write("root/chosen-earlier", "unrelated");
        session.handle(key(KeyCode::Enter));
        if was_directory {
            assert_eq!(text(&session), "replacement");
            assert_eq!(session.buffer().unwrap().file().unwrap().path(), target);
        } else {
            assert!(!query(&session.frame().unwrap()).starts_with("Find file:"));
            assert_eq!(session.tree().root(), Some(target.as_path()));
            assert_eq!(text(&session), "retained\ntext");
            session.handle(key(KeyCode::Down));
            session.handle(key(KeyCode::Enter));
            assert_eq!(text(&session), "child contents");
        }
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(
            fs::read(fixture.0.join("root/chosen-earlier")).unwrap(),
            b"unrelated"
        );
    }
}

fn replace_kind(target: &std::path::Path, directory: bool) {
    if directory {
        fs::remove_dir(target).unwrap();
        fs::write(target, b"replacement").unwrap();
    } else {
        fs::remove_file(target).unwrap();
        fs::create_dir(target).unwrap();
        fs::write(target.join("nested"), b"child contents").unwrap();
    }
}

// Tab binds the visible default or navigated identity before refreshing. Missing
// names cannot become a new row on repeated Tab, nor consume dirty source state.
#[test]
fn vanished_completion() {
    for explicit in [false, true] {
        let fixture = Fixture::new();
        let mut session = super::acceptance::editor(&fixture);
        fixture.write("root/alpine.txt", "alternative");
        session.insert('!');
        session.frame().unwrap();
        let view = session.view().clone();
        let stamp = session.buffer().unwrap().stamp();
        let history = session.buffer().unwrap().history().len();
        session.execute(Command::FindFile);
        session.handle(Event::Paste("al".into()));
        if explicit {
            session.handle(key(KeyCode::Down));
        }
        let name = if explicit { "alpine.txt" } else { "alpha.txt" };
        fs::remove_file(fixture.0.join("root").join(name)).unwrap();
        fixture.write("root/albatross.txt", "unrelated row");
        for _ in 0..2 {
            session.handle(key(KeyCode::Tab));
            assert!(
                session
                    .echo()
                    .contains("Selected path is no longer present")
            );
            let frame = session.frame().unwrap();
            assert!(query(&frame).starts_with("Find file: al  "));
            let selected = frame
                .rows()
                .iter()
                .find(|row| {
                    row.band() == rev::display::Band::Completion && row.selection().is_some()
                })
                .unwrap();
            assert_eq!(selected.text().trim(), name);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(session.buffer().unwrap().history().len(), history);
        }
        fixture.write(&format!("root/{name}"), "restored path");
        session.handle(key(KeyCode::Tab));
        assert_eq!(
            query(&session.frame().unwrap()),
            format!("Find file: {name}")
        );
        session.execute(Command::KeyboardQuit);
        session.frame().unwrap();
        assert_eq!(*session.view(), view);
        assert_eq!(text(&session), "!retained\ntext");
        assert_eq!(
            fs::read(fixture.0.join("root/albatross.txt")).unwrap(),
            b"unrelated row"
        );
    }
}

// A denied component refresh keeps its editable query, selected identity and
// dirty buffer. Restoring owned access permits retry without replacing a row.
#[test]
fn denied_completion() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = Fixture::new();
    let mut session = super::acceptance::editor(&fixture);
    session.insert('!');
    session.frame().unwrap();
    let view = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::FindFile);
    session.handle(Event::Paste("al".into()));
    let root = fixture.0.join("root");
    let permissions = fs::metadata(&root).unwrap().permissions();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o000)).unwrap();
    session.handle(key(KeyCode::Tab));
    let notice = session.echo().to_owned();
    let frame = session.frame().unwrap();
    fs::set_permissions(&root, permissions).unwrap();
    assert!(notice.contains("Permission denied"));
    assert!(query(&frame).starts_with("Find file: al  "));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.handle(key(KeyCode::Tab));
    assert_eq!(query(&session.frame().unwrap()), "Find file: alpha.txt");
    session.execute(Command::KeyboardQuit);
    session.frame().unwrap();
    assert_eq!(*session.view(), view);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(text(&session), "!retained\ntext");
}
