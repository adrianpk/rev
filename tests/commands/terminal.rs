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
        let mut input = Input::default();
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
        input.handle_at(super::control('g'), now + Duration::from_millis(10)),
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

// Consumed or rejected suffixes suppress held-key repeats without suppressing
// the next distinct press; neither a Meta nor nested suffix may leak into text.
#[test]
fn suffix_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let now = Instant::now();
    for (prefix, suffix, expected) in [
        (
            vec![super::key(KeyCode::Esc)],
            'f',
            Outcome::Command(Command::Motion(Motion::ForwardWord)),
        ),
        (
            vec![super::key(KeyCode::Esc)],
            'q',
            Outcome::Message("Unbound key"),
        ),
        (
            vec![super::control('x')],
            'q',
            Outcome::Message("Undefined key sequence"),
        ),
        (
            vec![super::control('x'), super::character('r')],
            'w',
            Outcome::Message("Undefined key sequence"),
        ),
    ] {
        let mut input = Input::default();
        for event in prefix {
            input.handle_at(event, now);
        }
        assert_eq!(input.handle_at(super::character(suffix), now), expected);
        for kind in [Release, Repeat] {
            assert_eq!(
                input.handle_at(super::phase(super::character(suffix), kind), now),
                Outcome::Ignored
            );
        }
        assert!(matches!(
            input.handle_at(super::character('z'), now),
            Outcome::Editing(_)
        ));
    }
}

// Escape releases/repeats and resize do not extend its original deadline. C-g
// cancels immediately and leaves no later timer message to affect new input.
#[test]
fn escape_phases() {
    use crossterm::event::KeyEventKind::{Release, Repeat};
    let now = Instant::now();
    let mut input = Input::default();
    input.handle_at(super::key(KeyCode::Esc), now);
    for event in [
        super::phase(super::key(KeyCode::Esc), Repeat),
        super::phase(super::key(KeyCode::Esc), Release),
        Event::Resize(0, 0),
    ] {
        assert_eq!(
            input.handle_at(event, now + Duration::from_millis(10)),
            Outcome::Ignored
        );
    }
    assert_eq!(
        input.remaining(now + Duration::from_millis(10)),
        Some(META_TIMEOUT - Duration::from_millis(10))
    );
    assert_eq!(
        input.handle_at(super::control('g'), now),
        Outcome::Command(Command::KeyboardQuit)
    );
    assert_eq!(input.expire(now + META_TIMEOUT), Outcome::Ignored);
    assert_eq!(input.remaining(now), None);
    input.handle_at(super::key(KeyCode::Esc), now + META_TIMEOUT);
    assert_eq!(
        input.handle_at(super::key(KeyCode::Esc), now + META_TIMEOUT),
        Outcome::Message("Escape")
    );
    assert_eq!(input.remaining(now + META_TIMEOUT), None);
}

// Modified suffixes stay with argument/inventory/decision owners; expiration
// preserves focus, while explicit cancellation restores editing exactly once.
#[test]
fn focused_meta() {
    use rev::input::Focus;
    let now = Instant::now();
    for focus in [Focus::Argument, Focus::Decision, Focus::Inventory] {
        let mut input = Input::default();
        match focus {
            Focus::Argument => input.argument("Path: ", "kept").unwrap(),
            Focus::Decision => input.decision("Overwrite?").unwrap(),
            _ => input.inventory().unwrap(),
        }
        input.handle_at(super::key(KeyCode::Esc), now);
        assert!(matches!(
            input.handle_at(super::character('f'), now),
            Outcome::Message(_)
        ));
        assert_eq!(input.focus(), focus);
        input.handle_at(super::key(KeyCode::Esc), now);
        assert_eq!(input.expire(now + META_TIMEOUT), Outcome::Message("Escape"));
        assert_eq!(input.focus(), focus);
        if let Some(prompt) = input.prompt() {
            assert_eq!(prompt.text(), "kept");
        }
        input.handle_at(super::key(KeyCode::Esc), now + META_TIMEOUT);
        assert_eq!(
            input.handle_at(super::control('g'), now + META_TIMEOUT),
            Outcome::Command(Command::KeyboardQuit)
        );
        assert_eq!(input.focus(), Focus::Editing);
    }
}

// Pending Meta cannot defer or bypass guarded quit. Control chords are routed
// immediately and the buffer's actual dirty state determines the exit result.
#[test]
fn pending_quit() {
    use rev::session::{Action, Session};
    use rev::view::Size;
    let now = Instant::now();
    for dirty in [false, true] {
        let mut session = Session::empty(Size {
            width: 48,
            height: 8,
        })
        .unwrap();
        if dirty {
            session.insert('X');
        }
        session.handle_at(super::key(KeyCode::Esc), now);
        session.handle_at(Event::Resize(0, 0), now);
        assert!(session.frame().unwrap().cursor().is_none());
        session.handle_at(super::control('x'), now);
        assert_eq!(
            session.handle_at(super::control('c'), now),
            if dirty { Action::Redraw } else { Action::Exit }
        );
        assert_eq!(session.buffer().unwrap().len_chars(), usize::from(dirty));
        if dirty {
            assert!(session.echo().contains("Unsaved changes"));
        }
    }
}

// Expiration is still presented when the simultaneous event is an ignored
// release; later prompt typing stays isolated and explicit cancellation works.
#[test]
fn session_expiry() {
    use rev::session::{Action, Session};
    use rev::view::Size;
    let now = Instant::now();
    let mut session = Session::empty(Size {
        width: 48,
        height: 8,
    })
    .unwrap();
    session.execute(Command::FindFile);
    session.handle_at(super::key(KeyCode::Esc), now);
    let release = super::phase(
        super::key(KeyCode::Esc),
        crossterm::event::KeyEventKind::Release,
    );
    assert_eq!(
        session.handle_at(release, now + META_TIMEOUT),
        Action::Redraw
    );
    assert_eq!(session.echo(), "Escape");
    assert_eq!(session.expire_input(now + META_TIMEOUT), Action::Wait);
    session.handle_at(super::character('x'), now + META_TIMEOUT);
    assert_eq!(session.buffer().unwrap().len_chars(), 0);
    session.handle_at(super::control('g'), now + META_TIMEOUT);
    session.handle_at(super::character('a'), now + META_TIMEOUT);
    assert_eq!(session.buffer().unwrap().len_chars(), 1);
}

// Each report reaches real per-buffer undo, recovering the clean empty state
// once for one typing group rather than merely resolving an enum in the decoder.
#[test]
fn reported_undo() {
    use rev::session::Session;
    use rev::view::Size;
    for events in [
        vec![super::control('/')],
        vec![super::control('_')],
        vec![super::control('7')],
        vec![super::control('x'), super::character('u')],
    ] {
        let mut session = Session::empty(Size {
            width: 48,
            height: 8,
        })
        .unwrap();
        session.insert('é');
        session.insert('界');
        for event in events {
            session.handle(event);
        }
        let buffer = session.buffer().unwrap();
        assert_eq!(buffer.len_chars(), 0);
        assert!(!buffer.is_modified());
        assert_eq!(buffer.redo_history().len(), 1);
    }
}
