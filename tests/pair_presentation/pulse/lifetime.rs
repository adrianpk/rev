use super::{Editor, advance, now};
use crate::surface;
use rev::session::Action;
use std::time::Duration;

// Expiry is exact at every admitted duration boundary. Steady redraw has no
// trigger, each endpoint uses the match palette, and expiry leaves no deadline.
#[test]
fn duration_and_expiry() {
    for duration in [50, 250, 1000] {
        let mut e = Editor::new(duration, "(x)", 0);
        assert!(e.sources().is_empty());
        assert_eq!(e.remaining(), None);
        e.jump();
        let frame = e.frame();
        let cells = surface::draw(&frame);
        for column in [0, 2] {
            assert!(cells[&(0, column)].underline);
            assert_eq!(
                cells[&(0, column)].foreground,
                Some(e.native.palette.pairs.match_text)
            );
            assert_eq!(
                cells[&(0, column)].background,
                Some(e.native.palette.pairs.match_background)
            );
        }
        assert_eq!(
            e.remaining(),
            Some(Duration::from_millis(u64::try_from(duration).unwrap()))
        );
        advance(duration - 1);
        assert_eq!(e.sources(), [0, 2]);
        assert_eq!(e.remaining(), Some(Duration::from_millis(1)));
        advance(1);
        assert_eq!(e.remaining(), Some(Duration::ZERO));
        assert_eq!(e.native.session.expire_input(now()), Action::Redraw);
        assert!(e.sources().is_empty());
        assert_eq!(e.remaining(), None);
        advance(1000);
        assert_eq!(e.native.session.expire_input(now()), Action::Wait);
        e.native.unchanged("(x)", 2);
    }
}

// Repaint and resize reproject the same input generation, including suspended
// geometry. Restoring a surface cannot restart an expired generation.
#[test]
fn repaint_and_resize() {
    let mut e = Editor::new(250, "(x)", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    advance(100);
    for (width, height) in [(70, 14), (31, 6), (0, 0), (80, 15)] {
        e.native
            .session
            .handle(crossterm::event::Event::Resize(width, height));
        e.frame();
        assert_eq!(e.remaining(), Some(Duration::from_millis(150)));
    }
    advance(150);
    assert!(e.sources().is_empty());
    e.native
        .session
        .handle(crossterm::event::Event::Resize(70, 14));
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    e.native.unchanged("(x)", 2);
}

// A new identified pair replaces the old expiry. The old due time must neither
// clear the replacement nor extend it to another complete duration.
#[test]
fn replacement() {
    use rev::command::{Command, Motion};
    let mut e = Editor::new(250, "() []", 0);
    e.jump();
    assert_eq!(e.sources(), [0, 1]);
    advance(20);
    e.native.session.execute(Command::Motion(Motion::BufferEnd));
    assert_eq!(e.sources(), [3, 4]);
    assert_eq!(e.remaining(), Some(Duration::from_millis(250)));
    advance(230);
    assert_eq!(e.native.session.expire_input(now()), Action::Wait);
    assert_eq!(e.sources(), [3, 4]);
    assert_eq!(e.remaining(), Some(Duration::from_millis(20)));
    advance(20);
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), None);
    e.native.unchanged("() []", 5);
}

// Timer ownership composes through the existing minimum-deadline owner. Pair
// expiry cannot cancel the independent Org countdown or change its due time.
#[test]
fn independent_deadline() {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
    use rev::command::{Command, OrgCommand};
    let mut e = Editor::new(250, "(x)", 0);
    e.native.session = e.native.session.with_timer_clock(now);
    e.native.session.execute(Command::Org(OrgCommand::TimerSet));
    e.native.session.handle(Event::Paste("0:00:03".into()));
    e.native.session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    e.jump();
    assert_eq!(e.sources(), [0, 2]);
    assert_eq!(e.remaining(), Some(Duration::from_millis(250)));
    advance(250);
    assert!(e.sources().is_empty());
    assert_eq!(e.remaining(), Some(Duration::from_millis(2750)));
}
