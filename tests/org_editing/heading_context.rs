use rev::command::{History, Motion, OrgCommand};

use super::headings::goto;
use super::visibility::action;
use super::*;

pub(super) fn meta_return(session: &mut Session) {
    session.handle(Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT)));
    settle(session);
}

// Physical Meta-Return and its exact M-x name share checked insertion. Existing
// drawers, blocks, descendants and blank separators remain one untouched suffix.
#[test]
fn attached_sections() {
    let tail = "\n:PROPERTIES:\n:ID: root\n:END:\n\nbody\n\n#+begin_example\n,* Literal\n#+end_example\n\n** Child\nchild\n* Peer\n";
    let source = format!("* Root{tail}");
    for invocation in ["key", "name", "direct"] {
        let (_fixture, mut session) = editor("", "notes.org", &source);
        session.execute(Command::Motion(Motion::LineEnd));
        match invocation {
            "key" => meta_return(&mut session),
            "name" => {
                named(&mut session, "org-meta-return");
                settle(&mut session);
            }
            _ => action(&mut session, OrgCommand::InsertHeading),
        }
        assert_eq!(text(&session), format!("* Root\n* {tail}"), "{invocation}");
        assert_eq!(session.view().point.position.0, 9, "{invocation}");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), source);
    }
}

// Insertion at EOF keeps the original final-newline state, BOM and separators.
// The initiating point stays on the new title, including wide/combining source.
#[test]
fn source_fidelity() {
    for separator in ["\n", "\r\n"] {
        for final_newline in [false, true] {
            let source = format!(
                "\u{feff}* 界e\u{301}{separator}body{}",
                if final_newline { separator } else { "" }
            );
            let (fixture, mut session) = editor("", "notes.org", &source);
            session.execute(Command::Motion(Motion::BufferEnd));
            let before = text(&session);
            meta_return(&mut session);
            let after = format!(
                "{before}{}* {}",
                if final_newline { "" } else { separator },
                if final_newline { separator } else { "" }
            );
            assert_eq!(text(&session), after);
            assert_eq!(
                session
                    .view()
                    .cell(session.buffer().unwrap())
                    .unwrap()
                    .column,
                2
            );
            session.execute(Command::SaveBuffer);
            let expected = format!("\u{feff}{after}");
            assert_eq!(
                std::fs::read(fixture.0.join("notes.org")).unwrap(),
                expected.as_bytes()
            );
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), before);
            session.execute(Command::History(History::Redo));
            assert_eq!(text(&session), after);
        }
    }
}

// Explicit heading insertion rejects deeper owners. M-RET continues real list
// source while tables, metadata and literal bodies retain their own admission.
#[test]
fn precise_context() {
    let cases = [
        ("list", "* Root\n- item\n", 2),
        ("table", "* Root\n| a | b |\n", 2),
        ("drawer", "* Root\n:LOGBOOK:\nentry\n:END:\n", 3),
        (
            "literal",
            "* Root\n#+begin_example\n,* Literal\n#+end_example\n",
            3,
        ),
        ("planning", "* Root\nSCHEDULED: <2026-10-28 Wed>\n", 2),
        ("malformed", "* Root\n#+begin_src text\n", 2),
        ("list EOF", "* Root\n- item", 0),
        ("table EOF", "* Root\n| a |", 0),
        ("open block EOF", "* Root\n#+begin_example", 0),
    ];
    for (name, source, line) in cases {
        let (_fixture, mut session) = editor("", "notes.org", source);
        if line == 0 {
            session.execute(Command::Motion(Motion::BufferEnd));
        } else {
            goto(&mut session, line);
        }
        let before = session.view().clone();
        action(&mut session, OrgCommand::InsertHeading);
        assert!(
            session.echo().contains("unavailable"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view(), &before, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
        meta_return(&mut session);
        if name.starts_with("list") {
            let expected = if line == 0 {
                "* Root\n- item\n- "
            } else {
                "* Root\n- \n- item\n"
            };
            assert_eq!(text(&session), expected, "{name}");
            assert_eq!(session.buffer().unwrap().history().len(), 1);
            session.execute(Command::History(History::Undo));
            assert_eq!(text(&session), source, "{name}");
            assert_eq!(
                session.view().point.position,
                before.point.position,
                "{name}"
            );
            assert_eq!(
                session.view().point.stamp,
                session.buffer().unwrap().stamp()
            );
            assert_eq!(session.view().top, before.top, "{name}");
            assert!(!session.buffer().unwrap().is_modified(), "{name}");
            continue;
        }
        assert!(
            session.echo().contains("unavailable"),
            "{name}: {}",
            session.echo()
        );
        assert_eq!(text(&session), source, "{name}");
        assert_eq!(session.view(), &before, "{name}");
        assert!(session.buffer().unwrap().history().is_empty(), "{name}");
    }
}
