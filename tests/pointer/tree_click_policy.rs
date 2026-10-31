use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, TreeCommand};
use rev::session::{Action, Session};
use rev::view::{Size, View};

use super::config_fixture::Fixture;
use super::focus::press;
use super::tree_click::{cell, editor, source};

fn configured(fixture: &Fixture, enabled: bool, activate: bool) -> Session {
    let settings = format!("mouse.enabled = {enabled}\nmouse.tree_activate_on_click = {activate}");
    let startup = rev::config::prepare(&fixture.write("config.toml", &settings));
    assert!(startup.diagnostic.is_none());
    let path = fixture.write("root/source.txt", "retained");
    fixture.write("root/choice.txt", "choice");
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&path).unwrap();
    let mut view = View::new(buffers.get(id).unwrap());
    view.bookmark_markers = false;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    session.load(&fixture.0.join("root")).unwrap();
    session.execute(Command::KeyboardQuit);
    session.insert('!');
    session.frame().unwrap();
    session
}

// The frozen startup leaf controls activation independently of global admission.
// Selection-only can use a cached missing entry under a denied root without IO;
// restoring the file still permits ordinary keyboard opening. Runtime toggling
// enables the same startup policy after a globally ignored press.
#[test]
fn activation_policy() {
    for enabled in [false, true] {
        for activate in [false, true] {
            let fixture = Fixture::new();
            let mut session = configured(&fixture, enabled, activate);
            let position = cell(&mut session, "choice.txt");
            let stamp = session.buffer().unwrap().stamp();
            let history = session.buffer().unwrap().history().len();
            let target = fixture.0.join("root/choice.txt");
            fixture.write(
                "config.toml",
                &format!(
                    "mouse.enabled = true\nmouse.tree_activate_on_click = {}",
                    !activate
                ),
            );
            let permissions = fs::metadata(fixture.0.join("root")).unwrap().permissions();
            if !activate && enabled {
                fs::remove_file(&target).unwrap();
                fs::set_permissions(fixture.0.join("root"), fs::Permissions::from_mode(0o000))
                    .unwrap();
            }
            let action = session.handle(press(position));
            fs::set_permissions(fixture.0.join("root"), permissions).unwrap();
            if enabled && activate {
                assert_eq!(source(&session), "choice");
                assert!(!session.tree().focused());
            } else {
                assert_eq!(session.buffer().unwrap().stamp(), stamp);
                assert_eq!(session.buffer().unwrap().history().len(), history);
                assert_eq!(source(&session), "!retained");
                assert_eq!(session.tree().focused(), enabled);
                assert_eq!(
                    action,
                    if enabled {
                        Action::Redraw
                    } else {
                        Action::Wait
                    }
                );
            }
            if enabled && !activate {
                assert_eq!(session.tree().selected().unwrap().path(), target);
                assert!(!target.exists());
                fixture.write("root/choice.txt", "choice");
                session.handle(Event::Key(KeyEvent::new(
                    KeyCode::Enter,
                    KeyModifiers::NONE,
                )));
                assert_eq!(source(&session), "choice");
            } else if !enabled {
                assert_eq!(
                    session.tree().selected().unwrap().path(),
                    fixture.0.join("root")
                );
                session.execute(Command::MouseMode);
                session.handle(press(position));
                assert_eq!(session.tree().selected().unwrap().path(), target);
                assert_eq!(session.tree().focused(), !activate);
                assert_eq!(
                    source(&session),
                    if activate { "choice" } else { "!retained" }
                );
            }
            assert_eq!(
                fs::read(fixture.0.join("root/source.txt")).unwrap(),
                b"retained"
            );
            assert_eq!(fs::read(target).unwrap(), b"choice");
        }
    }
}

// File aliases retain the canonical dirty buffer, point and history through
// the actual click owner; a tree label/link never manufactures a second source.
#[test]
fn canonical_alias() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    symlink("source.txt", fixture.0.join("root/link.txt")).unwrap();
    session.execute(Command::Tree(TreeCommand::Focus));
    session.execute(Command::Tree(TreeCommand::Refresh));
    session.execute(Command::KeyboardQuit);
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    let view = session.view().clone();
    let position = cell(&mut session, "link.txt");
    session.handle(press(position));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(*session.view(), view);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    assert_eq!(source(&session), "!retained\ntext");
    assert_eq!(
        session.buffer().unwrap().file().unwrap().path(),
        fixture.0.join("root/source.txt")
    );
    assert_eq!(
        session.tree().selected().unwrap().path(),
        fixture.0.join("root/link.txt")
    );
    assert!(
        fs::symlink_metadata(fixture.0.join("root/link.txt"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read(fixture.0.join("root/source.txt")).unwrap(),
        b"retained\ntext"
    );
}

// A denied directory action keeps the selected cached entry collapsed and
// leaves dirty source/history intact; restoring owned access allows retry.
#[test]
fn denied_directory() {
    let fixture = Fixture::new();
    let mut session = editor(&fixture);
    let path = fixture.0.join("root/child");
    fixture.write("root/child/nested", "child");
    session.execute(Command::Tree(TreeCommand::Focus));
    session.execute(Command::Tree(TreeCommand::Refresh));
    session.execute(Command::KeyboardQuit);
    session.insert('!');
    let position = cell(&mut session, "child/");
    let stamp = session.buffer().unwrap().stamp();
    let permissions = fs::metadata(&path).unwrap().permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    session.handle(press(position));
    let error = session.echo().to_owned();
    fs::set_permissions(&path, permissions).unwrap();
    assert!(error.contains("Permission denied"));
    assert_eq!(session.tree().selected().unwrap().path(), path);
    assert!(!session.tree().selected().unwrap().expanded());
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.handle(press(position));
    assert!(session.tree().selected().unwrap().expanded());
    assert_eq!(source(&session), "!retained\ntext");
}
