use super::*;
use rev::command::OrgCommand as C;
use rev::input::{Input, OrgContext, Outcome};
// Familiar timer chords and agenda countdown map to the compiled identities;
// prefix cancellation and non-Org editing retain their ordinary key semantics.
#[test]
fn timer_keys() {
    let mut input = Input::default();
    input.set_org_context(Some(OrgContext::Editing));
    for (key, command) in [
        ('0', C::TimerStart),
        (';', C::TimerSet),
        ('.', C::TimerInsert),
        ('-', C::TimerItem),
        (',', C::TimerPause),
        ('_', C::TimerStop),
    ] {
        input.handle(control('c'));
        input.handle(control('x'));
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(
                KeyCode::Char(key),
                KeyModifiers::NONE
            ))),
            Outcome::Command(Command::Org(command))
        );
        assert_eq!(Command::named(command.name()), Some(Command::Org(command)));
    }
    input.set_org_context(Some(OrgContext::Agenda));
    assert_eq!(
        input.handle(Event::Key(KeyEvent::new(
            KeyCode::Char(';'),
            KeyModifiers::NONE
        ))),
        Outcome::Command(Command::Org(C::TimerSet))
    );
    input.set_org_context(Some(OrgContext::Editing));
    input.handle(control('c'));
    input.handle(control('x'));
    assert_eq!(
        input.handle(control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
}
