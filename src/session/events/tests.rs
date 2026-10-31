use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::command::{Command, Motion};
use crate::view::Size;

fn session() -> Session {
    Session::empty(Size {
        width: 80,
        height: 24,
    })
    .unwrap()
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn text(session: &Session) -> String {
    let buffer = session.buffer().unwrap();
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

// Cold reflow yields to native input between finite batches. Source edits and
// C-g remain admitted; pending surfaces expose no stale pointer destination.
#[test]
fn cold_reflow() {
    use crate::buffer::Buffers;
    use crate::coordinates::Scalar;
    use crate::view::View;
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

    let source = "one two a\t界e\u{301} ".repeat(6000);
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get(id).unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(buffer.len_chars());
    let original = view.point.position;
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 6,
        },
    )
    .unwrap();
    assert!(session.poll_frame().unwrap().cursor().is_none());
    assert_eq!(
        session.remaining_input(Instant::now()),
        Some(Duration::ZERO)
    );
    assert_eq!(
        session.handle(Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 1,
            row: 1,
            modifiers: KeyModifiers::NONE,
        })),
        Action::Wait
    );
    assert_eq!(session.view().point.position, original);
    assert_eq!(
        session.handle(key(KeyCode::Char('x'), KeyModifiers::NONE)),
        Action::Redraw
    );
    assert_eq!(text(&session), format!("{source}x"));
    assert_eq!(
        session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL)),
        Action::Redraw
    );
    let point = session.view().point;
    for _ in 0..100 {
        if !session.layout_pending {
            break;
        }
        let action = session.expire_input(Instant::now());
        assert_eq!(
            action,
            if session.layout_pending {
                Action::Wait
            } else {
                Action::Redraw
            }
        );
    }
    assert!(!session.layout_pending);
    assert!(session.poll_frame().unwrap().cursor().is_some());
    let buffer = session.buffer().unwrap();
    let width = session
        .view()
        .text_width(buffer, session.pane_size())
        .unwrap();
    assert_eq!(
        session.view().wrapped_rows(buffer, width, 4).unwrap(),
        session
            .view()
            .clone_for_split()
            .wrapped_rows(buffer, width, 4)
            .unwrap(),
    );
    assert_eq!(session.view().point, point);
    assert_eq!(text(&session), format!("{source}x"));
    assert_eq!(session.remaining_input(Instant::now()), None);
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
    session.handle(Event::Resize(47, 6));
    assert!(session.poll_frame().unwrap().cursor().is_none());
    for _ in 0..100 {
        if !session.layout_pending {
            break;
        }
        session.expire_input(Instant::now());
    }
    assert!(!session.layout_pending);
    assert!(session.poll_frame().unwrap().cursor().is_some());
    assert_eq!(session.view().point, point);
    assert_eq!(text(&session), format!("{source}x"));
    assert_eq!(session.remaining_input(Instant::now()), None);
}

// A busy source cannot starve drawing. All events left at the count boundary
// remain available to the next batch, with no change to text or history.
#[test]
fn event_limit() {
    let mut session = session();
    let original = "row\n".repeat(300);
    session.handle(Event::Paste(original.clone()));
    session.execute(Command::Motion(Motion::BufferStart));
    let stamp = session.buffer().unwrap().stamp();
    let history = session.buffer().unwrap().history().len();
    let mut pending = VecDeque::from(vec![key(KeyCode::Down, KeyModifiers::NONE); 200]);
    while let Some(event) = pending.pop_front() {
        let before = pending.len() + 1;
        let started = Instant::now();
        let action = session.handle(event);
        assert_eq!(
            session
                .drain_input(action, started, || Ok(pending.pop_front()))
                .unwrap(),
            Action::Redraw
        );
        assert!(before - pending.len() <= INPUT_BATCH_EVENTS);
        let frame = session.frame().unwrap();
        assert!(frame.cursor().is_some());
    }
    assert_eq!(
        session.view().cell(session.buffer().unwrap()).unwrap().row,
        200
    );
    assert_eq!(text(&session), original);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().history().len(), history);
}

