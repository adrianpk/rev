use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use rev::command::{Command, Motion};
use rev::display::Size;
use rev::input::{Input, META_TIMEOUT, Outcome, Prefix};
use rev::session::{Action, Session};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

// Native pointer reports cannot consume or extend the existing keyboard chord
// or Escape deadline. The following key still resolves exactly once in order.
#[test]
fn keyboard_continuity() {
    let now = Instant::now();
    for kind in [MouseEventKind::ScrollUp, MouseEventKind::Moved] {
        let mut input = Input::default();
        input.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), now);
        input.handle_at(
            Event::Mouse(super::mouse(kind)),
            now + Duration::from_millis(10),
        );
        assert_eq!(input.prefix(), Some(Prefix::Meta));
        assert_eq!(input.remaining(now), Some(META_TIMEOUT));
        assert_eq!(
            input.handle_at(
                key(KeyCode::Char('f'), KeyModifiers::NONE),
                now + Duration::from_millis(20)
            ),
            Outcome::Command(Command::Motion(Motion::ForwardWord))
        );
        assert_eq!(input.remaining(now), None);
        let later = now + Duration::from_millis(30);
        input.handle_at(key(KeyCode::Char('x'), KeyModifiers::CONTROL), later);
        input.handle_at(Event::Mouse(super::mouse(kind)), later);
        assert_eq!(input.prefix(), Some(Prefix::ControlX));
        assert_eq!(
            input.handle_at(key(KeyCode::Char('c'), KeyModifiers::CONTROL), later),
            Outcome::Command(Command::Quit)
        );
    }
}

// The existing Session timer presents expiration once even when a non-key
// event arrives at the boundary. No pointer timer or idle redraw is introduced.
#[test]
fn native_expiry() {
    let now = Instant::now();
    let mut session = Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap();
    let before = (session.buffer().unwrap().stamp(), session.view().point);
    assert_eq!(
        session.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), now),
        Action::Redraw
    );
    let pointer = Event::Mouse(super::mouse(MouseEventKind::Moved));
    assert_eq!(
        session.handle_at(pointer.clone(), now + META_TIMEOUT),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Escape");
    assert_eq!(session.handle_at(pointer, now + META_TIMEOUT), Action::Wait);
    assert_eq!(
        (session.buffer().unwrap().stamp(), session.view().point),
        before
    );
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(
        session.handle_at(
            key(KeyCode::Char('f'), KeyModifiers::NONE),
            now + META_TIMEOUT
        ),
        Action::Redraw
    );
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        "f"
    );
    assert_eq!(buffer.history().len(), 1);
}

// Unsupported motion leaves native editing state intact, and C-g still owns
// cancellation without promoting pointer reports to command or text events.
#[test]
fn cancellation_order() {
    let now = Instant::now();
    let mut input = Input::default();
    input.argument("Path: ", "keep").unwrap();
    input.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), now);
    for _ in 0..128 {
        assert_eq!(
            input.handle_at(Event::Mouse(super::mouse(MouseEventKind::Moved)), now),
            Outcome::Ignored
        );
    }
    assert_eq!(input.remaining(now), Some(META_TIMEOUT));
    assert_eq!(
        input.handle_at(key(KeyCode::Char('g'), KeyModifiers::CONTROL), now),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.remaining(now), None);
    assert!(input.prompt().is_none());
}
