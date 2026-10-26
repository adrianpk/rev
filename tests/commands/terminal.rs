use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::{Command, History, KillCommand, LinearCommand, Motion};
use rev::input::{Input, Key, META_TIMEOUT, Outcome, Prefix};

// Alt/Meta reports and separately delivered Escape prefixes share the exact
// implemented bindings, preserving Shift's already reported < and > characters.
#[test]
fn compatible_meta() {
    let now = Instant::now();
    for (character, motion) in [
        ('<', Motion::BufferStart),
        ('>', Motion::BufferEnd),
        ('f', Motion::ForwardWord),
    ] {
        for modifiers in [
            KeyModifiers::ALT,
            KeyModifiers::META,
            KeyModifiers::ALT | KeyModifiers::SHIFT,
        ] {
            let mut input = Input::default();
            assert_eq!(
                input.handle_at(
                    Event::Key(KeyEvent::new(KeyCode::Char(character), modifiers)),
                    now
                ),
                Outcome::Command(Command::Motion(motion))
            );
        }
        let mut input = Input::default();
        assert_eq!(
            input.handle_at(super::key(KeyCode::Esc), now),
            Outcome::Prefix(Prefix::Meta)
        );
        assert_eq!(input.prefix(), Some(Prefix::Meta));
        assert_eq!(
            input.handle_at(
                super::character(character),
                now + META_TIMEOUT - Duration::from_nanos(1)
            ),
            Outcome::Command(Command::Motion(motion))
        );
        assert_eq!(input.remaining(now), None);
    }
}

// The boundary is finite and exact: expiry is one-shot, and a fresh key at or
// after it is ordinary input rather than a lost event or a stale Meta suffix.
#[test]
fn escape_deadline() {
    let now = Instant::now();
    let mut input = Input::default();
    input.handle_at(super::key(KeyCode::Esc), now);
    assert_eq!(input.remaining(now), Some(META_TIMEOUT));
    assert_eq!(
        input.expire(now + META_TIMEOUT - Duration::from_nanos(1)),
        Outcome::Ignored
    );
    assert_eq!(input.expire(now + META_TIMEOUT), Outcome::Message("Escape"));
    assert_eq!(input.expire(now + META_TIMEOUT), Outcome::Ignored);
    assert_eq!(input.prefix(), None);
    for late in [META_TIMEOUT, META_TIMEOUT + Duration::from_secs(1)] {
        input.handle_at(super::key(KeyCode::Esc), now);
        assert_eq!(
            input.handle_at(super::character('f'), now + late),
            Outcome::Editing(Key {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::NONE
            })
        );
        assert_eq!(input.remaining(now + late), None);
    }
}

// A staged Meta suffix belongs to its existing nested keymap. It cannot escape
// a rectangle prefix to invoke linear copy, and C-g/guarded quit remain available.
#[test]
fn nested_meta() {
    let now = Instant::now();
    let mut input = Input::default();
    input.handle_at(super::control('x'), now);
    input.handle_at(super::character('r'), now);
    input.handle_at(super::key(KeyCode::Esc), now);
    assert_eq!(
        input.handle_at(super::character('w'), now),
        Outcome::Message("Undefined key sequence")
    );
    assert_eq!(input.prefix(), None);
    input.handle_at(super::control('x'), now);
    input.handle_at(super::character('r'), now);
    input.handle_at(super::key(KeyCode::Esc), now);
    assert_eq!(
        input.handle_at(super::control('g'), now),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.prefix(), None);
    input.handle_at(super::key(KeyCode::Esc), now);
    assert_eq!(
        input.handle_at(super::control('x'), now),
        Outcome::Prefix(Prefix::ControlX)
    );
    assert_eq!(
        input.handle_at(super::control('c'), now),
        Outcome::Command(Command::Quit)
    );
}

// Legacy 0x1f's C-7 report and distinguishable punctuation reports converge on
// undo; the existing C-x u sequence remains the terminal-independent alternative.
#[test]
fn undo_reports() {
    for character in ['/', '_', '7'] {
        let mut input = Input::default();
        assert_eq!(
            input.handle(super::control(character)),
            Outcome::Command(Command::History(History::Undo))
        );
    }
    let mut input = Input::default();
    input.handle(super::control('x'));
    assert_eq!(
        input.handle(super::character('u')),
        Outcome::Command(Command::History(History::Undo))
    );
    assert_eq!(
        input.handle(super::character('7')),
        Outcome::Editing(Key {
            code: KeyCode::Char('7'),
            modifiers: KeyModifiers::NONE
        })
    );
}

// Staging Escape is only a modifier. It must not invalidate the current yank
// before its y suffix can reach the same real handler as an Alt-y event.
#[test]
fn meta_yank() {
    use rev::session::{Action, Session};
    use rev::view::Size;
    let now = Instant::now();
    let mut session = Session::empty(Size {
        width: 48,
        height: 8,
    })
    .unwrap();
    session.replace(0..0, "aa bb").unwrap();
    session.execute(Command::Motion(Motion::BufferStart));
    session.execute(Command::Kill(KillCommand::Word));
    session.execute(Command::KeyboardQuit);
    session.execute(Command::Kill(KillCommand::Word));
    session.execute(Command::Linear(LinearCommand::Yank));
    assert_eq!(
        session.handle_at(super::key(KeyCode::Esc), now),
        Action::Redraw
    );
    assert_eq!(
        session.handle_at(super::character('y'), now),
        Action::Redraw
    );
    assert_eq!(session.echo(), "");
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        "aa"
    );
}
