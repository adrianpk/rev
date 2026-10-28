use rev::buffer::Mode;
use rev::command::{Command, OrgCommand};
use rev::input::{Focus, Input, OrgContext, Outcome};

use super::{Event, KeyCode, control, key};

// Search, command entry, arguments, decisions and inventories own cancellation
// above every Org role. No consumed prompt suffix reaches underlying editing.
#[test]
fn transient_cancellation() {
    for context in OrgContext::ALL {
        for stage in ["search", "command", "argument", "decision", "inventory"] {
            let mut input = Input::default();
            input.set_org_context(Some(context));
            open(&mut input, stage);
            assert_ne!(input.focus(), Focus::Editing, "{context:?} {stage}");
            assert_eq!(
                input.handle(control('g')),
                Outcome::Command(Command::KeyboardQuit)
            );
            assert_eq!(input.focus(), Focus::Editing);
            assert_eq!(input.org_context(), Some(context));
            assert_eq!(input.prefix(), None);
        }
    }
}

fn open(input: &mut Input, stage: &str) {
    match stage {
        "search" => input.search(rev::search::Direction::Forward).unwrap(),
        "command" => input.commands().unwrap(),
        "argument" => input.argument("Draft: ", "kept").unwrap(),
        "decision" => input.decision("Continue?").unwrap(),
        "inventory" => input.inventory().unwrap(),
        _ => unreachable!("named transient fixture"),
    }
}

// The public palette contains working mode/move commands and no future feature
// placeholders. A narrowed mode prefix completes to its compiled identity.
#[test]
fn operational_catalog() {
    let mut input = Input::default();
    input.commands().unwrap();
    input.handle(Event::Paste("org-mo".into()));
    assert_eq!(
        input.palette().unwrap().candidates(),
        [
            Command::Mode(Mode::Org),
            Command::Org(OrgCommand::MoveItemDown),
            Command::Org(OrgCommand::MoveItemUp),
            Command::Org(OrgCommand::MoveSubtreeDown),
            Command::Org(OrgCommand::MoveSubtreeUp),
        ]
    );
    input.handle(Event::Paste("d".into()));
    assert_eq!(
        input.palette().unwrap().candidates(),
        [Command::Mode(Mode::Org)]
    );
    assert_eq!(input.handle(key(KeyCode::Tab)), Outcome::Changed);
    assert_eq!(input.prompt().unwrap().text(), "org-mode");
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Command(Command::Mode(Mode::Org))
    );
    for absent in [
        "org-agenda",
        "org-capture",
        "org-export-dispatch",
        "org-ctrl-c-ctrl-c",
        "org-edit-special",
    ] {
        assert_eq!(Command::named(absent), None, "{absent}");
    }
}
