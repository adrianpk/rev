use crossterm::event::{KeyEvent, KeyModifiers};
use rev::command::{KillCommand, Motion, Split};

use super::{Command, Event, Focus, Input, KeyCode, Outcome, control, key};

fn meta(value: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(value), KeyModifiers::ALT))
}

fn query(text: &str) -> Input {
    let mut input = Input::default();
    input.commands().unwrap();
    input.handle(Event::Paste(text.into()));
    input
}

// Both native navigation pairs select candidates without rewriting the draft.
// Endpoints clamp; repeated completion fills the same selected identity.
#[test]
fn candidate_keys() {
    for (name, previous, next) in [
        ("arrows", key(KeyCode::Up), key(KeyCode::Down)),
        ("control", control('p'), control('n')),
    ] {
        let mut input = query("split-window");
        for _ in 0..3 {
            assert_eq!(input.handle(next.clone()), Outcome::Changed, "{name}");
        }
        assert_eq!(input.palette().unwrap().selected(), Some(1), "{name}");
        assert_eq!(input.prompt().unwrap().text(), "split-window", "{name}");
        for _ in 0..3 {
            input.handle(previous.clone());
        }
        assert_eq!(input.palette().unwrap().selected(), Some(0), "{name}");
        input.handle(next);
        input.handle(key(KeyCode::Tab));
        input.handle(key(KeyCode::Tab));
        assert_eq!(input.prompt().unwrap().text(), "split-window-right");
        assert_eq!(input.palette().unwrap().candidates().len(), 2);
        assert_eq!(
            input.handle(key(KeyCode::Enter)),
            Outcome::Command(Command::Split(Split::Right)),
            "{name}"
        );
    }
}

// A default highlight alone cannot accept an empty query. Explicit navigation
// overrides an exact draft, while a freely typed exact name keeps its identity.
#[test]
fn acceptance_rules() {
    for (name, text, events, expected) in [
        ("empty", "", vec![], Outcome::Message("No command entered")),
        (
            "exact",
            "kill-word",
            vec![],
            Outcome::Command(Command::Kill(KillCommand::Word)),
        ),
        (
            "exact then selected",
            "kill-word",
            vec![key(KeyCode::Down)],
            Outcome::Command(Command::Kill(KillCommand::BackwardWord)),
        ),
        (
            "partial",
            "split-window",
            vec![],
            Outcome::Command(Command::Split(Split::Below)),
        ),
        (
            "selected",
            "split-window",
            vec![key(KeyCode::Down)],
            Outcome::Command(Command::Split(Split::Right)),
        ),
        (
            "completed then selected",
            "split-window",
            vec![key(KeyCode::Tab), key(KeyCode::Down)],
            Outcome::Command(Command::Split(Split::Right)),
        ),
        (
            "explicit empty",
            "",
            vec![key(KeyCode::Up)],
            Outcome::Command(Command::Motion(Motion::BackwardChar)),
        ),
        (
            "completed empty",
            "",
            vec![key(KeyCode::Tab)],
            Outcome::Command(Command::Motion(Motion::BackwardChar)),
        ),
        (
            "no match",
            "界absent",
            vec![key(KeyCode::Down)],
            Outcome::Message("Unknown command"),
        ),
    ] {
        let mut input = query(text);
        for event in events {
            input.handle(event);
        }
        assert_eq!(input.handle(key(KeyCode::Enter)), expected, "{name}");
        assert_eq!(input.focus(), Focus::Editing, "{name}");
    }
}

// Cursor-only motion, empty paste, refused controls and no-op deletion do not
// discard selection or the original completion filter. A real edit does.
#[test]
fn query_changes() {
    let mut input = query("kill");
    input.handle(control('n'));
    input.handle(key(KeyCode::Tab));
    for event in [
        key(KeyCode::Left),
        key(KeyCode::End),
        key(KeyCode::Delete),
        Event::Paste(String::new()),
        Event::Paste("\n".into()),
    ] {
        input.handle(event);
        assert_eq!(input.palette().unwrap().selected(), Some(1));
        assert_eq!(input.palette().unwrap().candidates().len(), 10);
        assert_eq!(input.prompt().unwrap().text(), "kill-line");
    }
    input.handle(key(KeyCode::Backspace));
    assert_eq!(input.palette().unwrap().selected(), Some(0));
    assert_eq!(input.palette().unwrap().top(), 0);
    assert_eq!(
        input.palette().unwrap().candidates(),
        [Command::Kill(KillCommand::Line)]
    );
    input.handle(Event::Paste("界".into()));
    assert_eq!(input.palette().unwrap().selected(), None);
    assert_eq!(
        input.handle(key(KeyCode::Up)),
        Outcome::Message("No matching command")
    );
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Message("Unknown command")
    );
}

// History has its own Meta route. Candidate movement neither recalls accepted
// names nor damages the draft restored by M-n after a recall.
#[test]
fn separate_history() {
    let mut input = query("newline");
    input.handle(key(KeyCode::Enter));
    input.commands().unwrap();
    input.handle(Event::Paste("split-window".into()));
    input.handle(control('n'));
    assert_eq!(input.prompt().unwrap().text(), "split-window");
    input.handle(meta('p'));
    assert_eq!(input.prompt().unwrap().text(), "newline");
    input.handle(control('p'));
    input.handle(meta('n'));
    assert_eq!(input.prompt().unwrap().text(), "split-window");
    assert_eq!(input.palette().unwrap().selected(), Some(0));
    input.handle(control('g'));
    input.commands().unwrap();
    input.handle(meta('p'));
    assert_eq!(input.prompt().unwrap().text(), "newline");
    input.handle(meta('n'));
    assert_eq!(input.prompt().unwrap().text(), "");
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Message("No command entered")
    );
}

// Compatible Escape delivery reaches the same history actions as direct Meta
// without inserting p/n into the query or stealing candidate navigation keys.
#[test]
fn escape_history() {
    let mut input = query("newline");
    input.handle(key(KeyCode::Enter));
    input.commands().unwrap();
    let now = std::time::Instant::now();
    for (suffix, expected) in [('p', "newline"), ('n', "")] {
        input.handle_at(key(KeyCode::Esc), now);
        input.handle_at(super::character(suffix), now);
        assert_eq!(input.prompt().unwrap().text(), expected);
    }
    assert_eq!(
        input.handle(key(KeyCode::Enter)),
        Outcome::Message("No command entered")
    );
}
