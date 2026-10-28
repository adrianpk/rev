use super::*;
use rev::{
    command::{Command, OrgCommand as C, Split},
    session::{Action, Session},
};
use std::time::{Duration, Instant};
fn clock() -> Instant {
    static BASE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *BASE.get_or_init(Instant::now)
}
fn session() -> Session {
    Session::empty(Size {
        width: 100,
        height: 12,
    })
    .unwrap()
    .with_timer_clock(clock)
}
fn countdown(session: &mut Session) {
    session.execute(Command::Org(C::TimerSet));
    session.handle(crossterm::event::Event::Paste("0:00:03".into()));
    session.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Enter,
            crossterm::event::KeyModifiers::NONE,
        ),
    ));
}
fn count(session: &mut Session, needle: &str) -> usize {
    session
        .frame()
        .unwrap()
        .rows()
        .iter()
        .filter(|r| r.text().contains(needle))
        .count()
}
// Status is derived only when painting; two panes retain one active indicator.
// Paused/expired state is visible without adding a periodic repaint deadline.
#[test]
fn timer_status() {
    let mut session = session();
    session.execute(Command::Org(C::TimerStart));
    assert_eq!(count(&mut session, "<T+0:00:00>"), 1);
    assert_eq!(session.remaining_input(clock()), None);
    session.execute(Command::Split(Split::Right));
    assert_eq!(count(&mut session, "<T+0:00:00>"), 1);
    session.execute(Command::Org(C::TimerPause));
    assert_eq!(count(&mut session, "paused>"), 1);
    session.execute(Command::Org(C::TimerStop));
    assert_eq!(count(&mut session, "<T"), 0);
    countdown(&mut session);
    assert_eq!(count(&mut session, "<T-0:00:03>"), 1);
    let point = session.view().clone();
    let stamp = session.buffer().unwrap().stamp();
    assert_eq!(
        session.expire_input(clock() + Duration::from_secs(3)),
        Action::Redraw
    );
    assert_eq!(count(&mut session, "<Timer done>"), 1);
    assert_eq!(session.view(), &point);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(
        session.expire_input(clock() + Duration::from_secs(30)),
        Action::Wait
    );
    assert_eq!(session.remaining_input(clock()), None);
}
// Tiny/suspended projections retain valid cells and cursor bounds. Large timer
// labels do not split state tokens or corrupt the underlying Unicode surface.
#[test]
fn timer_geometry() {
    use crossterm::event::Event;
    use unicode_width::UnicodeWidthStr;
    let mut session = session();
    session.handle(Event::Paste("界e\u{301}".into()));
    session.execute(Command::Org(C::TimerStart));
    for (width, height) in [(100, 12), (35, 6), (20, 2), (4, 2), (1, 1), (0, 0)] {
        session.handle(Event::Resize(width, height));
        let frame = session.frame().unwrap();
        for row in frame.rows() {
            assert_eq!(UnicodeWidthStr::width(row.text()), usize::from(width));
        }
        if let Some(cursor) = frame.cursor() {
            assert!(cursor.column < width && cursor.row < height);
        }
    }
    assert_eq!(session.remaining_input(clock()), None);
}
