use super::{key, session, text};
use crate::command::{Command, Motion};
use crate::config::PairHighlight;
use crate::session::events::{INPUT_BATCH_EVENTS, INPUT_BATCH_TIME};
use crate::session::{Action, Session};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[path = "../../../../tests/support/completed.rs"]
mod frames;

// Exercise the real native drain owner with active pair work. The count/time
// boundary leaves queued input intact and every serviced batch draws a frame.
#[test]
fn pair_presentation_continuity() {
    assert_eq!(INPUT_BATCH_EVENTS, 32);
    assert_eq!(INPUT_BATCH_TIME, Duration::from_millis(4));
    let source: String = (0..1000)
        .map(|row| format!("Line {row:04} (value)\n"))
        .collect();
    for style in [
        PairHighlight::Depth,
        PairHighlight::Match,
        PairHighlight::Pulse,
        PairHighlight::Off,
    ] {
        service(style, &source);
    }
}

fn service(style: PairHighlight, source: &str) {
    let mut s = session();
    s.handle(Event::Paste(source.to_owned()));
    s.execute(Command::Motion(Motion::BufferStart));
    for _ in 0..10 {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
    let defaults = s.buffer().unwrap().pair_policy();
    s.windows.selected_mut().pairs.choose(style, defaults);
    completed(&mut s);
    assert_eq!(s.view().point.position.0, 10);
    let stamp = s.buffer().unwrap().stamp();
    let history = s.buffer().unwrap().history().len();
    let mut pending: VecDeque<_> = (0..108)
        .map(|index| {
            Event::Key(KeyEvent::new_with_kind(
                if index < 96 {
                    KeyCode::Down
                } else {
                    KeyCode::Up
                },
                KeyModifiers::NONE,
                if matches!(index, 0 | 96) {
                    KeyEventKind::Press
                } else {
                    KeyEventKind::Repeat
                },
            ))
        })
        .collect();
    let mut frames = 0;
    while let Some(event) = pending.pop_front() {
        let before = pending.len() + 1;
        let started = Instant::now();
        let action = s.handle(event);
        s.drain_input(action, started, || Ok(pending.pop_front()))
            .unwrap();
        assert!(before - pending.len() <= INPUT_BATCH_EVENTS);
        frames += completed(&mut s);
    }
    assert!(frames >= 4);
    time_boundary(&mut s);
    s.handle(Event::Key(KeyEvent::new_with_kind(
        KeyCode::Up,
        KeyModifiers::NONE,
        KeyEventKind::Release,
    )));
    s.expire_input(Instant::now() + Duration::from_secs(1));
    completed(&mut s);
    assert_eq!(s.view().cell(s.buffer().unwrap()).unwrap().row, 84);
    assert_eq!(s.remaining_input(Instant::now()), None);
    let point = s.view().point;
    assert_eq!(
        s.expire_input(Instant::now() + Duration::from_millis(350)),
        Action::Wait
    );
    assert_eq!(s.view().point, point);
    assert_eq!(point.stamp, stamp);
    assert_eq!(s.buffer().unwrap().history().len(), history);
    assert_eq!(text(&s), source);
    eprintln!(
        "CONTINUITY_BATCH style={style:?} inputs=108 complete_frames={frames} event_limit=32 time_target_ms=4"
    );
}

fn completed(s: &mut Session) -> usize {
    let frame = s.frame().unwrap();
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    let mut count = 0;
    frames::completed(std::str::from_utf8(&output).unwrap(), &mut 0, &mut |_| {
        count += 1
    });
    assert_eq!(count, 1);
    count
}

fn time_boundary(s: &mut Session) {
    let mut reads = 0;
    let point = s.view().point;
    let result = s
        .drain_input(Action::Redraw, Instant::now() - INPUT_BATCH_TIME, || {
            reads += 1;
            Ok(Some(key(KeyCode::Down, KeyModifiers::NONE)))
        })
        .unwrap();
    assert_eq!(reads, 0);
    assert_eq!(result, Action::Redraw);
    assert_eq!(s.view().point, point);
}
