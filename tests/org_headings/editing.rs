use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{LinearCommand, Motion, RegionCommand};
use rev::input::keymap::Context;

use super::controls::{key, selector};
use super::*;

// User mappings reach the same native handlers in each effective profile map;
// neither command acquires a factory shortcut in any existing context.
#[test]
fn configured_bindings() {
    let defaults = rev::config::Snapshot::default();
    for context in Context::ALL {
        assert!(
            defaults
                .keymaps()
                .map(context)
                .bindings()
                .all(|(_, binding)| {
                    !matches!(
                        binding.name(),
                        "org-pretty-headings-mode" | "org-reset-heading-presentation"
                    )
                })
        );
    }
    for (profile, context) in [("non-modal", "org"), ("modal", "vim_normal")] {
        let config = format!(
            "editor.interaction_profile='{profile}'\n[keys.{context}]\n'M-j'='org-pretty-headings-mode'\n'M-r'='org-reset-heading-presentation'"
        );
        let (fixture, mut session) = editor(&config, "*** Heading\n");
        settle(&mut session);
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::ALT,
        )));
        assert_eq!(
            session.view().heading_presentation_override(),
            Some(false),
            "{profile}"
        );
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('r'),
            KeyModifiers::ALT,
        )));
        assert_eq!(
            session.view().heading_presentation_override(),
            None,
            "{profile}"
        );
        assert_eq!(
            std::fs::read_to_string(fixture.0.join("config.toml")).unwrap(),
            config
        );
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Both native selection owners retain canonical stars through a presentation
// command. Actual region/Visual copy, insertion and undo never consume markers.
#[test]
fn source_copy_and_edit() {
    for profile in ["non-modal", "modal"] {
        let (_fixture, mut session) = editor(
            &format!("editor.interaction_profile='{profile}'"),
            "*** Heading\n",
        );
        settle(&mut session);
        if profile == "modal" {
            for c in "vll".chars() {
                session.handle(key(KeyCode::Char(c)));
            }
        } else {
            session.execute(Command::Region(RegionCommand::Set));
            for _ in 0..3 {
                session.execute(Command::Motion(Motion::ForwardChar));
            }
        }
        let point = session.view().point;
        let mark = session.buffer().unwrap().mark();
        selector(&mut session, "org-pretty-headings-mode", profile == "modal");
        assert!(!session.view().pretty_headings(), "{profile}");
        assert_eq!(session.view().point, point);
        assert_eq!(session.buffer().unwrap().mark(), mark);
        assert_eq!(
            session.frame().unwrap().cursor(),
            Some(rev::display::Position {
                row: 0,
                column: 7 + point.position.0 as u16,
            })
        );
        if profile == "modal" {
            assert!(
                session
                    .editing_state()
                    .unwrap()
                    .label()
                    .starts_with("VISUAL")
            );
            session.handle(key(KeyCode::Char('y')));
        } else {
            session.execute(Command::Linear(LinearCommand::Copy));
        }
        assert_eq!(
            session.killed_text().unwrap().to_string(),
            "***",
            "{profile}"
        );
        assert_eq!(text(session.buffer().unwrap()), "*** Heading\n");
        assert!(session.buffer().unwrap().history().is_empty());
        session.execute(Command::Motion(Motion::BufferStart));
        if profile == "modal" {
            session.handle(key(KeyCode::Char('i')));
        }
        session.handle(key(KeyCode::Char('*')));
        if profile == "modal" {
            session.handle(key(KeyCode::Esc));
        }
        settle(&mut session);
        assert_eq!(
            text(session.buffer().unwrap()),
            "**** Heading\n",
            "{profile}"
        );
        assert_eq!(session.buffer().unwrap().history().len(), 1, "{profile}");
        session.execute(Command::History(History::Undo));
        settle(&mut session);
        assert_eq!(
            text(session.buffer().unwrap()),
            "*** Heading\n",
            "{profile}"
        );
        assert!(!session.view().pretty_headings());
        assert!(!session.buffer().unwrap().is_modified());
    }
}
