use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, Split};
use rev::input::{Focus, Input, Outcome};
use rev::session::{Action, Session};
use rev::view::Size;

fn open() -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::ALT))
}

fn editor() -> Session {
    Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap()
}

fn named(session: &mut Session, name: &str) -> Action {
    assert_eq!(session.handle(open()), Action::Redraw);
    assert_eq!(session.handle(Event::Paste(name.into())), Action::Redraw);
    session.handle(super::key(KeyCode::Enter))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Ranking is stable and fixed-name only: prefixes precede lexical substrings;
// no expression, case folding or language completion changes the catalog.
#[test]
fn filter_order() {
    let cases = [
        (
            "kill",
            vec![
                "kill-buffer",
                "kill-line",
                "kill-region",
                "kill-ring-save",
                "kill-terminal",
                "kill-word",
                "backward-kill-word",
            ],
        ),
        ("window-right", vec!["split-window-right"]),
        ("WINDOW", vec![]),
        ("undo; newline", vec![]),
    ];
    for (query, expected) in cases {
        let mut input = Input::default();
        input.commands().unwrap();
        input.handle(Event::Paste(query.into()));
        let actual: Vec<_> = input
            .palette()
            .unwrap()
            .candidates()
            .into_iter()
            .map(Command::name)
            .collect();
        assert_eq!(actual, expected, "{query}");
        assert_eq!(input.focus(), Focus::Command);
        assert_eq!(input.label(), Some("M-x "));
    }
}

// Tab completes and cycles the original filtered set, rather than narrowing to
// its first completed name. Return consumes the selected shared command once.
#[test]
fn tab_cycle() {
    let mut input = Input::default();
    assert_eq!(
        input.handle(open()),
        Outcome::Command(Command::ExecuteExtended)
    );
    input.commands().unwrap();
    input.handle(Event::Paste("split-window".into()));
    for expected in [
        "split-window-below",
        "split-window-right",
        "split-window-below",
        "split-window-right",
    ] {
        assert_eq!(input.handle(super::key(KeyCode::Tab)), Outcome::Changed);
        assert_eq!(input.prompt().unwrap().text(), expected);
    }
    assert_eq!(
        input.handle(super::key(KeyCode::Enter)),
        Outcome::Command(Command::Split(Split::Right))
    );
    assert_eq!(input.focus(), Focus::Editing);
    assert!(input.prompt().is_none());
}

// M-x reaches the real edit/history handlers, including redo with no key binding;
// query characters and command acceptance never become document insertions.
#[test]
fn named_history() {
    let mut session = editor();
    named(&mut session, "newline");
    assert_eq!(text(&session), "\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    named(&mut session, "undo");
    assert_eq!(text(&session), "");
    named(&mut session, "undo-redo");
    assert_eq!(text(&session), "\n");
    assert_eq!(session.buffer().unwrap().history().len(), 1);
}
