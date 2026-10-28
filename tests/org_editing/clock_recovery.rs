use super::clocks::clock;
use super::headings::goto;
use super::properties::argument;
use super::visibility::action;
use super::*;
use rev::command::{History, OrgCommand as C};
use rev::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::{Duration, UNIX_EPOCH};

fn resolve(session: &mut Session, number: &str, operation: &str) {
    action(session, C::ResolveClocks);
    argument(session, number);
    argument(session, operation);
}
// Resume observes a new qualified source after undo rather than guessing that
// an old revision is equivalent. Original timestamp spelling survives closing.
#[test]
fn clock_recovery_resume() {
    let source = "* Task\n:LOGBOOK:\n  CLOCK: [2024-03-01 raw 14:00]\n:END:\nbody";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(clock);
    resolve(&mut session, "1", "resume");
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
    action(&mut session, C::ClockOut);
    let closed = source.replace("raw 14:00]", "raw 14:00]--[2024-03-01 Fri 14:30] =>  0:30");
    assert_eq!(text(&session), closed, "{}", session.echo());
    session.execute(Command::History(History::Undo));
    goto(&mut session, 1);
    resolve(&mut session, "1", "resume");
    action(&mut session, C::ClockOut);
    assert_eq!(text(&session), closed);
}

// Explicit end selection excludes rejected idle time. Invalid/future input
// remains a correctable draft without a source/history change; C-g preserves it.
#[test]
fn clock_recovery_end() {
    let source = "* Task\nCLOCK: [2024-03-01 Fri 10:00]\nbody";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(clock);
    resolve(&mut session, "1", "close");
    argument(&mut session, "bad date");
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Org clock end: bad date"))
    );
    assert_eq!(text(&session), source);
    session.handle(control('g'));
    resolve(&mut session, "1", "close");
    argument(&mut session, "2024-03-01 12:15");
    assert_eq!(
        text(&session),
        source.replace("10:00]", "10:00]--[2024-03-01 Fri 12:15] =>  2:15")
    );
    session.execute(Command::History(History::Undo));
    resolve(&mut session, "1", "close");
    argument(&mut session, "2024-03-02 12:00");
    assert!(session.echo().contains("after"));
    assert_eq!(text(&session), source);
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("Org clock end: 2024-03-02 12:00"))
    );
}

fn autumn(_: &rev::config::Time) -> Result<ZonedOperation, String> {
    let env = ZoneEnvironment {
        localtime: "unused".into(),
        zoneinfo: "unused".into(),
        tz: Some("CET-1CEST,M3.5.0,M10.5.0/3".into()),
    };
    ZonedOperation::capture(&ZoneSelection::Local, &env, || {
        Ok(UNIX_EPOCH + Duration::from_secs(1729996200))
    })
}
// Both repeated start offsets remain visible choices, and selecting close in
// that branch still requests its end. The selected instant determines elapsed.
#[test]
fn clock_recovery_offsets() {
    for (choice, duration) in [("1", "1:45"), ("2", "0:45")] {
        let source = "* Task\nCLOCK: [2024-10-27 Sun 02:15]\n";
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(autumn);
        resolve(&mut session, "1", "close");
        assert!(
            session.echo().contains("UTC+7200") && session.echo().contains("UTC+3600"),
            "{}",
            session.echo()
        );
        argument(&mut session, choice);
        assert!(
            session
                .frame()
                .unwrap()
                .rows()
                .iter()
                .any(|r| r.text().contains("Org clock end: "))
        );
        argument(&mut session, "2024-10-27 03:00");
        assert_eq!(
            text(&session),
            format!(
                "* Task\nCLOCK: [2024-10-27 Sun 02:15]--[2024-10-27 Sun 03:00] => {duration:>5}\n"
            ),
            "{}",
            session.echo()
        );
    }
}

// Keep is inert source; cancel removes only its observed interval/empty drawer.
// New source during a prompt invalidates its authority and cannot receive edits.
#[test]
fn clock_recovery_boundaries() {
    for operation in ["keep", "cancel"] {
        let source = "* Task\n:LOGBOOK:\nCLOCK: [2024-03-01 Fri 10:00]\n:END:\nbody";
        let (_fixture, session) = editor("", "clock.org", source);
        let mut session = session.with_org_clock(clock);
        resolve(&mut session, "1", operation);
        assert_eq!(
            text(&session),
            if operation == "keep" {
                source
            } else {
                "* Task\nbody"
            }
        );
    }
    let source = "* Task\nCLOCK: [2024-03-01 Fri 10:00]\n";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(clock);
    resolve(&mut session, "1", "close");
    session.replace(0..0, "changed").unwrap();
    argument(&mut session, "2024-03-01 12:00");
    assert!(session.echo().contains("source or view changed"));
    assert_eq!(text(&session), format!("changed{source}"));
}

// Explicit recovery admits at most 64 unfinished source intervals in the entry.
// Exceeding the bound cannot adopt an arbitrary interval or change canonical text.
#[test]
fn clock_recovery_limit() {
    for (count, allowed) in [(64, true), (65, false)] {
        let source = format!(
            "* Task\n{}",
            "CLOCK: [2024-03-01 Fri 10:00]\n".repeat(count)
        );
        let (_fixture, session) = editor("", "clock.org", &source);
        let mut session = session.with_org_clock(clock);
        action(&mut session, C::ResolveClocks);
        assert_eq!(
            session.echo().contains("interval(s)"),
            allowed,
            "{}",
            session.echo()
        );
        if !allowed {
            assert!(session.echo().contains("64 incomplete"));
        }
        assert_eq!(text(&session), source);
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// A spring gap never becomes an invented start or end instant. Source remains
// exact and an entered invalid end stays visible for correction or cancellation.
#[test]
fn clock_recovery_missing() {
    let source = "* Task\nCLOCK: [2024-03-31 Sun 02:30]\nbody";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(autumn);
    resolve(&mut session, "1", "resume");
    assert!(session.echo().contains("Nonexistent"));
    assert_eq!(text(&session), source);
    argument(&mut session, "cancel");
    assert_eq!(text(&session), "* Task\nbody");
    let source = "* Task\nCLOCK: [2024-03-31 Sun 01:30]\nbody";
    let (_fixture, session) = editor("", "clock.org", source);
    let mut session = session.with_org_clock(autumn);
    resolve(&mut session, "1", "close");
    argument(&mut session, "2024-03-31 02:30");
    assert!(session.echo().contains("Nonexistent"));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .iter()
            .any(|r| r.text().contains("2024-03-31 02:30"))
    );
    assert_eq!(text(&session), source);
    assert!(session.buffer().unwrap().history().is_empty());
}
