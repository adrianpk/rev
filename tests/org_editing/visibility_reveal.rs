use rev::command::{History, Motion, OrgCommand};
use rev::coordinates::Cell;

use super::visibility::{OUTLINE, action, visible};
use super::*;

// Search reveals the ancestor path and exact entry, preserving sibling folds in
// this view and all folds in its peer. Cancel restores the saved requested state.
#[test]
fn hierarchy_reveal() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    action(&mut session, OrgCommand::Overview);
    session.execute(Command::Split(Split::Right));
    let peer = session.windows().order()[1];
    let origin = session.view().clone();
    session.execute(Command::SearchForward);
    session.handle(Event::Paste("grand body".into()));
    session.frame().unwrap();
    assert_eq!(visible(&session), [0, 2, 4, 5, 6, 8, 10]);
    let buffer = session.buffer().unwrap();
    let other = session.windows().get(peer).unwrap();
    assert_eq!(
        (0..other.visible_lines(buffer))
            .map(|row| other.source_row(buffer, row).unwrap())
            .collect::<Vec<_>>(),
        [0, 8, 10]
    );
    session.execute(Command::KeyboardQuit);
    assert_eq!(session.view(), &origin);
    session.handle(key(KeyCode::BackTab));
    settle(&mut session);
    assert_eq!(visible(&session), [0, 2, 4, 6, 8, 10]);
    assert_eq!(text(&session), OUTLINE);
}

// Source-targeted motion reveals only the entry path, while ordinary visible-row
// movement skips closed source. Repeated resize retains each view's choice.
#[test]
fn motion_state() {
    let (_fixture, mut session) = editor("", "notes.org", OUTLINE);
    action(&mut session, OrgCommand::Overview);
    session.execute(Command::Scroll(rev::command::Scroll::NextLine));
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap(),
        Cell { row: 8, column: 0 }
    );
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("6".into()));
    session.handle(key(KeyCode::Enter));
    session.frame().unwrap();
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        5
    );
    assert_eq!(visible(&session), [0, 2, 4, 5, 6, 8, 10]);
    for (width, height) in [(2, 2), (0, 0), (80, 24)] {
        session.handle(Event::Resize(width, height));
        session.frame().unwrap();
    }
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 7);
    session.execute(Command::Motion(Motion::BufferStart));
    action(&mut session, OrgCommand::ShowSubtree);
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 10); // Peer body stays closed.
    session.replace(0..0, "!").unwrap();
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 11);
    session.execute(Command::History(History::Undo));
    assert_eq!(session.view().visible_lines(session.buffer().unwrap()), 11);
    assert_eq!(text(&session), OUTLINE);
}

// The same named handlers remain usable on read-only source. Explicitly closing
// a drawer while inside it moves to its real opener, preserving both delimiters.
#[test]
fn readonly_controls() {
    let source = "* Root\r\n:LOGBOOK:\r\nentry\r\n:END:\r\n** Child\r\nbody";
    let (mut buffers, id) = super::folds::buffer(source);
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 12,
        },
    )
    .unwrap();
    session.execute(Command::GotoLine);
    session.handle(Event::Paste("3".into()));
    session.handle(key(KeyCode::Enter));
    action(&mut session, OrgCommand::HideDrawer);
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        1
    );
    let frame = session.frame().unwrap();
    assert!(frame.rows().iter().any(|r| r.text().contains(":LOGBOOK:")));
    assert!(frame.rows().iter().any(|r| r.text().contains(":END:")));
    assert!(!frame.rows().iter().any(|r| r.text().contains("entry")));
    action(&mut session, OrgCommand::ShowAll);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().is_readonly());
    assert!(session.buffer().unwrap().history().is_empty());
}

// Each activated command has a real shared native handler through M-x. An alias
// can select it without changing generic editor bindings or allowing prompt fallthrough.
#[test]
fn named_controls() {
    let source = "* Root\n:LOGBOOK:\nentry\n:END:\n** Child\nbody\n";
    for command in OrgCommand::ALL {
        let (_a, mut direct) = editor("", "notes.org", source);
        let (_b, mut by_name) = editor("", "notes.org", source);
        for session in [&mut direct, &mut by_name] {
            settle(session);
            if matches!(
                command,
                OrgCommand::HideDrawer | OrgCommand::ShowDrawer | OrgCommand::ToggleDrawer
            ) {
                session.execute(Command::GotoLine);
                session.handle(Event::Paste("2".into()));
                session.handle(key(KeyCode::Enter));
            }
        }
        action(&mut direct, command);
        named(&mut by_name, command.name());
        settle(&mut by_name);
        assert_eq!(
            visible(&direct),
            visible(&by_name),
            "{command:?}: {}",
            by_name.echo()
        );
        assert_eq!(
            direct.view().point.position,
            by_name.view().point.position,
            "{command:?}"
        );
        assert_eq!(text(&by_name), source);
        assert!(by_name.buffer().unwrap().history().is_empty());
    }
}

// The finite projection model rejects an oversized observation rather than
// certifying a partial global view. Show-all and editing remain available.
#[test]
fn incomplete_visibility() {
    let source = "* Entry\nbody\n".repeat(4097);
    let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", &source);
    action(&mut session, OrgCommand::Overview);
    assert!(session.echo().contains("incomplete"), "{}", session.echo());
    assert_eq!(
        session.view().visible_lines(session.buffer().unwrap()),
        8195
    );
    action(&mut session, OrgCommand::ShowAll);
    session.insert('!');
    assert_eq!(text(&session), format!("!{source}"));
}

// Delayed intent is qualified by view/source/focus. An edit or transient prompt
// retires it before publication, without a periodic retry or a late fold.
#[test]
fn retired_requests() {
    for name in ["edit", "prompt", "mode"] {
        let (_fixture, mut session) = editor("syntax.enabled = false", "notes.org", OUTLINE);
        session.execute(Command::Org(OrgCommand::Overview));
        match name {
            "edit" => {
                session.insert('!');
            }
            "prompt" => {
                session.execute(Command::FindFile);
            }
            _ => {
                session.execute(Command::Mode(Mode::Text));
            }
        }
        settle(&mut session);
        assert_eq!(
            session.view().visible_lines(session.buffer().unwrap()),
            11,
            "{name}"
        );
        assert_eq!(session.remaining_input(Instant::now()), None, "{name}");
        if name == "edit" {
            assert_eq!(text(&session), format!("!{OUTLINE}"));
        } else {
            assert_eq!(text(&session), OUTLINE);
        }
    }
}
