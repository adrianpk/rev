use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::buffer::Buffers;
use crate::command::{History, Scroll};
use crate::coordinates::Scalar;
use crate::gutter::Change;

use super::*;

fn editor(source: &str, position: usize) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let b = buffers.get(id).unwrap();
    let mut view = View::new(b);
    view.gutter.apply(Change::Visible(false));
    view.point.position = Scalar(position);
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 6,
        },
    )
    .unwrap()
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new(code, modifiers))
}

fn text(session: &Session) -> String {
    let b = session.buffer().unwrap();
    b.slice(&b.span(0..b.len_chars()).unwrap())
        .unwrap()
        .to_string()
}

fn settle(session: &mut Session) {
    session.poll_frame().unwrap();
    for _ in 0..512 {
        if session.remaining_input(Instant::now()).is_none() {
            return;
        }
        if session.expire_input(Instant::now()) == Action::Redraw {
            session.poll_frame().unwrap();
        }
    }
    panic!("native layout/input did not settle");
}

fn resized(source: &str) -> Session {
    let mut session = editor(source, 65500);
    session.frame().unwrap();
    session.handle(Event::Resize(47, 6));
    session
}

// Commands wait before their effects. Every later edit, deletion and history
// boundary must match the same native sequence with an already warm mapping.
#[test]
fn pending_edits() {
    let source = format!("{}\nxyz", "a".repeat(65536));
    for command in [
        Command::Scroll(Scroll::PreviousLine),
        Command::Motion(Motion::LineEnd),
        Command::Kill(KillCommand::Line),
    ] {
        let mut actual = resized(&source);
        let mut expected = resized(&source);
        expected.frame().unwrap();
        let origin = actual.view().point;
        actual.execute(command);
        expected.execute(command);
        assert!(actual.layout.held.is_some());
        assert_eq!(actual.view().point, origin);
        assert_eq!(text(&actual), source);
        for event in [
            key(KeyCode::Char('A'), KeyModifiers::NONE),
            key(KeyCode::Char('B'), KeyModifiers::NONE),
            key(KeyCode::Backspace, KeyModifiers::NONE),
        ] {
            actual.handle(event.clone());
            expected.handle(event);
        }
        assert_eq!(text(&actual), source);
        settle(&mut actual);
        settle(&mut expected);
        assert_eq!(text(&actual), text(&expected), "{command:?}");
        assert_eq!(actual.view().point.position, expected.view().point.position);
        assert_eq!(
            actual.buffer().unwrap().history().len(),
            expected.buffer().unwrap().history().len()
        );
        assert_eq!(
            actual.killed_text().map(|s| s.to_string()),
            expected.killed_text().map(|s| s.to_string())
        );
        actual.execute(Command::History(History::Undo));
        expected.execute(Command::History(History::Undo));
        settle(&mut actual);
        settle(&mut expected);
        assert_eq!(text(&actual), text(&expected));
        assert_eq!(actual.remaining_input(Instant::now()), None);
    }
}

