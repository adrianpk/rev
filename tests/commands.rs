#![forbid(unsafe_code)]

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::input::{Focus, Input, Key, Outcome, Prefix};

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn character(value: char) -> Event {
    key(KeyCode::Char(value))
}

fn control(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::CONTROL))
}

fn feed(input: &mut Input, events: Vec<Event>) -> Vec<Outcome> {
    events
        .into_iter()
        .map(|event| input.handle(event))
        .collect()
}

// A complete binding produces the same implemented enum as its canonical name;
// a nested prefix consumes its next key instead of dispatching an absent command.
#[test]
fn command_sequences() {
    let cases = [
        (
            "quit",
            vec![control('x'), control('c')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::Quit),
            ],
        ),
        (
            "cancel",
            vec![control('x'), control('g')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Command(Command::KeyboardQuit),
            ],
        ),
        (
            "nested",
            vec![control('x'), character('r'), character('x')],
            vec![
                Outcome::Prefix(Prefix::ControlX),
                Outcome::Prefix(Prefix::Rectangle),
                Outcome::Message("Undefined key sequence"),
            ],
        ),
    ];
    for (name, events, expected) in cases {
        let mut input = Input::default();
        assert_eq!(feed(&mut input, events), expected, "{name}");
        assert_eq!(input.prefix(), None, "{name}");
    }
    for command in Command::ALL {
        assert_eq!(Command::named(command.name()), Some(command));
        assert!(
            command
                .name()
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        );
    }
    assert_eq!(Command::named("find-file"), None);
    assert_eq!(Command::named(""), None);
}

// Argument input belongs to the prompt until completion; Enter returns its
// value exactly once and restores editing without forwarding that Enter.
#[test]
fn argument_focus() {
    let mut input = Input::default();
    input.argument("Path: ", "é界").unwrap();
    assert_eq!(input.focus(), Focus::Argument);
    assert_eq!(input.label(), Some("Path: "));
    assert_eq!(input.handle(key(KeyCode::Left)), Outcome::Ignored);
    assert_eq!(input.handle(character('x')), Outcome::Ignored);
    assert_eq!(input.prompt().unwrap().text(), "éx界");
    assert_eq!(input.prompt().unwrap().cursor(), "éx".len());
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Argument("éx界".into())
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert!(input.prompt().is_none());
    assert_eq!(
        input.handle(character('z')),
        Outcome::Editing(Key {
            code: KeyCode::Char('z'),
            modifiers: KeyModifiers::NONE
        })
    );
}

// A decision has no implicit affirmative default. Invalid input retains focus,
// and only an explicit answer produces a result and closes the decision.
#[test]
fn explicit_decisions() {
    for (name, answer, expected) in [
        ("yes", 'y', true),
        ("no", 'n', false),
        ("upper yes", 'Y', true),
        ("upper no", 'N', false),
    ] {
        let mut input = Input::default();
        input.decision("Overwrite? y/n").unwrap();
        assert_eq!(input.label(), Some("Overwrite? y/n"));
        assert_eq!(
            input.handle(key(KeyCode::Enter)),
            Outcome::Message("Answer y or n"),
            "{name}"
        );
        assert_eq!(input.focus(), Focus::Decision);
        assert_eq!(
            input.handle(character(answer)),
            Outcome::Decision(expected),
            "{name}"
        );
        assert_eq!(input.focus(), Focus::Editing);
    }
}
