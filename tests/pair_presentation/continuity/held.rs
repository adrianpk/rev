use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use rev::session::Action;
use std::time::{Duration, Instant};

use super::fixture::{Case, Editor};
use super::{advance, now};

// Native repeat/release input over the frozen 1000-line, 96 Down/12 Up workload
// preserves source and makes complete frames. Physical input is left to Slice 6.
#[test]
fn navigation_service() {
    let source: String = (0..1000)
        .map(|row| format!("Line {row:04} (value)\n"))
        .collect();
    for profile in ["non-modal", "modal"] {
        for style in ["depth", "match", "pulse", "off"] {
            navigate(
                Case {
                    language: "plain",
                    profile,
                    style,
                },
                &source,
            );
        }
    }
}

fn navigate(case: Case, source: &str) {
    let mut e = Editor::new(case, source, 10);
    e.settle(|_, _| {});
    let stamp = e.session.buffer().unwrap().stamp();
    let history = e.session.buffer().unwrap().history().len();
    let before = e.observer.frames;
    let started = Instant::now();
    let mut first = None;
    let mut maximum = Duration::ZERO;
    for index in 0..108 {
        let turn = Instant::now();
        let code = if index < 96 {
            KeyCode::Down
        } else {
            KeyCode::Up
        };
        e.session.handle_at(
            Event::Key(KeyEvent::new_with_kind(
                code,
                KeyModifiers::NONE,
                if matches!(index, 0 | 96) {
                    KeyEventKind::Press
                } else {
                    KeyEventKind::Repeat
                },
            )),
            now(),
        );
        let expected = if index < 96 { index + 1 } else { 191 - index };
        e.settle(|frame, cells| {
            assert!(frame.cursor().is_some());
            assert!(cells.values().any(|cell| cell.character == 'L'));
        });
        assert_eq!(
            e.session
                .view()
                .cell(e.session.buffer().unwrap())
                .unwrap()
                .row,
            expected
        );
        assert!(e.session.buffer().unwrap().retained_pair_bytes() <= 2097152);
        first.get_or_insert_with(|| started.elapsed());
        maximum = maximum.max(turn.elapsed());
    }
    assert!(first.unwrap() <= Duration::from_millis(250));
    let release = Instant::now();
    e.session.handle_at(
        Event::Key(KeyEvent::new_with_kind(
            KeyCode::Up,
            KeyModifiers::NONE,
            KeyEventKind::Release,
        )),
        now(),
    );
    advance(1000);
    e.session.expire_input(now());
    e.settle(|_, _| {});
    assert!(release.elapsed() <= Duration::from_millis(250));
    let point = e.session.view().point;
    let viewport = (e.session.view().top, e.session.view().left);
    advance(350);
    assert_eq!(e.session.expire_input(now()), Action::Wait);
    assert_eq!(e.session.remaining_input(now()), None);
    assert_eq!(e.session.view().point, point);
    assert_eq!((e.session.view().top, e.session.view().left), viewport);
    assert_eq!(point.stamp, stamp);
    assert_eq!(e.session.buffer().unwrap().history().len(), history);
    assert_eq!(e.source(), source);
    assert!(e.observer.frames > before);
    eprintln!(
        "CONTINUITY_HELD {case:?} completed_frames={} first_us={} max_native_turn_us={} release_us={}",
        e.observer.frames - before,
        first.unwrap().as_micros(),
        maximum.as_micros(),
        release.elapsed().as_micros()
    );
}
