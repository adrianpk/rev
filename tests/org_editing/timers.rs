use super::*;
use rev::command::{History, Motion, OrgCommand as C};
fn frozen() -> Instant {
    static BASE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *BASE.get_or_init(Instant::now)
}
fn action(session: &mut Session, command: C) {
    if matches!(
        command,
        C::TimerStart | C::TimerSet | C::TimerPause | C::TimerStop | C::TimerShow
    ) {
        session.execute(Command::Org(command));
    } else {
        visibility::action(session, command);
    }
}
// The native source owner preserves CRLF/Unicode and one history unit. M-RET
// recognizes a timed description while non-timer list semantics remain intact.
#[test]
fn timer_notes() {
    let source = "* Heading\r\nbody 界\r\n";
    let (_fixture, session) = editor("", "notes.org", source);
    let mut session = session.with_timer_clock(frozen);
    settle(&mut session);
    headings::goto(&mut session, 2);
    action(&mut session, C::TimerItem);
    let expected = "* Heading\r\n- 0:00:00 :: body 界\r\n";
    assert_eq!(text(&session), expected, "{}", session.echo());
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), expected);
    session.execute(Command::Motion(Motion::LineEnd));
    action(&mut session, C::MetaReturn);
    assert!(text(&session).contains("\r\n- 0:00:00 :: \r\n"));
    action(&mut session, C::TimerPause);
    assert!(session.echo().contains("paused"));
    action(&mut session, C::TimerStop);
    assert_eq!(session.remaining_input(Instant::now()), None);
}
// Rejected source must not auto-start a timer. Countdown drafts/replacement are
// cancellable and invalid input remains visible without changing the old timer.
#[test]
fn timer_admission() {
    for (name, source, row) in [
        ("ordinary list", "- ordinary item\n", 1),
        ("literal", "#+begin_example\nbody\n#+end_example\n", 2),
        ("heading", "* Heading\n", 1),
    ] {
        let (_fixture, mut session) = editor("", "bad.org", source);
        settle(&mut session);
        headings::goto(&mut session, row);
        action(&mut session, C::TimerItem);
        assert_eq!(text(&session), source, "{name}");
        action(&mut session, C::TimerShow);
        assert!(
            session.echo().contains("No Org timer"),
            "{name}: {}",
            session.echo()
        );
    }
    let (_fixture, mut session) = editor("[org.timer]\ndefault='0:00:03'", "count.org", "body");
    settle(&mut session);
    action(&mut session, C::TimerSet);
    properties::argument(&mut session, "bad");
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("bad"))
    );
    session.handle(control('g'));
    action(&mut session, C::TimerShow);
    assert!(session.echo().contains("No Org timer"));
    action(&mut session, C::TimerSet);
    properties::argument(&mut session, "");
    assert!(session.echo().contains("started"));
    action(&mut session, C::TimerSet);
    properties::argument(&mut session, "0:00:07");
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Replace Org"))
    );
    session.handle(control('g'));
    action(&mut session, C::TimerStop);
    assert_eq!(text(&session), "body");
    assert!(session.buffer().unwrap().history().is_empty());
}
