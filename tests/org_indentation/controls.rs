use rev::buffer::Mode;
use rev::command::{BufferCommand, RegionCommand};

use super::*;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn selector(session: &mut Session, name: &str, colon: bool) {
    session.handle(if colon {
        key(KeyCode::Char(':'))
    } else {
        Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
    });
    session.handle(Event::Paste(name.into()));
    session.handle(key(KeyCode::Enter));
}

fn control(session: &mut Session, command: OrgCommand, enabled: bool) {
    session.execute(Command::Org(command));
    assert_eq!(session.buffer().unwrap().org_indentation(), enabled);
    assert_eq!(
        session.echo(),
        if enabled {
            "Org indentation enabled"
        } else {
            "Org indentation disabled"
        }
    );
}

// Shared native selectors toggle the startup value and admit explicit,
// idempotent choices without source, history, saved-state or TOML changes.
#[test]
fn native_selectors() {
    for (profile, colon) in [("non-modal", false), ("modal", false), ("modal", true)] {
        for startup in [true, false] {
            let config = format!("editor.interaction_profile='{profile}'\norg.indent={startup}");
            let (f, mut session) = editor(&config, "** H\nbody\n", 30, 10);
            let config = std::fs::read(f.0.join("config.toml")).unwrap();
            let point = session.view().point;
            let stamp = session.buffer().unwrap().stamp();
            for (name, enabled) in [
                ("org-indent-mode", !startup),
                ("org-indent-mode", startup),
                ("org-indent-enable", true),
                ("org-indent-enable", true),
                ("org-indent-disable", false),
                ("org-indent-disable", false),
            ] {
                selector(&mut session, name, colon);
                assert_eq!(session.buffer().unwrap().org_indentation(), enabled);
                assert_eq!(
                    session.echo(),
                    if enabled {
                        "Org indentation enabled"
                    } else {
                        "Org indentation disabled"
                    }
                );
                settle(&mut session);
                assert_eq!(session.view().point, point);
                assert_eq!(session.buffer().unwrap().stamp(), stamp);
                assert_eq!(
                    session.frame().unwrap().rows()[1].text().trim_end(),
                    if enabled { "    body" } else { "body" }
                );
            }
            assert_eq!(std::fs::read(f.0.join("config.toml")).unwrap(), config);
            assert_eq!(
                std::fs::read_to_string(f.0.join("notes.org")).unwrap(),
                "** H\nbody\n"
            );
            assert_eq!(source(&session), "** H\nbody\n");
            assert!(session.buffer().unwrap().history().is_empty());
            assert!(!session.buffer().unwrap().is_modified());
        }
    }
}

// Explicit keymaps bind the same three names in each profile. Prompt ownership
// and cancellation cannot invoke a presentation command behind the selector.
#[test]
fn configured_bindings() {
    for (profile, context) in [("non-modal", "org"), ("modal", "vim_normal")] {
        let config = format!(
            "editor.interaction_profile='{profile}'\n[keys.{context}]\n'M-j'='org-indent-mode'\n'M-k'='org-indent-enable'\n'M-l'='org-indent-disable'"
        );
        let (_f, mut session) = editor(&config, "** H\nbody\n", 30, 10);
        for (ch, enabled) in [('j', false), ('k', true), ('k', true), ('l', false)] {
            session.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::ALT,
            )));
            assert_eq!(session.buffer().unwrap().org_indentation(), enabled);
        }
        session.execute(Command::ExecuteExtended);
        session.handle(Event::Paste("org-indent-enable".into()));
        session.handle(key(KeyCode::Tab));
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('k'),
            KeyModifiers::ALT,
        )));
        assert!(!session.buffer().unwrap().org_indentation());
        session.execute(Command::KeyboardQuit);
        assert!(!session.buffer().unwrap().org_indentation());
        assert_eq!(source(&session), "** H\nbody\n");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Buffer choices are shared by split views but independent of pretty headings,