// Budget expiry and successful exit both leave the next event untouched. An
// ignored event must also retain any redraw already requested in this batch.
#[test]
fn batch_boundaries() {
    for (name, action, elapsed) in [
        ("time", Action::Redraw, INPUT_BATCH_TIME),
        ("exit", Action::Exit, Duration::ZERO),
    ] {
        let mut session = session();
        let mut reads = 0;
        let outcome = session
            .drain_input(action, Instant::now() - elapsed, || {
                reads += 1;
                Ok(Some(key(KeyCode::Char('X'), KeyModifiers::NONE)))
            })
            .unwrap();
        assert_eq!(outcome, action, "{name}");
        assert_eq!(reads, 0, "{name}");
        assert!(text(&session).is_empty(), "{name}");
    }
    let mut session = session();
    let mut pending = VecDeque::from([Event::FocusGained]);
    assert_eq!(
        session
            .drain_input(Action::Redraw, Instant::now(), || Ok(pending.pop_front()))
            .unwrap(),
        Action::Redraw
    );
}

// Batched handlers must match drawing after each event, including reversal,
// cancellation, resize and undo. Intermediate frames are the only omitted work.
#[test]
fn ordered_edit_history() {
    let mut reference = session();
    let mut batched = session();
    let original = "abc\n".repeat(120);
    let mut events = vec![
        Event::Paste(original.clone()),
        key(KeyCode::Char('<'), KeyModifiers::ALT),
    ];
    events.extend(vec![key(KeyCode::Down, KeyModifiers::NONE); 80]);
    events.extend(vec![key(KeyCode::Up, KeyModifiers::NONE); 30]);
    events.extend([
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('r'), KeyModifiers::NONE),
        key(KeyCode::Char('g'), KeyModifiers::CONTROL),
        key(KeyCode::Char('X'), KeyModifiers::NONE),
        Event::Resize(100, 30),
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('u'), KeyModifiers::NONE),
        Event::FocusGained,
    ]);
    for event in events.clone() {
        reference.handle(event);
        reference.frame().unwrap();
    }
    let mut pending = VecDeque::from(events);
    while let Some(event) = pending.pop_front() {
        let started = Instant::now();
        let action = batched.handle(event);
        batched
            .drain_input(action, started, || Ok(pending.pop_front()))
            .unwrap();
        batched.frame().unwrap();
    }
    assert_eq!(text(&batched), original);
    assert_eq!(text(&batched), text(&reference));
    assert_eq!(
        batched.view().point.position,
        reference.view().point.position
    );
    assert_eq!(batched.view().top, reference.view().top);
    assert_eq!(batched.view().left, reference.view().left);
    assert_eq!(batched.view().column, reference.view().column);
    assert_eq!(
        batched.frame().unwrap().cursor(),
        reference.frame().unwrap().cursor()
    );
    assert_eq!(batched.echo(), reference.echo());
    let buffer = batched.buffer().unwrap();
    assert_eq!(buffer.history().len(), 1);
    assert_eq!(buffer.redo_history().len(), 1);
    assert_eq!(
        buffer.stamp().revision,
        reference.buffer().unwrap().stamp().revision
    );
}

// Quit is handled at its original ordered position. No later queued edit can
// run once the native handler accepts exit.
#[test]
fn exit_stops_reading() {
    let mut session = session();
    let mut pending = VecDeque::from([
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        key(KeyCode::Char('c'), KeyModifiers::CONTROL),
        key(KeyCode::Char('X'), KeyModifiers::NONE),
    ]);
    assert_eq!(
        session
            .drain_input(Action::Wait, Instant::now(), || Ok(pending.pop_front()))
            .unwrap(),
        Action::Exit
    );
    assert_eq!(pending.len(), 1);
    assert!(text(&session).is_empty());
}

// Each event still checks the Escape deadline. A late plain key is inserted;
// a timely suffix retains Meta routing rather than entering the document.
#[test]
fn escape_in_batch() {
    for (name, age, expected) in [
        ("timely", Duration::ZERO, ""),
        ("expired", crate::input::META_TIMEOUT, "f"),
    ] {
        let mut session = session();
        let action = session.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), Instant::now() - age);
        let mut pending = VecDeque::from([key(KeyCode::Char('f'), KeyModifiers::NONE)]);
        session
            .drain_input(action, Instant::now(), || Ok(pending.pop_front()))
            .unwrap();
        assert_eq!(text(&session), expected, "{name}");
        assert_eq!(session.input.remaining(Instant::now()), None, "{name}");
    }
}

// Native waiting is indefinite when no deadline exists. A click and an inside
// drag must retain that behavior instead of introducing periodic mouse polling.
#[test]
fn no_idle_poll() {
    let mut session = session();
    session.handle(Event::Paste("a\nb\n".to_owned()));
    session.execute(Command::Motion(Motion::BufferStart));
    for kind in [
        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left),
    ] {
        session.handle(mouse(kind, 0));
        assert!(
            session
                .input_ready(Instant::now(), |_| panic!("idle polling"))
                .unwrap()
        );
    }
}

