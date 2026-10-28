use super::*;
use rev::command::{Command, OrgCommand as C};
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
fn arm(session: &mut Session) {
    session.execute(Command::Org(C::TimerSet));
    session.handle(Event::Paste("0:00:03".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
}
// Canceling normal quit retains the countdown and unsaved source. Expiry during
// the save decision preserves the question; completed exit disarms session work.
#[test]
fn timer_quit() {
    let mut session = session();
    arm(&mut session);
    session.handle(Event::Paste("unsaved".into()));
    session.execute(Command::Quit);
    let question = session.echo().to_owned();
    session.expire_input(clock() + Duration::from_secs(3));
    assert_eq!(session.echo(), question);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Timer done"))
    );
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(session.buffer().unwrap().len_chars(), 7);
    arm(&mut session);
    session.execute(Command::Quit);
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(
        session.remaining_input(clock()),
        Some(Duration::from_secs(3))
    );
    session.execute(Command::Quit);
    assert_eq!(
        session.handle(Event::Key(KeyEvent::new(
            KeyCode::Char('n'),
            KeyModifiers::NONE
        ))),
        Action::Exit
    );
    assert_eq!(session.remaining_input(clock()), None);
}
// New sessions never inherit a deadline. A canceled countdown prompt keeps the
// existing timer; direct clean exit cancels its only armed deadline immediately.
#[test]
fn timer_session_exit() {
    let mut session = session();
    arm(&mut session);
    session.execute(Command::Org(C::TimerSet));
    session.handle(Event::Paste("0:00:07".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('g'),
        KeyModifiers::CONTROL,
    )));
    assert_eq!(
        session.remaining_input(clock()),
        Some(Duration::from_secs(3))
    );
    assert_eq!(session.execute(Command::Quit), Action::Exit);
    assert_eq!(session.remaining_input(clock()), None);
    assert_eq!(super::timers::session().remaining_input(clock()), None);
}
