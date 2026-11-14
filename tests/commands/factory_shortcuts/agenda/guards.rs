use super::*;
use rev::input::{Focus, Input, OrgContext, Outcome, Prefix};

fn event(chord: &str) -> Event {
    let sequence = Sequence::parse(chord).unwrap();
    let chord = sequence.chords()[0];
    Event::Key(KeyEvent::new(chord.code, chord.modifiers))
}

// A pending Agenda family belongs to that role, never a later prompt or role.
// Even configured prompt accepts cannot turn its suffixes into decision consent.
#[test]
fn prompt_ownership() {
    for modal in [false, true] {
        let snapshot = settings(&format!(
            "{}\n[keys.minibuffer]\n'M'='minibuffer-accept'\n'/'='minibuffer-accept'\n'A'='minibuffer-accept'\n'G'='minibuffer-accept'\n'S'='minibuffer-accept'\n'd'='minibuffer-accept'\n",
            profile(modal.then_some(ModalCommand::Normal))
        ));
        let mut input = Input::with_settings(&snapshot);
        input.set_org_context(Some(OrgContext::Agenda));
        assert_eq!(input.handle(event("v")), Outcome::Prefix(Prefix::Sequence));
        input.argument("Query: ", "retained").unwrap();
        assert_eq!(input.prefix(), None);
        assert_eq!(input.handle(event("v")), Outcome::Changed);
        assert_eq!(input.prompt().unwrap().text(), "retainedv");
        input.cancel();
        for suffix in ["M", "/", "A", "G", "S", "d"] {
            input.handle(event("v"));
            input.decision("Authorize source action? y/n").unwrap();
            assert_eq!(input.prefix(), None);
            assert_eq!(
                input.handle(event(suffix)),
                Outcome::Message("Answer y or n")
            );
            assert_eq!(input.focus(), Focus::Decision);
            assert_eq!(
                input.handle(event("C-g")),
                Outcome::Command(Command::KeyboardQuit)
            );
        }
        assert_eq!(input.handle(event("v")), Outcome::Prefix(Prefix::Sequence));
        input.set_org_context(Some(OrgContext::Editing));
        assert_eq!(input.prefix(), None);
        assert!(!matches!(
            input.handle(event("S")),
            Outcome::Command(Command::Org(_))
        ));
    }
}
