use rev::command::OrgCommand;
use rev::input::OrgContext;

use super::*;

// Familiar Org chords and terminal Meta reports resolve compiled identities.
// Other editing roles inherit their own maps; prefixes and prompts retain cancel.
#[test]
fn heading_keys() {
    for (suffix, command) in [
        ('n', OrgCommand::NextHeading),
        ('p', OrgCommand::PreviousHeading),
        ('f', OrgCommand::ForwardHeading),
        ('b', OrgCommand::BackwardHeading),
        ('u', OrgCommand::ParentHeading),
    ] {
        let mut input = Input::default();
        input.set_org_context(Some(OrgContext::Editing));
        input.handle(control('c'));
        assert_eq!(
            input.handle(control(suffix)),
            Outcome::Command(Command::Org(command))
        );
        assert_eq!(Command::named(command.name()), Some(Command::Org(command)));
    }
    for modifiers in [KeyModifiers::ALT, KeyModifiers::META] {
        let mut input = Input::default();
        input.set_org_context(Some(OrgContext::Editing));
        let event = Event::Key(KeyEvent::new(KeyCode::Enter, modifiers));
        assert_eq!(
            input.handle(event),
            Outcome::Command(Command::Org(OrgCommand::MetaReturn))
        );
    }
    let mut input = Input::default();
    input.set_org_context(Some(OrgContext::Editing));
    input.handle(key(KeyCode::Esc));
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Command(Command::Org(OrgCommand::MetaReturn))
    );
    input.handle(control('c'));
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    for context in [
        None,
        Some(OrgContext::Capture),
        Some(OrgContext::SourceEdit),
    ] {
        input.set_org_context(context);
        assert_ne!(
            input.handle(Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT))),
            Outcome::Command(Command::Org(OrgCommand::MetaReturn))
        );
    }
}
