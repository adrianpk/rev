use rev::command::{Command, OrgCommand};
use rev::input::{Input, OrgContext, Outcome};

use super::*;

// Compiled identities round-trip, while native/familiar key reports and named
// entry resolve the same handlers. Plain editing, capture and prompts keep TAB.
#[test]
fn visibility_keys() {
    let mut input = Input::default();
    assert_eq!(
        input.handle(key(KeyCode::Tab)),
        Outcome::Command(Command::InsertTab)
    );
    input.set_org_context(Some(OrgContext::Editing));
    assert_eq!(
        input.handle(key(KeyCode::Tab)),
        Outcome::Command(Command::Org(OrgCommand::Cycle))
    );
    assert_eq!(
        input.handle(key(KeyCode::BackTab)),
        Outcome::Command(Command::Org(OrgCommand::GlobalCycle))
    );
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT))),
        Outcome::Command(Command::Org(OrgCommand::GlobalCycle))
    );
    assert_eq!(Command::matching("org-").len(), 118);
    for command in OrgCommand::ALL {
        assert_eq!(Command::named(command.name()), Some(Command::Org(command)));
    }
    input.argument("Draft: ", "kept").unwrap();
    assert_eq!(input.handle(key(KeyCode::Tab)), Outcome::Complete);
    assert_eq!(input.prompt().unwrap().text(), "kept");
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    input.set_org_context(Some(OrgContext::Capture));
    assert_eq!(
        input.handle(key(KeyCode::Tab)),
        Outcome::Command(Command::InsertTab)
    );
}

// Clock chords and M-x share compiled identities. The reserved radio checkbox
// chord keeps its existing meaning, and C-g cancels the clock prefix normally.
#[test]
fn clock_keys() {
    for (letter, command) in [
        ('i', OrgCommand::ClockIn),
        ('o', OrgCommand::ClockOut),
        ('q', OrgCommand::ClockCancel),
        ('j', OrgCommand::ClockGoto),
        ('z', OrgCommand::ResolveClocks),
        ('x', OrgCommand::ClockInLast),
        ('d', OrgCommand::ClockDisplay),
        ('c', OrgCommand::ClockReport),
        ('u', OrgCommand::UpdateDynamicBlock),
        ('r', OrgCommand::ToggleRadioButton),
    ] {
        let mut input = Input::default();
        input.set_org_context(Some(OrgContext::Editing));
        input.handle(control('c'));
        input.handle(control('x'));
        assert_eq!(
            input.handle(control(letter)),
            Outcome::Command(Command::Org(command))
        );
    }
    let mut input = Input::default();
    input.set_org_context(Some(OrgContext::Editing));
    input.handle(control('c'));
    input.handle(control('x'));
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
}
