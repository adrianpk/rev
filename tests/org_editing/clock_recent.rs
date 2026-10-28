use super::clocks::clock;
use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{BufferCommand, OrgCommand as C};

// Recent goto and re-clock retain checked task ownership. A fresh re-clock
// inserts a new interval; the earlier closed source remains exact.
#[test]
fn clock_recent_actions() {
    let source = "* Task\nbody\n* Peer\ntext";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(clock);
    action(&mut session, C::ClockIn);
    action(&mut session, C::ClockOut);
    let closed = text(&session);
    goto(&mut session, 6);
    action(&mut session, C::ClockGoto);
    assert_eq!(super::headings::row(&session), 0);
    action(&mut session, C::ClockInLast);
    assert_eq!(text(&session).matches("CLOCK:").count(), 2);
    action(&mut session, C::ClockCancel);
    assert_eq!(text(&session), closed);
    session.handle(Event::Paste("changed".into()));
    let changed = text(&session);
    action(&mut session, C::ClockInLast);
    assert!(session.echo().contains("source changed"));
    assert_eq!(text(&session), changed);
}

// Zero disables history. With two slots, closing the newest buffer reveals the
// prior live task; one slot evicts it instead of keeping unbounded locators.
#[test]
fn clock_history_limit() {
    for limit in [0, 1, 2] {
        let config = format!("org.clock.history_limit={limit}");
        let (fixture, session) = editor(&config, "first.org", "* First\nbody");
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        action(&mut session, C::ClockOut);
        let second = fixture.write("second.org", "* Second\nbody");
        session.load(&second).unwrap();
        action(&mut session, C::ClockIn);
        action(&mut session, C::ClockOut);
        session.execute(Command::Buffer(BufferCommand::Kill));
        argument(&mut session, "");
        session.handle(key(KeyCode::Char('n')));
        action(&mut session, C::ClockGoto);
        if limit == 2 {
            assert!(
                session.echo().contains("recent clock"),
                "{}",
                session.echo()
            );
            assert!(text(&session).starts_with("* First\n"));
        } else {
            assert!(
                session.echo().contains("No active or recent"),
                "{}",
                session.echo()
            );
        }
    }
}
