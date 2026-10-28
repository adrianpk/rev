use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use rev::display::Position;
use rev::session::Action;

use super::autoscroll::{TICK, arm};
use super::scroll::{editor, key, lines};
use super::selection::{drag, range, release};

// Motion updates the current edge without postponing its deadline or creating
// catch-up ticks; a late host wake still advances only one factory-sized step.
#[test]
fn clock_bounds() {
    let mut session = editor(&lines(200));
    let now = Instant::now();
    assert_eq!(session.remaining_input(now), None);
    arm(&mut session, now);
    for index in 0..1000 {
        session.handle_at(
            drag(7 + index % 3, 11),
            now + TICK - Duration::from_millis(1),
        );
    }
    assert_eq!(session.remaining_input(now), Some(TICK));
    assert_eq!(session.expire_input(now + TICK), Action::Redraw);
    assert_eq!(session.view().top, 3);
    assert_eq!(session.remaining_input(now + TICK), Some(TICK));
    let late = now + TICK * 1000;
    assert_eq!(session.expire_input(late), Action::Redraw);
    assert_eq!(session.view().top, 6);
    assert_eq!(session.expire_input(late), Action::Wait);
    assert_eq!(session.remaining_input(late), Some(TICK));
    session.handle_at(drag(7, 1), late);
    assert_eq!(session.remaining_input(late), None);
    session.handle_at(drag(7, 10), late + TICK);
    assert_eq!(session.remaining_input(late + TICK), Some(TICK));
    assert_eq!(session.expire_input(late + TICK), Action::Wait);
}

// The current text-area height, including a one-row pane, bounds each step.
// Document end removes the deadline immediately and preserves canonical bytes.
#[test]
fn row_bounds() {
    for height in [3, 4, 5, 12] {
        let source = lines(8);
        let mut session = editor(&source);
        session.handle(Event::Resize(80, height));
        let rows = height - 2;
        let now = Instant::now();
        session.handle_at(super::focus::press(Position { column: 7, row: 0 }), now);
        session.handle_at(drag(u16::MAX, rows), now);
        assert_eq!(session.expire_input(now + TICK), Action::Redraw);
        assert_eq!(session.view().top, usize::from(rows.min(3)));
        for tick in 2..=10 {
            session.expire_input(now + TICK * tick);
        }
        assert_eq!(session.view().top, 8);
        assert_eq!(session.view().point.position.0, source.len());
        assert_eq!(session.remaining_input(now + TICK * 20), None);
        assert_eq!(session.expire_input(now + TICK * 20), Action::Wait);
        assert!(session.buffer().unwrap().history().is_empty());
        let mut saved = Vec::new();
        rev::file::write(session.buffer().unwrap(), &mut saved).unwrap();
        assert_eq!(saved, source.as_bytes());
    }
}

// Ready release and every native cancellation class win at an already-due
// deadline; no later expiry can perform a deferred step from the old capture.
#[test]
fn input_priority() {
    for event in [
        release(7, 11),
        key(KeyCode::Char('g'), KeyModifiers::CONTROL),
        key(KeyCode::Esc, KeyModifiers::NONE),
        key(KeyCode::Char('x'), KeyModifiers::CONTROL),
        Event::FocusLost,
        Event::Resize(80, 12),
        Event::Paste(String::new()),
        super::focus::press(Position { column: 0, row: 0 }),
    ] {
        let mut session = editor(&lines(40));
        let now = Instant::now();
        arm(&mut session, now);
        session.handle_at(event.clone(), now + TICK * 100);
        session.expire_input(now + TICK * 200);
        assert_eq!(session.view().top, 0, "{event:?}");
        assert!(!session.cancel_pointer_capture(), "{event:?}");
        assert_eq!(session.remaining_input(now + TICK * 200), None);
        assert_eq!(range(&session), None);
    }
}

// BOF/EOF, including the empty document, cannot leave an idle polling deadline.
#[test]
fn terminal_rows() {
    for text in ["", "a", "a\n"] {
        let mut session = editor(text);
        let now = Instant::now();
        session.handle_at(super::focus::press(Position { column: 6, row: 0 }), now);
        session.handle_at(drag(6, u16::MAX), now);
        session.expire_input(now + TICK);
        assert_eq!(session.remaining_input(now + TICK), None);
        let before = session.view().clone();
        assert_eq!(session.expire_input(now + TICK * 10), Action::Wait);
        assert_eq!(session.view(), &before);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}
