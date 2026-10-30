use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyEvent};
use rev::{
    buffer::Buffers,
    command::{Binding, Command, OrgCommand},
    config::{Snapshot, default_template, prepare},
    input::{Input, OrgContext, Outcome, keymap::Sequence},
    session::Session,
    view::{Size, View},
};

fn press(input: &mut Input, sequence: &Sequence) -> Outcome {
    let mut outcome = Outcome::Ignored;
    for key in sequence.chords() {
        outcome = input.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
    }
    outcome
}

// Exercise every inherited Org-role command chord through native input, using
// both absent-file defaults and the real packaged template. Identity equality
// here complements feature suites that exercise the shared command handlers.
#[test]
fn default_routes() {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", default_template()));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    for snapshot in [Snapshot::default(), startup.snapshot] {
        for role in OrgContext::ALL {
            check_role(&snapshot, role);
        }
    }
}

fn check_role(snapshot: &Snapshot, role: OrgContext) {
    for (chord, name) in role.keymap().defaults() {
        let Some(Binding::Command(command)) = Binding::named(name) else {
            continue;
        };
        let mut input = Input::with_settings(snapshot);
        input.set_org_context(Some(role));
        let sequence = Sequence::parse(chord).unwrap();
        assert_eq!(
            press(&mut input, &sequence),
            Outcome::Command(command),
            "{role:?}: {chord} -> {name}"
        );
        assert_eq!(Command::named(name), Some(command), "{name}");
    }
}

fn session(path: &std::path::Path) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(true);
    let view = View::new(buffer);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}

fn observed(session: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while session.echo() == "Org structural observation is pending; C-g cancels" {
        assert!(Instant::now() < deadline, "{}", session.echo());
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn outline_notice(command: OrgCommand) -> bool {
    use OrgCommand::*;
    matches!(
        command,
        Cycle
            | GlobalCycle
            | Overview
            | Contents
            | ShowAll
            | HideSubtree
            | ShowChildren
            | ShowSubtree
            | HideDrawer
            | ShowDrawer
            | ToggleDrawer
            | NextHeading
            | PreviousHeading
            | ForwardHeading
            | BackwardHeading
            | ParentHeading
    )
}

// The entire Org registry must enter real Org dispatch, including wrong-context
// diagnostics. An unhandled command must not silently fall through to the
// outline visibility fallback. Read-only source blocks all edit publication.
#[test]
fn native_catalog() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("source.org", "* TODO Parent\nBody\n** TODO Child\nText\n");
    for command in OrgCommand::ALL {
        let mut session = session(&path);
        let stamp = session.buffer().unwrap().stamp();
        session.execute(Command::Org(command));
        observed(&mut session);
        assert_ne!(
            session.echo(),
            "Command is not a text edit",
            "{}",
            command.name()
        );
        assert_ne!(
            session.echo(),
            "Native Org mode is not active",
            "{}",
            command.name()
        );
        if !outline_notice(command) {
            assert_ne!(
                session.echo(),
                command.name(),
                "unhandled {} reached visibility",
                command.name()
            );
        }
        assert_eq!(
            session.buffer().unwrap().stamp(),
            stamp,
            "{}",
            command.name()
        );
        assert_eq!(
            session.buffer().unwrap().history().len(),
            0,
            "{}",
            command.name()
        );
        session.execute(Command::KeyboardQuit);
    }
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        "* TODO Parent\nBody\n** TODO Child\nText\n"
    );
    assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
}

// Capability prose must name only real compiled commands. This catches a
// plausible-looking operation added to reference text before its implementation.
#[test]
fn control_reference() {
    let reference = include_str!("../../docs/reference/org-controls.md");
    let names = reference
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|name| name.starts_with("org-") || *name == "text-mode");
    let mut count = 0;
    for name in names {
        assert!(
            Command::named(name).is_some(),
            "unsupported documented command: {name}"
        );
        count += 1;
    }
    assert!(count > 50);
}