fn mouse(kind: crossterm::event::MouseEventKind, row: u16) -> Event {
    Event::Mouse(crossterm::event::MouseEvent {
        kind,
        column: 6,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

// A release arriving at the timeout boundary is read before servicing the due
// tick. The same injected native poll boundary also covers ordinary ready input.
#[test]
fn timeout_release() {
    use crossterm::event::{MouseButton, MouseEventKind};
    for (name, answers) in [
        ("ready", vec![true]),
        ("boundary", vec![false, true]),
        ("timeout", vec![false, false]),
    ] {
        let mut session = session();
        session.handle(Event::Paste("row\n".repeat(100)));
        session.execute(Command::Motion(Motion::BufferStart));
        let now = Instant::now();
        session.handle_at(mouse(MouseEventKind::Down(MouseButton::Left), 0), now);
        session.handle_at(mouse(MouseEventKind::Drag(MouseButton::Left), 23), now);
        let due = now + Duration::from_millis(100);
        let mut answers = VecDeque::from(answers);
        let ready = session
            .input_ready(due, |wait| {
                assert_eq!(wait, Duration::ZERO);
                Ok(answers.pop_front().unwrap())
            })
            .unwrap();
        assert!(answers.is_empty(), "{name}");
        if ready {
            session.handle_at(mouse(MouseEventKind::Up(MouseButton::Left), 23), due);
        }
        session.expire_input(due);
        assert_eq!(session.view().top, if ready { 0 } else { 3 }, "{name}");
    }
}

// Queued cancellation is read before a due countdown, including the timeout
// boundary. A delayed, uncancelled wake produces one notice and no new deadline.
#[test]
fn countdown_input_priority() {
    use crate::org::time::TimerDuration;
    for (name, answers) in [
        ("ready", vec![true]),
        ("boundary", vec![false, true]),
        ("due", vec![false, false]),
    ] {
        let mut session = session();
        let now = Instant::now();
        let due = now + Duration::from_secs(1);
        let id = session.view().point.stamp.buffer;
        session
            .buffers
            .get_mut(id)
            .unwrap()
            .select_mode(crate::buffer::Mode::Org)
            .unwrap();
        session
            .timer
            .countdown(now, TimerDuration::parse("0:00:01").unwrap())
            .unwrap();
        let mut answers = VecDeque::from(answers);
        let ready = session
            .input_ready(due, |wait| {
                assert_eq!(wait, Duration::ZERO);
                Ok(answers.pop_front().unwrap())
            })
            .unwrap();
        if ready {
            for event in [
                key(KeyCode::Char('c'), KeyModifiers::CONTROL),
                key(KeyCode::Char('x'), KeyModifiers::CONTROL),
                key(KeyCode::Char('_'), KeyModifiers::NONE),
            ] {
                session.handle_at(event, due);
            }
        }
        let outcome = session.expire_input(due + Duration::from_secs(10));
        assert_eq!(
            outcome,
            if ready { Action::Wait } else { Action::Redraw },
            "{name}"
        );
        assert_eq!(session.timer_notice, !ready, "{name}");
        assert_eq!(session.remaining_input(due), None, "{name}");
        assert_eq!(
            session.expire_input(due + Duration::from_secs(20)),
            Action::Wait,
            "{name}"
        );
    }
}
// Running relatives, paused/stopped countdowns and idle sessions never poll the
// native event source. Only a selected active countdown supplies a wait duration.
#[test]
fn timer_idle_wait() {
    use crate::org::time::TimerDuration;
    let mut session = session();
    let now = Instant::now();
    session.timer.start(now).unwrap();
    assert!(
        session
            .input_ready(now, |_| panic!("relative idle poll"))
            .unwrap()
    );
    session.timer.stop().unwrap();
    session
        .timer
        .countdown(now, TimerDuration::parse("0:00:03").unwrap())
        .unwrap();
    assert_eq!(session.remaining_input(now), Some(Duration::from_secs(3)));
    session
        .timer
        .pause_or_continue(now + Duration::from_secs(1))
        .unwrap();
    assert!(
        session
            .input_ready(now, |_| panic!("paused idle poll"))
            .unwrap()
    );
    session.timer.stop().unwrap();
    assert!(
        session
            .input_ready(now, |_| panic!("stopped idle poll"))
            .unwrap()
    );
}
