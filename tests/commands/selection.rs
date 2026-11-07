use crossterm::event::{KeyEvent, KeyModifiers};
use rev::command::{KillCommand, Split};

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

// Return accepts the default highlight with an empty query. Explicit navigation
// overrides an exact draft, while a freely typed exact name keeps its identity.
#[test]
fn acceptance_rules() {
    for (name, text, events, expected) in [
        (
            "empty",
            "",
            vec![],
            Outcome::Command(Command::AiApplyResult),
        ),
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
            Outcome::Command(Command::AiApplyResult),
        ),
        (
            "completed empty",
            "",
            vec![key(KeyCode::Tab)],
            Outcome::Command(Command::AiApplyResult),
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
        assert_eq!(input.palette().unwrap().candidates().len(), 11);
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
        Outcome::Command(Command::Newline)
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
        Outcome::Command(Command::Newline)
    );
}

// The quit spelling completes through both ordinary selectors, reuses effective
// action hints and descriptions, and can be assigned wherever exit is allowed.
#[test]
fn quit_alias_completion() {
    let alias = Command::named("quit").expect("quit alias");
    assert_eq!(alias.name(), "quit");
    assert_eq!(alias.description(), Command::Quit.description());
    for name in ["q", "qa", "qa!", "quit!"] {
        assert_eq!(Command::named(name), None, "{name}");
    }
    for (profile, binding, hint) in [
        ("non-modal", "", "C-x C-c"),
        ("modal", "", "C-x C-c"),
        ("non-modal", "[keys.editor]\n'M-j'='quit'", "M-j"),
        ("modal", "[keys.vim_normal]\n'M-j'='quit'", "M-j"),
    ] {
        let fixture = super::config_fixture::Fixture::new();
        let startup = rev::config::prepare(&fixture.write(
            "config.toml",
            &format!("editor.interaction_profile='{profile}'\n{binding}"),
        ));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut input = Input::with_settings(&startup.snapshot);
        for prefix in ["q", "qu", "qui"] {
            input.commands().unwrap();
            input.handle(Event::Paste(prefix.into()));
            let palette = input.palette().unwrap();
            assert_eq!(palette.hint(alias), Some(hint), "{profile}: {binding}");
            let position = palette
                .candidates()
                .iter()
                .position(|c| *c == alias)
                .unwrap();
            for _ in 0..position {
                input.handle(key(KeyCode::Down));
            }
            input.handle(key(KeyCode::Tab));
            assert_eq!(input.prompt().unwrap().text(), "quit", "{prefix}");
            assert_eq!(input.handle(key(KeyCode::Enter)), Outcome::Command(alias));
        }
        input.handle(control('x'));
        assert_eq!(input.handle(control('c')), Outcome::Command(Command::Quit));
        if !binding.is_empty() {
            assert_eq!(input.handle(meta('j')), Outcome::Command(Command::Quit));
        }
    }
}
