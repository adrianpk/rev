use std::os::unix::fs::PermissionsExt;

use rev::command::{Command, History, Motion, Split, TreeCommand};
use rev::config::prepare;

use super::*;

fn configured(fixture: &Fixture, source: &Path) -> Session {
    let config = fixture.file("config.toml", b"tree.root = 'file-parent'\nlanguage.rust.tab_width = 2\nlanguage.go.tab_width = 8\nhistory.group_limit = 1\nhistory.payload_target_bytes = 1\nminibuffer.input_limit = 32\nminibuffer.candidate_limit = 1");
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(source).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 240,
            height: 12,
        },
    )
    .unwrap();
    session.execute(Command::Split(Split::Right));
    session.insert('!');
    session.execute(Command::Motion(Motion::BufferEnd));
    session.insert('?');
    session.execute(Command::History(History::Undo));
    fs::write(config, b"language.go.tab_width = 3").unwrap();
    session
}

fn choose(session: &mut Session, target: &Path) {
    for _ in 0..session.tree().entries().len() {
        session.execute(Command::Tree(TreeCommand::Previous));
    }
    for _ in 0..session.tree().entries().len() {
        if session.tree().selected().unwrap().path() == target {
            break;
        }
        session.execute(Command::Tree(TreeCommand::Next));
    }
    assert_eq!(session.tree().selected().unwrap().path(), target);
}

fn rename(session: &mut Session, name: &str) {
    session.execute(Command::Tree(TreeCommand::Rename));
    session.handle(Event::Paste(name.into()));
    session.handle(press(KeyCode::Enter));
}

// CFG09-CFG11/CFG16-CFG17/UI01: failed file/directory renames retain the
// configured policy, shared views, truncated undo and redo future. Success
// changes only paths/policy; an explicit save preserves BOM/CRLF and state.
#[test]
fn rename_history() {
    for directory in [false, true] {
        rename_case(directory);
    }
}

fn rename_case(directory: bool) {
    let fixture = Fixture::new();
    let source = if directory { "old/child.rs" } else { "old.rs" };
    fs::create_dir_all(fixture.0.join(source).parent().unwrap()).unwrap();
    let original = "\u{feff}\t界e\u{301}\r\nlast".as_bytes();
    let source = fixture.file(source, original);
    let selected = if directory {
        fixture.0.join("old")
    } else {
        source.clone()
    };
    let occupied = fixture.file("occupied.go", b"unrelated");
    let mut session = configured(&fixture, &source);
    session.execute(Command::Tree(TreeCommand::Focus));
    session.execute(Command::Tree(TreeCommand::Expand));
    choose(&mut session, &selected);
    let before = snapshot(session.buffer().unwrap());
    assert_eq!(before.history, 0);
    assert_eq!(before.redo, 1);
    assert!(before.modified);
    let views = session
        .windows()
        .iter()
        .map(|(id, view)| (id, view.clone()))
        .collect::<Vec<_>>();
    rename(&mut session, "occupied.go");
    assert!(!session.echo().starts_with("Renamed "));
    assert_eq!(snapshot(session.buffer().unwrap()), before);
    assert_eq!(session.buffer().unwrap().policy().tab_width, 2);
    let permissions = fs::metadata(&fixture.0).unwrap().permissions();
    fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o500)).unwrap();
    rename(&mut session, "denied.go");
    let echo = session.echo().to_owned();
    fs::set_permissions(&fixture.0, permissions).unwrap();
    assert!(echo.contains("Permission denied"), "{echo}");
    assert_eq!(snapshot(session.buffer().unwrap()), before);
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>(),
        views
    );
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(fs::read(&occupied).unwrap(), b"unrelated");
    assert!(!fixture.0.join("denied.go").exists());
    rename(&mut session, "moved.go");
    assert!(session.echo().starts_with("Renamed "), "{}", session.echo());
    let target = fixture.0.join(if directory {
        "moved.go/child.rs"
    } else {
        "moved.go"
    });
    let buffer = session.buffer().unwrap();
    assert_eq!(buffer.file().unwrap().path(), target);
    assert_eq!(buffer.policy().tab_width, if directory { 2 } else { 8 });
    assert_eq!(
        (buffer.stamp(), buffer.state()),
        (before.stamp, before.state)
    );
    assert_eq!(buffer.redo_history().len(), 1);
    assert_eq!(
        session
            .windows()
            .iter()
            .map(|(id, view)| (id, view.clone()))
            .collect::<Vec<_>>(),
        views
    );
    assert!(!selected.exists());
    assert_eq!(fs::read(&target).unwrap(), original);
    session.execute(Command::KeyboardQuit);
    let peer = session.windows().order()[1];
    session.select_view(peer).unwrap();
    saved_future(&mut session, &target, &source, &before.bytes);
    assert_eq!(fs::read(occupied).unwrap(), b"unrelated");
}

fn saved_future(session: &mut Session, target: &Path, old: &Path, bytes: &[u8]) {
    session.execute(Command::SaveBuffer);
    assert_eq!(fs::read(target).unwrap(), bytes);
    assert!(!old.exists());
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Redo));
    assert!(text(session).ends_with('?'));
    assert!(session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Undo));
    assert_eq!(snapshot(session.buffer().unwrap()).bytes, bytes);
    assert!(!session.buffer().unwrap().is_modified());
    session.execute(Command::History(History::Undo));
    assert_eq!(snapshot(session.buffer().unwrap()).bytes, bytes);
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(fs::read(target).unwrap(), bytes);
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.buffer().unwrap().newline(), "\r\n");
    session.frame().unwrap();
    for (_, view) in session.windows().iter() {
        assert_eq!(view.point.stamp, session.buffer().unwrap().stamp());
        assert!(
            rev::coordinates::Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(view.point.position)
                .unwrap()
        );
    }
}
