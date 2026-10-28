use super::headings::{goto, row};
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::{Duration, UNIX_EPOCH};

pub(super) fn clock(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709303400))
    })
}
fn later(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    ZonedOperation::capture(&ZoneSelection::Utc, &ZoneEnvironment::system(), || {
        Ok(UNIX_EPOCH + Duration::from_secs(1709308860))
    })
}
// Clocking belongs to the current section, not a child or the point at out.
// Each operation is a normal history record and goto reveals the actual owner.
#[test]
fn owned_interval() {
    let source = "* Parent\nbody 界\n** Child\nchild\n* Peer\nend";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(clock);
    goto(&mut session, 2);
    action(&mut session, C::ClockIn);
    let opened = "* Parent\n:LOGBOOK:\nCLOCK: [2024-03-01 Fri 14:30]\n:END:\nbody 界\n** Child\nchild\n* Peer\nend";
    assert_eq!(text(&session), opened, "{}", session.echo());
    action(&mut session, C::Overview);
    action(&mut session, C::NextHeading);
    action(&mut session, C::ClockGoto);
    assert_eq!(row(&session), 0);
    assert_eq!(text(&session), opened);
    session = session.with_org_clock(later);
    action(&mut session, C::ClockOut);
    let closed = opened.replace(
        "[2024-03-01 Fri 14:30]",
        "[2024-03-01 Fri 14:30]--[2024-03-01 Fri 16:01] =>  1:31",
    );
    assert_eq!(text(&session), closed, "{}", session.echo());
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), opened);
    session.execute(Command::History(History::Redo));
    assert_eq!(text(&session), closed);
    session.execute(Command::History(History::Undo));
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), source);
}
// Local/inherited drawer ownership preserves planning, properties and old logs;
// duplicate drawers, incomplete source clocks and literal point reject safely.
#[test]
fn clock_placement() {
    for (name, source, wanted) in [
        (
            "metadata",
            "* Task\nSCHEDULED: <2024-03-02 Sat>\n:PROPERTIES:\n:CLOCK_INTO_DRAWER: WORK\n:END:\n:WORK:\nold log\n:END:\nbody",
            ":WORK:\nCLOCK: [2024-03-01 Fri 14:30]\nold log",
        ),
        (
            "inherited",
            "* Parent\n:PROPERTIES:\n:LOG_INTO_DRAWER: WORK\n:END:\n** Task\nbody",
            "** Task\n:WORK:\nCLOCK:",
        ),
        (
            "plain",
            "* Task\n:PROPERTIES:\n:CLOCK_INTO_DRAWER: nil\n:END:\nbody",
            ":END:\nCLOCK: [2024-03-01 Fri 14:30]\nbody",
        ),
        ("EOF", "* Task", "* Task\n:LOGBOOK:\nCLOCK:"),
    ] {
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        if name == "inherited" {
            goto(&mut session, 6);
        }
        action(&mut session, C::ClockIn);
        assert!(
            text(&session).contains(wanted),
            "{name}: {} / {}",
            text(&session),
            session.echo()
        );
    }
    for source in [
        "* Task\nCLOCK: [2024-03-01 Fri 14:30]\n",
        "* Task\n:LOGBOOK:\na\n:END:\n:LOGBOOK:\nb\n:END:\n",
    ] {
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Same-minute records are retained by default. Explicit removal and cancel
// remove only their interval and an otherwise empty drawer in one transaction.
#[test]
fn zero_and_cancel() {
    for (name, config, source) in [
        ("default", "", "* Task\nbody"),
        ("remove", "org.clock.remove_zero=true", "* Task\nbody"),
        ("EOF", "org.clock.remove_zero=true", "* Task"),
        (
            "empty drawer",
            "org.clock.remove_zero=true",
            "* Task\n:LOGBOOK:\n:END:\nbody",
        ),
        (
            "other log",
            "org.clock.remove_zero=true",
            "* Task\n:LOGBOOK:\nold log\n:END:\nbody",
        ),
    ] {
        let (_fixture, session) = editor(config, "clock.org", source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        let opened = text(&session);
        action(&mut session, C::ClockOut);
        let expected = if name == "default" {
            opened.replace(
                "[2024-03-01 Fri 14:30]",
                "[2024-03-01 Fri 14:30]--[2024-03-01 Fri 14:30] =>  0:00",
            )
        } else if name == "empty drawer" {
            "* Task\nbody".into()
        } else {
            source.into()
        };
        assert_eq!(text(&session), expected, "{name}: {}", session.echo());
        assert_eq!(session.buffer().unwrap().history().len(), 2);
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), opened, "{name}");
    }
    cancel_cases();
}
fn cancel_cases() {
    for (source, expected) in [
        ("* Task", "* Task"),
        ("* Task\nbody", "* Task\nbody"),
        (
            "* Task\n:LOGBOOK:\nold log\n:END:\n",
            "* Task\n:LOGBOOK:\nold log\n:END:\n",
        ),
    ] {
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        let opened = text(&session);
        action(&mut session, C::ClockCancel);
        assert_eq!(text(&session), expected, "{}", session.echo());
        session.execute(Command::History(History::Undo));
        assert_eq!(text(&session), opened);
    }
}

// Configured placement is live. Source properties retain their explicit
// precedence and nil placement does not manufacture a drawer.
#[test]
fn clock_policy() {
    for (config, source, wanted) in [
        ("org.clock.drawer='WORK'", "* Task\nbody", ":WORK:\nCLOCK:"),
        ("org.clock.drawer=''", "* Task\nbody", "* Task\nCLOCK:"),
        (
            "org.clock.drawer='WORK'",
            "* Task\n:PROPERTIES:\n:LOG_INTO_DRAWER: LOG\n:CLOCK_INTO_DRAWER: CLOCKS\n:END:\nbody",
            ":CLOCKS:\nCLOCK:",
        ),
    ] {
        let (_fixture, session) = editor(config, "clock.org", source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ClockIn);
        assert!(
            text(&session).contains(wanted),
            "{}: {}",
            text(&session),
            session.echo()
        );
        action(&mut session, C::ClockCancel);
        assert_eq!(text(&session), source);
    }
}
