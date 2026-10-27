#[path = "goto/bounds.rs"]
mod bounds;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::input::{Input, Outcome, Prefix};
use rev::session::Session;
use rev::view::Size;

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn editor(text: &str) -> Session {
    let mut session = Session::empty(Size {
        width: 80,
        height: 10,
    })
    .unwrap();
    session.replace(0..0, text).unwrap();
    session
}

fn submit(session: &mut Session, text: &str) {
    session.handle(Event::Paste(text.into()));
    session.handle(key(KeyCode::Enter, KeyModifiers::NONE));
}

// Both default suffixes share the compiled command. M-g itself retains a prefix
// and C-g cancels it without publishing a goto command or a text edit.
#[test]
fn native_aliases() {
    for (name, suffix) in [("plain", KeyModifiers::NONE), ("meta", KeyModifiers::ALT)] {
        let mut input = Input::default();
        assert_eq!(
            input.handle(key(KeyCode::Char('g'), KeyModifiers::ALT)),
            Outcome::Prefix(Prefix::Sequence),
            "{name}"
        );
        assert_eq!(input.prefix_label().as_deref(), Some("M-g"), "{name}");
        assert_eq!(
            input.handle(key(KeyCode::Char('g'), suffix)),
            Outcome::Command(Command::GotoLine),
            "{name}"
        );
    }
    let mut input = Input::default();
    input.handle(key(KeyCode::Char('g'), KeyModifiers::ALT));
    assert_eq!(
        input.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL)),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(Command::named("goto-line"), Some(Command::GotoLine));
}

// Bindings and M-x reach the actual argument owner and the same logical line;
// argument text stays transient until acceptance and never edits the document.
#[test]
fn prompt_dispatch() {
    for name in ["plain", "meta", "named"] {
        let mut session = editor("界e\u{301}\r\nsecond\r\nlast");
        let point = session.view().point;
        let history = session.buffer().unwrap().history().len();
        if name == "named" {
            session.execute(Command::ExecuteExtended);
            submit(&mut session, "goto-line");
        } else {
            session.handle(key(KeyCode::Char('g'), KeyModifiers::ALT));
            session.handle(key(
                KeyCode::Char('g'),
                if name == "plain" {
                    KeyModifiers::NONE
                } else {
                    KeyModifiers::ALT
                },
            ));
        }
        assert_eq!(session.view().point, point, "{name}");
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|row| row.text().starts_with("Goto line: ")),
            "{name}"
        );
        submit(&mut session, "+2");
        assert_eq!(session.view().point.position.0, 5, "{name}");
        assert_eq!(
            session.view().cell(session.buffer().unwrap()).unwrap().row,
            1,
            "{name}"
        );
        assert_eq!(session.buffer().unwrap().history().len(), history, "{name}");
    }
}
