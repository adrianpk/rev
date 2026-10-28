use super::*;
use rev::org::io::ClockStore;

// Real clock/save publication round-trips escaped paths, Unicode, BOM/CRLF and
// exact captured seconds. Restart requires explicit visit and interval adoption.
#[test]
fn clock_restart() {
    let fixture = Fixture::new();
    let (_first, source, state) = saved(&fixture);
    let before = std::fs::read(&source).unwrap();
    let stored = ClockStore::open(&state).unwrap().clock().unwrap().clone();
    assert_eq!(stored.start_seconds, 1709303437);
    assert_eq!(stored.path, source);
    let mut session = host(&fixture, None, true).with_org_clock(later);
    session.restore_clock_state(&state).unwrap();
    assert_eq!(text(&session), "");
    assert!(session.echo().contains("visit and verify"));
    assert_eq!(session.remaining_input(Instant::now()), None);
    argument(&mut session, "recover");
    assert!(session.echo().contains("interval(s)"), "{}", session.echo());
    argument(&mut session, "1");
    argument(&mut session, "resume");
    assert!(
        session.echo().contains("saved clock resumed"),
        "{}",
        session.echo()
    );
    assert_eq!(std::fs::read(&source).unwrap(), before);
    assert_eq!(
        ClockStore::open(&state)
            .unwrap()
            .clock()
            .unwrap()
            .acknowledged_seconds,
        1709307061
    );
    action(&mut session, C::ClockOut);
    assert!(
        text(&session).contains("14:30]--[2024-03-01 Fri 15:31] =>  1:01"),
        "{}",
        session.echo()
    );
    assert!(ClockStore::open(&state).unwrap().clock().is_none());
    session.execute(Command::SaveBuffer);
    assert_eq!(
        std::fs::read(&source).unwrap(),
        format!("\u{feff}{}", text(&session)).as_bytes()
    );
    let restored = ClockStore::open(&state).unwrap();
    assert!(restored.clock().is_none());
}

// Persistence off ignores even a non-regular state destination and creates no
// state directory. Opening a selected absent store is also read-only until use.
#[test]
fn clock_state_opt_in() {
    let fixture = Fixture::new();
    let source = fixture.write("clock.org", "* Task\nbody");
    let occupied = fixture.write("occupied", "unchanged");
    let mut session = host(&fixture, Some(&source), false);
    assert_eq!(
        session
            .restore_clock_state(&occupied.join("state.toml"))
            .unwrap(),
        Action::Wait
    );
    action(&mut session, C::ClockIn);
    assert!(session.echo().contains("started"));
    assert_eq!(std::fs::read_to_string(&occupied).unwrap(), "unchanged");
    assert!(!fixture.0.join("state").exists());
    let empty = fixture.0.join("state/org-clock.toml");
    let store = ClockStore::open(&empty).unwrap();
    assert!(store.clock().is_none());
    assert!(!empty.parent().unwrap().exists());
}

// The saved record may exist before a normal document save, but it never
// fabricates a missing source interval or silently accepts unaccounted time.
#[test]
fn clock_unsaved_recovery() {
    let fixture = Fixture::new();
    let source = fixture.write("clock.org", "* Task\nbody");
    let state = fixture.0.join("state/org-clock.toml");
    let mut first = host(&fixture, Some(&source), true);
    first.restore_clock_state(&state).unwrap();
    action(&mut first, C::ClockIn);
    let bytes = std::fs::read(&state).unwrap();
    let mut session = host(&fixture, None, true);
    session.restore_clock_state(&state).unwrap();
    argument(&mut session, "recover");
    assert!(!session.echo().contains("resumed"));
    assert_eq!(std::fs::read(&state).unwrap(), bytes);
    assert_eq!(std::fs::read_to_string(&source).unwrap(), "* Task\nbody");
    assert!(text(&first).contains("CLOCK:"));
}

// Merely discovering state neither visits nor edits a missing source. A failed
// explicit recovery retains the record, and keep/cancellation remain non-writing.
#[test]
fn clock_missing_source() {
    let fixture = Fixture::new();
    let (_first, source, state) = saved(&fixture);
    std::fs::remove_file(&source).unwrap();
    let bytes = std::fs::read(&state).unwrap();
    let mut session = host(&fixture, None, true);
    assert!(session.restore_clock_state(&state).is_ok());
    assert_eq!(text(&session), "");
    cancel(&mut session);
    assert_eq!(std::fs::read(&state).unwrap(), bytes);
    session.restore_clock_state(&state).unwrap();
    argument(&mut session, "keep");
    assert_eq!(text(&session), "");
    session.restore_clock_state(&state).unwrap();
    argument(&mut session, "recover");
    assert!(session.echo().contains("file changed"));
    assert_eq!(std::fs::read(&state).unwrap(), bytes);
    assert!(!source.exists());
}

// Default end accepts the captured instant, not a rounded earlier second. A
// same-minute restart retains the frozen zero interval without waiting in real time.
#[test]
fn clock_restart_zero() {
    let fixture = Fixture::new();
    let (_first, _source, state) = saved(&fixture);
    let mut session = host(&fixture, None, true);
    session.restore_clock_state(&state).unwrap();
    argument(&mut session, "recover");
    argument(&mut session, "1");
    argument(&mut session, "close");
    argument(&mut session, "");
    assert!(
        session.echo().contains("closed at the accepted end"),
        "{}",
        session.echo()
    );
    assert!(text(&session).contains("14:30]--[2024-03-01 Fri 14:30] =>  0:00"));
    assert!(ClockStore::open(&state).unwrap().clock().is_none());
}
