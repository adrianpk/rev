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
    action(&mut session, OrgCommand::HideAllDrawers);
    assert!(!visible(&session).contains(&2));
    action(&mut session, OrgCommand::ToggleAllDrawers);
    assert!(visible(&session).contains(&2));
    action(&mut session, OrgCommand::CycleSetStartupVisibility);
    assert_eq!(visible(&session), (0..6).collect::<Vec<_>>());
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().is_readonly());
    assert!(session.buffer().unwrap().history().is_empty());
}

// Each activated command has a real shared native handler through M-x. An alias
// can select it without changing generic editor bindings or allowing prompt fallthrough.
#[test]
fn named_controls() {
    let source = "* Root\n:LOGBOOK:\nentry\n:END:\n** Child\nbody\n";
    for command in [
        OrgCommand::Cycle,
        OrgCommand::GlobalCycle,
        OrgCommand::Overview,
        OrgCommand::Contents,
        OrgCommand::ShowAll,
        OrgCommand::HideSubtree,
        OrgCommand::ShowChildren,
        OrgCommand::ShowSubtree,
        OrgCommand::HideDrawer,
        OrgCommand::ShowDrawer,
        OrgCommand::ToggleDrawer,
    ] {
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
        let (_fixture, mut session) =
            unobserved_editor("syntax.enabled = false", "notes.org", OUTLINE);
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

// Runtime choices suppress a pending initial policy, including a queued drawer
// operation. Once admitted, splits, switches and redraws preserve per-window choices.
#[test]
fn startup_lifetime_visibility() {
    let source = "* Root\n:REQUEST:\nentry\n:END:\n** Child\nbody\n";
    for (name, command, expected) in [
        (
            "full reveal",
            OrgCommand::ShowAll,
            (0..7).collect::<Vec<_>>(),
        ),
        (
            "queued drawer",
            OrgCommand::HideAllDrawers,
            vec![0, 1, 3, 4, 5, 6],
        ),
    ] {
        let (fixture, mut session) = unobserved_editor(
            "syntax.enabled=false\norg.startup_folded='overview'",
            "notes.org",
            source,
        );
        session.execute(Command::Org(command));
        settle(&mut session);
        assert_eq!(visible(&session), expected, "{name}");
        session.execute(Command::Split(Split::Right));
        let peer = session.windows().order()[1];
        action(&mut session, OrgCommand::CycleSetStartupVisibility);
        assert_eq!(visible(&session), [0, 6]);
        let b = session.buffer().unwrap();
        let other = session.windows().get(peer).unwrap();
        assert_eq!(
            (0..other.visible_lines(b))
                .map(|r| other.source_row(b, r).unwrap())
                .collect::<Vec<_>>(),
            expected
        );
        session.load(&fixture.write("plain.txt", "plain")).unwrap();
        settle(&mut session);
        session.load(&fixture.0.join("notes.org")).unwrap();
        settle(&mut session);
        assert_eq!(visible(&session), [0, 6]);
        action(&mut session, OrgCommand::ShowAll);
        for (width, height) in [(2, 2), (0, 0), (100, 20)] {
            session.handle(Event::Resize(width, height));
            settle(&mut session);
            assert_eq!(visible(&session), (0..7).collect::<Vec<_>>());
        }
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// A checked disk reload retires old ranges without restoring startup. Reset
// resolves the current document, and a genuinely reopened view initializes anew.
#[test]
fn reload_startup_visibility() {
    let (fixture, mut session) = editor(
        "files.external_changes='ask'\norg.startup_folded='overview'",
        "notes.org",
        OUTLINE,
    );
    settle(&mut session);
    assert_eq!(visible(&session), [0, 8, 10]);
    let source = "#+STARTUP: content\n* New\nbody\n** Child\nbody";
    fixture.write("notes.org", source);
    session.execute(Command::RevertBuffer);
    let until = Instant::now() + std::time::Duration::from_secs(3);
    while text(&session) != source {
        assert!(Instant::now() < until, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::yield_now();
    }
    settle(&mut session);
    assert_eq!(visible(&session), (0..5).collect::<Vec<_>>());
    action(&mut session, OrgCommand::CycleSetStartupVisibility);
    assert_eq!(visible(&session), [0, 1, 3]);
    session.execute(Command::Motion(Motion::BufferStart));
    session.replace(0..0, " ").unwrap();
    settle(&mut session);
    assert_eq!(visible(&session), (0..5).collect::<Vec<_>>());
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(visible(&session), (0..5).collect::<Vec<_>>());
    // A fresh session owns a genuinely new view of the same document.
    let (_fresh_fixture, mut fresh) = editor("org.startup_folded='overview'", "notes.org", source);
    settle(&mut fresh);
    assert_eq!(visible(&fresh), [0, 1, 3]);
}

// Empty/read-only targets are harmless; malformed and over-limit structure
// cannot certify global drawer ranges. Explicit full reveal always remains usable.
#[test]
fn drawer_visibility_admission() {
    for (name, source, rejected) in [
        ("empty", "", false),
        ("prose", "* Root\nprose", false),
        ("unclosed drawer", "* Root\n:REQUEST:\nbody", true),
        (
            "unclosed example",
            "* Root\n#+begin_example\n:REQUEST:\nbody\n:END:",
            true,
        ),
    ] {
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
        let stamp = session.buffer().unwrap().stamp();
        for command in [
            OrgCommand::HideAllDrawers,
            OrgCommand::ShowAllDrawers,
            OrgCommand::ToggleAllDrawers,
            OrgCommand::CycleSetStartupVisibility,
        ] {
            action(&mut session, command);
            if rejected {
                assert!(
                    session.echo().contains("malformed"),
                    "{name}: {}",
                    session.echo()
                );
            }
            assert_eq!(
                visible(&session),
                (0..source.lines().count().max(1)).collect::<Vec<_>>(),
                "{name}"
            );
            assert_eq!(text(&session), source);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert!(session.buffer().unwrap().history().is_empty());
            assert!(session.buffer().unwrap().is_readonly());
        }
    }
    for source in ["* Root\n:REQUEST:\nbody", "* Root\n#+begin_example\nbody"] {
        let (_fixture, mut automatic) =
            editor("org.startup_folded='overview'", "notes.org", source);
        settle(&mut automatic);
        assert_eq!(
            visible(&automatic),
            (0..source.lines().count()).collect::<Vec<_>>()
        );
    }
    let source = "* Entry\nbody\n".repeat(4097);
    let (_fixture, mut session) = editor("org.startup_folded='showall'", "notes.org", &source);
    action(&mut session, OrgCommand::HideAllDrawers);
    assert!(session.echo().contains("incomplete"), "{}", session.echo());
    assert_eq!(visible(&session).len(), 8195);
    action(&mut session, OrgCommand::ShowAll);
    assert_eq!(visible(&session).len(), 8195);
}

// Each profile invokes the exact names once through its native selector.
// Actual usage persistence and unchanged source/mark/history guard the UI boundary.
#[test]
fn profile_visibility_commands() {
    use rev::command::usage::{History as UsageHistory, Store};
    let source = "* Root\n:REQUEST:\nentry\n:END:\nprose\n";
    for profile in ["non-modal", "modal"] {
        for (command, expected) in [
            (OrgCommand::HideAllDrawers, vec![0, 1, 3, 4, 5]),
            (OrgCommand::ShowAllDrawers, (0..6).collect()),
            (OrgCommand::ToggleAllDrawers, vec![0, 1, 3, 4, 5]),
            (OrgCommand::CycleSetStartupVisibility, vec![0, 1, 3, 4, 5]),
        ] {
            let config =
                format!("editor.interaction_profile='{profile}'\norg.startup_folded='showall'");
            let (fixture, mut session) = editor(&config, "notes.org", source);
            settle(&mut session);
            action(&mut session, OrgCommand::ShowAll);
            let path = fixture.0.join("usage.toml");
            session = session.with_command_usage(UsageHistory::open(&path));
            let stamp = session.buffer().unwrap().stamp();
            session.execute(Command::Region(rev::command::RegionCommand::Set));
            let mark = session.buffer().unwrap().mark();
            if profile == "modal" {
                session.handle(key(KeyCode::Char(':')));
            } else {
                session.handle(meta('x'));
            }
            session.handle(Event::Paste(command.name().into()));
            session.handle(key(KeyCode::Tab));
            session.handle(key(KeyCode::Enter));
            settle(&mut session);
            assert_eq!(
                visible(&session),
                expected,
                "{profile}: {command:?}: {}",
                session.echo()
            );
            session.flush_command_usage().unwrap();
            assert_eq!(Store::open(&path).unwrap().1, [command.name()], "{profile}");
            assert_eq!(session.buffer().unwrap().mark(), mark);
            assert_eq!(session.buffer().unwrap().stamp(), stamp);
            assert_eq!(text(&session), source);
            assert!(!session.buffer().unwrap().is_modified());
            assert!(session.buffer().unwrap().history().is_empty());
        }
    }
}
