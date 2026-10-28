use std::time::Instant;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::command::{Command, Motion, Scroll};
use rev::config::{Snapshot, prepare};
use rev::input::{Focus, Input, OrgContext, Outcome, Prefix};

fn settings(source: &str) -> Snapshot {
    let fixture = super::config_fixture::Fixture::new();
    let startup = prepare(&fixture.write("config.toml", source));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    startup.snapshot
}

fn meta(character: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::ALT))
}

// Every role resolves its own alias to an existing compiled operation. Editor
// mappings remain independent; prompt mappings have precedence over each role.
#[test]
fn role_routing() {
    let snapshot = settings(
        "[keys.editor]\n'M-j' = 'newline'\n[keys.org]\n'M-j' = 'undo'\n[keys.org_agenda]\n'M-j' = 'next-line'\n[keys.org_capture]\n'M-j' = 'backward-char'\n[keys.org_calendar]\n'M-j' = 'previous-line'\n[keys.org_source_edit]\n'M-j' = 'forward-char'\n[keys.minibuffer]\n'M-j' = 'minibuffer-accept'",
    );
    let cases = [
        (
            OrgContext::Editing,
            Command::History(rev::buffer::edit::History::Undo),
        ),
        (OrgContext::Agenda, Command::Scroll(Scroll::NextLine)),
        (OrgContext::Capture, Command::Motion(Motion::BackwardChar)),
        (OrgContext::Calendar, Command::Scroll(Scroll::PreviousLine)),
        (OrgContext::SourceEdit, Command::Motion(Motion::ForwardChar)),
    ];
    for (context, command) in cases {
        let mut input = Input::with_settings(&snapshot);
        input.set_org_context(Some(context));
        assert_eq!(
            input.handle(meta('j')),
            Outcome::Command(command),
            "{context:?}"
        );
        input.argument("Draft: ", "kept").unwrap();
        input.handle(Event::Paste("界".into()));
        assert_eq!(
            input.handle(meta('j')),
            Outcome::Argument("kept界".into()),
            "{context:?}"
        );
        assert_eq!(input.org_context(), Some(context));
        input.set_org_context(None);
        assert_eq!(input.handle(meta('j')), Outcome::Command(Command::Newline));
    }
}

// Changing workspace ownership retires both prefix suffixes and Escape timing.
// C-g cancels a prompt without discarding the retained workspace role or draft
// authority into a later fresh editor key.
#[test]
fn role_transitions() {
    let snapshot =
        settings("[keys.org]\n'C-c a' = 'newline'\n[keys.org_capture]\n'C-c a' = 'backward-char'");
    let mut input = Input::with_settings(&snapshot);
    input.set_org_context(Some(OrgContext::Editing));
    assert_eq!(
        input.handle(super::control('c')),
        Outcome::Prefix(Prefix::Sequence)
    );
    input.set_org_context(Some(OrgContext::Capture));
    assert_eq!(input.prefix(), None);
    assert!(matches!(
        input.handle(super::character('a')),
        Outcome::Editing(_)
    ));
    input.handle(super::key(KeyCode::Esc));
    assert!(input.remaining(Instant::now()).is_some());
    input.set_org_context(Some(OrgContext::SourceEdit));
    assert_eq!(input.remaining(Instant::now()), None);
    let repeated = Event::Key(KeyEvent {
        kind: KeyEventKind::Repeat,
        ..KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)
    });
    assert_eq!(input.handle(repeated), Outcome::Ignored);
    input.argument("Draft: ", "kept").unwrap();
    input.set_org_context(Some(OrgContext::Editing));
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(input.prompt().unwrap().text(), "kept");
    assert_eq!(
        input.handle(super::control('g')),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.org_context(), Some(OrgContext::Editing));
    assert!(matches!(
        input.handle(super::character('a')),
        Outcome::Editing(_)
    ));
}

// Non-text roles consume unbound text/paste while their configured navigation
// remains operational. A transient argument can still accept literal text.
#[test]
fn nontext_roles() {
    let snapshot = settings(
        "[keys.org_agenda]\n'M-j' = 'next-line'\n[keys.org_calendar]\n'M-j' = 'next-line'",
    );
    for context in [OrgContext::Agenda, OrgContext::Calendar] {
        let mut input = Input::with_settings(&snapshot);
        input.set_org_context(Some(context));
        assert!(matches!(
            input.handle(super::character('x')),
            Outcome::Message(_)
        ));
        assert!(matches!(
            input.handle(Event::Paste("blocked".into())),
            Outcome::Message(_)
        ));
        assert_eq!(
            input.handle(meta('j')),
            Outcome::Command(Command::Scroll(Scroll::NextLine))
        );
        input.argument("Date: ", "").unwrap();
        input.handle(Event::Paste("2026-10-28".into()));
        assert_eq!(
            input.handle(super::key(KeyCode::Enter)),
            Outcome::Argument("2026-10-28".into())
        );
        assert!(matches!(
            input.handle(super::character('x')),
            Outcome::Message(_)
        ));
    }
}