// wrapping and other buffers. Text mode suspends the choice; close retires it.
#[test]
fn buffer_lifecycle() {
    for profile in ["non-modal", "modal"] {
        let (f, mut session) = editor(
            &format!("editor.interaction_profile='{profile}'\norg.indent=false"),
            "** H\nbody\n* Peer\nother\n",
            64,
            14,
        );
        control(&mut session, OrgCommand::IndentEnable, true);
        let original = session.buffer().unwrap().id();
        session.execute(Command::Split(Split::Right));
        session.execute(Command::Org(OrgCommand::PrettyHeadings));
        let selected = session.windows().selected_id();
        for id in session.windows().order() {
            session.select_view(id).unwrap();
            assert!(session.buffer().unwrap().org_indentation());
            assert_eq!(session.view().pretty_headings(), id != selected);
        }
        session.select_view(selected).unwrap();
        session.execute(Command::VisualLineMode);
        assert!(!session.buffer().unwrap().soft_wrap());
        assert!(session.buffer().unwrap().org_indentation());
        session.execute(Command::Mode(Mode::Text));
        let origin = session.view().clone();
        for command in [
            OrgCommand::IndentMode,
            OrgCommand::IndentEnable,
            OrgCommand::IndentDisable,
        ] {
            session.execute(Command::Org(command));
            assert_eq!(session.echo(), "Native Org mode is not active");
            assert_eq!(session.view(), &origin);
        }
        assert!(!session.buffer().unwrap().org_indentation());
        session.execute(Command::Mode(Mode::Org));
        settle(&mut session);
        assert!(session.buffer().unwrap().org_indentation());
        session.execute(Command::Org(OrgCommand::HideSubtree));
        settle(&mut session);
        assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 4);
        session.handle(Event::Resize(42, 12));
        settle(&mut session);
        control(&mut session, OrgCommand::IndentDisable, false);
        control(&mut session, OrgCommand::IndentEnable, true);
        assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 4);
        let other = f.write("other.org", "** Other\nbody\n");
        session.load(&other).unwrap();
        assert!(!session.buffer().unwrap().org_indentation());
        assert!(session.buffer().unwrap().soft_wrap());
        control(&mut session, OrgCommand::IndentDisable, false);
        session.load(&f.0.join("notes.org")).unwrap();
        assert!(session.buffer().unwrap().org_indentation());
        assert!(!session.buffer().unwrap().soft_wrap());
        assert!(!session.view().pretty_headings());
        session.execute(Command::Buffer(BufferCommand::Kill));
        session.handle(key(KeyCode::Enter));
        session.load(&f.0.join("notes.org")).unwrap();
        assert_ne!(session.buffer().unwrap().id(), original);
        assert!(!session.buffer().unwrap().org_indentation());
        assert!(session.buffer().unwrap().soft_wrap());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// A policy change reflows the frame without changing its source stamp.
