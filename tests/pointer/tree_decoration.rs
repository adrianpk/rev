use rev::buffer::Buffers;
use rev::command::{Command, TreeCommand};
use rev::config::prepare;
use rev::display::{Position, Size};
use rev::panel::Composition;
use rev::session::Session;
use rev::view::View;

use super::config_fixture::Fixture;
use super::focus::press;
use super::*;

fn editor(
    fixture: &Fixture,
    decorated: bool,
    nerd_fonts: bool,
    activate: bool,
    width: u16,
) -> Session {
    let source = fixture.write("root/source.rs", "retained");
    fixture.write("root/界é.txt", "choice");
    let startup = prepare(&fixture.write("config.toml", &format!("tree.visible=true\ntree.width={width}\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}\nmouse.tree_activate_on_click={activate}")));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&source).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &startup.snapshot);
    Session::new(buffers, view, size()).unwrap()
}

fn size() -> Size {
    Size {
        width: 100,
        height: 10,
    }
}

// Disclosure, icon, name, partial wide glyph and remaining row space all resolve
// the canonical observed path under either click policy and all four settings.
// Selection-only presses also work after the target disappears from disk.
#[test]
fn full_row_actions() {
    for (decorated, nerd_fonts) in [(false, false), (false, true), (true, false), (true, true)] {
        for activate in [false, true] {
            for width in [5, 9, 24] {
                let fixture = Fixture::new();
                let mut session = editor(&fixture, decorated, nerd_fonts, activate, width);
                let target = fixture.0.join("root/界é.txt");
                let root = fixture.0.join("root");
                let stamp = session.buffer().unwrap().stamp();
                if !activate {
                    std::fs::remove_file(&target).unwrap();
                }
                for column in 0..width {
                    session.execute(Command::Tree(TreeCommand::Focus));
                    while session.tree().selected().unwrap().path() != root {
                        session.execute(Command::Tree(TreeCommand::Previous));
                    }
                    session.frame().unwrap();
                    let expected = session.tree().hit(3).unwrap();
                    let position = Position { column, row: 3 };
                    let composition =
                        Composition::workspace(session.windows(), session.tree(), size(), 0)
                            .unwrap();
                    // Tree projection uses these same full content bounds, independently
                    // of what portion of a label survived clipping at this column.
                    assert!(
                        composition
                            .tree()
                            .unwrap()
                            .content()
                            .local(position)
                            .is_some()
                    );
                    assert_eq!(
                        session.handle(press(position)),
                        rev::session::Action::Redraw
                    );
                    assert_eq!(session.tree().selected().unwrap().path(), target);
                    assert_eq!(session.tree().focused(), !activate);
                    assert_eq!(session.tree().hit(3), Some(expected));
                    assert_eq!(
                        session.frame().unwrap().rows()[3].selection(),
                        Some(0..width)
                    );
                    if !activate {
                        session.execute(Command::KeyboardQuit);
                        let frame = session.frame().unwrap();
                        assert_eq!(frame.rows()[2].selection(), Some(0..width));
                        assert_eq!(frame.rows()[3].selection(), None);
                    }
                    if activate {
                        assert_eq!(session.buffer().unwrap().file().unwrap().path(), target);
                    } else {
                        assert_eq!(session.buffer().unwrap().stamp(), stamp);
                    }
                    assert!(session.buffer().unwrap().history().is_empty());
                    assert!(!session.buffer().unwrap().is_modified());
                }
            }
        }
    }
}

// Scroll and hide/restore resize retain cached canonical rows even with all
// observed files gone. Clipped text and font choice cannot become path identity.
#[test]
fn cached_viewport() {
    for (decorated, nerd_fonts) in [(false, false), (false, true), (true, false), (true, true)] {
        let fixture = Fixture::new();
        for index in 0..20 {
            fixture.write(&format!("root/{index:02}-界é.txt"), "cached");
        }
        let mut session = editor(&fixture, decorated, nerd_fonts, false, 9);
        let root_hit = session.tree().hit(0).unwrap();
        std::fs::remove_dir_all(fixture.0.join("root")).unwrap();
        let wheel = Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 4,
            row: 1,
            modifiers: KeyModifiers::NONE,
        });
        assert_eq!(session.handle(wheel), rev::session::Action::Redraw);
        session.frame().unwrap();
        let current = session.tree().hit(0).unwrap();
        assert_ne!(current, root_hit);
        for column in [0, 2, 4, 6, 8] {
            session.frame().unwrap();
            session.handle(press(Position { column, row: 0 }));
            assert_eq!(
                session.tree().selected().unwrap().path(),
                fixture.0.join("root/01-界é.txt")
            );
            assert_eq!(session.tree().hit(0), Some(current.clone()));
        }
        session.handle(Event::Resize(31, 10));
        assert!(!session.tree().focused());
        assert!(
            Composition::workspace(
                session.windows(),
                session.tree(),
                Size {
                    width: 31,
                    height: 10
                },
                0
            )
            .unwrap()
            .tree()
            .is_none()
        );
        session.handle(Event::Resize(100, 10));
        assert!(session.tree().focused());
        assert_eq!(session.tree().hit(0), Some(current));
        session.frame().unwrap();
        assert!(!session.buffer().unwrap().is_modified());
    }
}
