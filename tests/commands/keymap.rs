use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Binding;
use rev::input::keymap::{Context, Sequence};
use rev::input::{Input, Outcome};

// Configured spellings collapse terminal aliases and Escape Meta pairs exactly
// like runtime keys. Shifted text is represented by the reported literal scalar.
#[test]
fn sequence_aliases() {
    for (left, right) in [
        ("C-I", "TAB"),
        ("C-m", "RET"),
        ("C-j", "Enter"),
        ("C-h", "Backspace"),
        ("C-?", "DEL"),
        ("C-@", "C-SPC"),
        ("NULL", "C-SPC"),
        ("C-/", "C-_"),
        ("C-7", "C-_"),
        ("A-j", "M-j"),
        ("C-M-h", "M-DEL"),
        ("ESC j", "M-j"),
        ("C-[ j", "M-j"),
        ("C-x ESC w", "C-x M-w"),
        ("S-TAB", "BackTab"),
    ] {
        assert_eq!(
            Sequence::parse(left),
            Sequence::parse(right),
            "{left}: {right}"
        );
    }
    assert_ne!(Sequence::parse("a"), Sequence::parse("A"));
    assert_eq!(Sequence::parse("C-x C-s").unwrap().chords().len(), 2);
    assert!(Sequence::parse(&["M-j"; Sequence::MAX_CHORDS].join(" ")).is_ok());
}

// Rejected syntax cannot leave a partially normalized sequence. Prefix length
// is finite, and bare Escape retains its existing timeout/cancel behavior.
#[test]
fn sequence_errors() {
    for text in [
        "",
        " ",
        "C-",
        "C-C-a",
        "M-A-a",
        "S-a",
        "F999",
        "Q-a",
        "ESC",
        "ESC C-a",
        "ESC ESC",
        "<unknown>",
    ] {
        assert!(Sequence::parse(text).is_err(), "{text}");
    }
    assert!(Sequence::parse(&["M-j"; Sequence::MAX_CHORDS + 1].join(" ")).is_err());
}

// The audited editor defaults are exercised through real prefix routing; a
// stale table entry or wrong command identity disagrees with native behavior.
#[test]
fn editor_defaults() {
    for (sequence, name) in Context::Editor.defaults() {
        let mut input = Input::default();
        let mut outcome = Outcome::Ignored;
        for chord in Sequence::parse(sequence).unwrap().chords() {
            outcome = input.handle(Event::Key(KeyEvent::new(chord.code, chord.modifiers)));
        }
        let Binding::Command(command) = Binding::named(name).unwrap() else {
            panic!("prompt action in editor defaults: {name}");
        };
        assert_eq!(outcome, Outcome::Command(command), "{sequence}");
    }
}

// Reported terminal aliases take the same native path as their configured
// spelling; cancellation and Meta handling still use their existing owners.
#[test]
fn runtime_aliases() {
    for (code, modifiers, name) in [
        (KeyCode::Char('i'), KeyModifiers::CONTROL, "insert-tab"),
        (KeyCode::Char('m'), KeyModifiers::CONTROL, "newline"),
        (
            KeyCode::Char('h'),
            KeyModifiers::CONTROL,
            "backward-delete-char",
        ),
        (KeyCode::Null, KeyModifiers::NONE, "set-mark-command"),
        (KeyCode::Char('/'), KeyModifiers::CONTROL, "undo"),
        (
            KeyCode::Char('x'),
            KeyModifiers::META,
            "execute-extended-command",
        ),
    ] {
        let mut input = Input::default();
        let Binding::Command(command) = Binding::named(name).unwrap() else {
            unreachable!()
        };
        assert_eq!(
            input.handle(Event::Key(KeyEvent::new(code, modifiers))),
            Outcome::Command(command)
        );
    }
}