// C-g cancels unfinished layout operations without throwing away earlier typed
// input. Queued chord state is processed before the interrupt clears that state.
#[test]
fn cancelled_operation() {
    let source = format!("{}\nxyz", "a".repeat(65536));
    let mut session = resized(&source);
    let origin = session.view().point.position;
    session.execute(Command::Kill(KillCommand::Line));
    assert!(session.layout.held.is_some());
    session.handle(key(KeyCode::Char('Q'), KeyModifiers::NONE));
    session.handle(key(KeyCode::Char('x'), KeyModifiers::CONTROL));
    session.handle(key(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(session.layout.held.is_none());
    settle(&mut session);
    assert_eq!(
        text(&session),
        format!("{}Q{}", &source[..65500], &source[65500..])
    );
    assert_eq!(session.view().point.position, Scalar(origin.0 + 1));
    assert!(session.killed_text().is_none());
    assert_eq!(session.input.prefix(), None);
    assert_eq!(session.input.focus(), Focus::Editing);
    assert_eq!(
        session.expire_input(Instant::now() + Duration::from_secs(10)),
        Action::Wait
    );
}

// Native backpressure stops the event reader at the owned FIFO cap. Events
// outside that cap remain at the source and eventually publish exactly once.
#[test]
fn queue_capacity() {
    let source = format!("{}\nxyz", "a".repeat(65536));
    let mut session = resized(&source);
    let mut expected = resized(&source);
    expected.frame().unwrap();
    expected.execute(Command::Motion(Motion::LineEnd));
    let insertion = expected.view().point.position.0;
    session.execute(Command::Motion(Motion::LineEnd));
    assert!(session.layout.held.is_some());
    let mut pending = VecDeque::from(vec![key(KeyCode::Char('x'), KeyModifiers::NONE); 300]);
    while !session.layout_full() {
        let event = pending.pop_front().unwrap();
        let started = Instant::now();
        let action = session.handle(event);
        session
            .drain_input(action, started, || Ok(pending.pop_front()))
            .unwrap();
    }
    assert_eq!(session.layout.input.len(), INPUT_LIMIT);
    let before = pending.len();
    session
        .drain_input(Action::Wait, Instant::now(), || Ok(pending.pop_front()))
        .unwrap();
    assert_eq!(pending.len(), before);
    settle(&mut session);
    while let Some(event) = pending.pop_front() {
        let started = Instant::now();
        let action = session.handle(event);
        session
            .drain_input(action, started, || Ok(pending.pop_front()))
            .unwrap();
    }
    settle(&mut session);
    assert_eq!(
        text(&session),
        format!(
            "{}{}{}",
            &source[..insertion],
            "x".repeat(300),
            &source[insertion..]
        )
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    settle(&mut session);
    assert_eq!(text(&session), source);
}

// A far point is admitted independently of its reflow. Reversal must abandon
// the old derived request and stop with the exact original source/history.
#[test]
fn far_reversal() {
    let source = "a".repeat(65536);
    let mut session = editor(&source, 0);
    session.poll_frame().unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(session.view().point.position, Scalar(source.len()));
    assert!(session.poll_frame().unwrap().cursor().is_none());
    session.execute(Command::Motion(Motion::BufferStart));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(0));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Canonical left/right motion uses observed boundaries, including a long RI
// run whose context must not be rescanned synchronously on every key.
#[test]
fn cold_grapheme() {
    let source = "🇦🇧".repeat(10000);
    let end = source.chars().count();
    let mut session = editor(&source, end);
    session.execute(Command::Motion(Motion::BackwardChar));
    assert!(session.layout.held.is_some());
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(end - 2));
    session.execute(Command::Motion(Motion::ForwardChar));
    settle(&mut session);
    assert_eq!(session.view().point.position, Scalar(end));
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}

// A new size recomputes an unfinished command at its same source point. A
// source publication invalidates it before any deferred destructive effect.
#[test]
fn changed_observation() {
    let source = format!("{}\nxyz", "a".repeat(65536));
    let mut resized = resized(&source);
    resized.execute(Command::Motion(Motion::LineEnd));
    assert!(resized.layout.held.is_some());
    resized.handle(Event::Resize(51, 6));
    let mut reference = editor(&source, 65500);
    reference.handle(Event::Resize(51, 6));
    reference.frame().unwrap();
    reference.execute(Command::Motion(Motion::LineEnd));
    settle(&mut resized);
    settle(&mut reference);
    assert_eq!(
        resized.view().point.position,
        reference.view().point.position
    );
    assert_eq!(text(&resized), source);

    let mut changed = editor(&source, 65500);
    changed.execute(Command::Kill(KillCommand::Line));
    assert!(changed.layout.held.is_some());
    changed.replace(0..1, "Q").unwrap();
    let current = changed.view().point;
    settle(&mut changed);
    assert_eq!(changed.view().point, current);
    assert_eq!(text(&changed), format!("Q{}", &source[1..]));
    assert!(changed.killed_text().is_none());
    assert_eq!(changed.buffer().unwrap().history().len(), 1);
}

// A pre-existing Escape chord keeps the arrival time of input queued behind
// a cold command, even if host expiry/resize happens after its real deadline.
#[test]
fn retained_meta_time() {
    let source = format!("{}\nxyz", "a".repeat(65536));
    let mut session = resized(&source);
    let now = Instant::now();
    session.handle_at(key(KeyCode::Esc, KeyModifiers::NONE), now);
    session.execute(Command::Motion(Motion::LineEnd));
    assert!(session.layout.held.is_some());
    session.handle_at(
        key(KeyCode::Char('f'), KeyModifiers::NONE),
        now + Duration::from_millis(1),
    );
    session.handle_at(Event::Resize(51, 6), now + Duration::from_secs(1));
    session.expire_input(now + Duration::from_secs(1));
    settle(&mut session);
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.input.prefix(), None);
}