// Current pointer hits and selection anchors still refer to source only.
#[test]
fn runtime_reflow() {
    for wrapped in [true, false] {
        let (_f, mut session) = editor(
            &format!("editor.soft_wrap={wrapped}"),
            "** H\nabcdefghijklmno\nsecond\n",
            12,
            12,
        );
        goto(&mut session, 2);
        session.execute(Command::Motion(Motion::ForwardChar));
        session.execute(Command::Region(RegionCommand::Set));
        session.execute(Command::Motion(Motion::ForwardChar));
        let point = session.view().point;
        let mark = session.buffer().unwrap().mark();
        for enabled in [false, true, false] {
            control(
                &mut session,
                if enabled {
                    OrgCommand::IndentEnable
                } else {
                    OrgCommand::IndentDisable
                },
                enabled,
            );
            settle(&mut session);
            assert_eq!(session.view().point, point);
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert!(session.buffer().unwrap().mark_active());
            let frame = session.frame().unwrap();
            assert_eq!(
                frame.cursor(),
                Some(Position {
                    row: 1,
                    column: if enabled { 6 } else { 2 }
                })
            );
            assert_eq!(
                frame.rows()[1].selection(),
                Some(if enabled { 5..6 } else { 1..2 })
            );
        }
        session.execute(Command::KeyboardQuit);
        control(&mut session, OrgCommand::IndentEnable, true);
        settle(&mut session);
        mouse(&mut session, MouseEventKind::Down(MouseButton::Left), 1, 0);
        assert_eq!(session.view().point.position, Scalar(5));
        assert_eq!(source(&session), "** H\nabcdefghijklmno\nsecond\n");
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Source changes recompute the enclosing heading in both panes. Undo/redo and
// structural moves retain an explicit choice while preserving source history.
#[test]
fn structural_choices() {
    for profile in ["non-modal", "modal"] {
        let (_f, mut session) = editor(
            &format!(
                "editor.interaction_profile='{profile}'\norg.indent=false\nsyntax.enabled=false"
            ),
            "* Root\n** H\nbody\n** Peer\nother\n",
            60,
            14,
        );
        control(&mut session, OrgCommand::IndentEnable, true);
        session.execute(Command::Split(Split::Right));
        goto(&mut session, 2);
        session.insert('*');
        settle(&mut session);
        for (_, view) in session.windows().iter() {
            let frame = Frame::project(
                session.buffer().unwrap(),
                view,
                Size {
                    width: 24,
                    height: 10,
                },
                "",
            )
            .unwrap();
            assert_eq!(frame.rows()[2].text().trim_end(), "      body");
        }
        for (history, expected) in [
            (History::Undo, "    body"),
            (History::Redo, "      body"),
            (History::Undo, "    body"),
        ] {
            session.execute(Command::History(history));
            settle(&mut session);
            assert!(session.buffer().unwrap().org_indentation());
            let frame = Frame::project(
                session.buffer().unwrap(),
                session.view(),
                Size {
                    width: 24,
                    height: 10,
                },
                "",
            )
            .unwrap();
            assert_eq!(frame.rows()[2].text().trim_end(), expected);
        }
        session.execute(Command::Org(OrgCommand::MoveSubtreeDown));
        settle(&mut session);
        assert_eq!(
            source(&session),
            "* Root\n** Peer\nother\n** H\nbody\n",
            "{}",
            session.echo()
        );
        assert!(session.buffer().unwrap().org_indentation());
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(source(&session), "* Root\n** H\nbody\n** Peer\nother\n");
        assert!(session.buffer().unwrap().org_indentation());
    }
}

// A real checked external reload changes only canonical text. Its settings
// reapplication and undo/redo cannot reset explicit enabled or disabled choices.
#[test]
fn reload_choices() {
    for (profile, startup, enabled) in [("non-modal", true, false), ("modal", false, true)] {
        let (f, mut session) = editor(
            &format!(
                "editor.interaction_profile='{profile}'\norg.indent={startup}\nfiles.external_changes='ask'"
            ),
            "** H\nbody\n",
            50,
            12,
        );
        control(&mut session, OrgCommand::IndentMode, enabled);
        session.execute(Command::Split(Split::Right));
        let ids = session.windows().order();
        f.write("notes.org", "*** New\nfresh\n");
        session.execute(Command::RevertBuffer);
        let deadline = Instant::now() + Duration::from_secs(3);
        while source(&session) != "*** New\nfresh\n" {
            assert!(Instant::now() < deadline, "{}", session.echo());
            session.expire_input(Instant::now());
            std::thread::sleep(Duration::from_millis(1));
        }
        settle(&mut session);
        for id in ids {
            session.select_view(id).unwrap();
            assert_eq!(session.buffer().unwrap().org_indentation(), enabled);
            let frame = Frame::project(
                session.buffer().unwrap(),
                session.view(),
                Size {
                    width: 24,
                    height: 10,
                },
                "",
            )
            .unwrap();
            assert_eq!(
                frame.rows()[1].text().trim_end(),
                if enabled { "      fresh" } else { "fresh" }
            );
        }
        for history in [History::Undo, History::Redo] {
            session.execute(Command::History(history));
            settle(&mut session);
            assert_eq!(session.buffer().unwrap().org_indentation(), enabled);
        }
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}

// A saved screen-column goal cannot survive a margin change, including in the
// unselected split. Vertical movement must start from the current source point.
#[test]
fn navigation_goals() {
    let (_f, mut session) = editor("", "** H\nbody words\nnext line\n", 60, 14);
    for _ in 0..3 {
        session.execute(Command::Motion(Motion::ForwardChar));
    }
    session.execute(Command::Scroll(Scroll::NextLine));
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        Cell { row: 1, column: 0 }
    );
    assert_eq!(session.view().column, Some(4));
    session.execute(Command::Split(Split::Right));
    control(&mut session, OrgCommand::IndentDisable, false);
    settle(&mut session);
    for (_, view) in session.windows().iter() {
        assert_eq!(view.column, None);
    }
    session.execute(Command::Scroll(Scroll::NextLine));
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        Cell { row: 2, column: 0 }
    );
}
