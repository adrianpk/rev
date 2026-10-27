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
// cannot force an obsolete directory descent or visit a replacement list row.
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
            assert_eq!(query(&session.frame().unwrap()), "Find file: choice/");
            assert_eq!(text(&session), "retained\ntext");
            session.handle(key(KeyCode::Up));
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
