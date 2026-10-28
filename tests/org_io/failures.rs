use super::*;
use rev::org::io::ClockStore;

// File provenance never substitutes for source or instant semantics. Independent
// mismatches in bytes, calendar meaning or the real file observation all reject.
#[test]
fn clock_recovery_mismatch() {
    for mismatch in ["file", "source", "instant"] {
        let fixture = Fixture::new();
        let (_first, source, state) = saved(&fixture);
        let mut store = ClockStore::open(&state).unwrap();
        if mismatch == "file" {
            std::fs::write(&source, "changed").unwrap();
        } else {
            let mut record = store.clock().unwrap().clone();
            if mismatch == "source" {
                record.source = record.source.replace("14:30", "14:29");
            } else {
                record.start_seconds -= 86400;
            }
            store.write(Some(record)).unwrap();
        }
        let bytes = std::fs::read(&state).unwrap();
        let file = std::fs::read(&source).unwrap();
        let mut session = host(&fixture, None, true).with_org_clock(later);
        session.restore_clock_state(&state).unwrap();
        argument(&mut session, "recover");
        if mismatch == "instant" {
            argument(&mut session, "1");
            argument(&mut session, "resume");
            assert!(
                session.echo().contains("selected start instant"),
                "{}",
                session.echo()
            );
        } else {
            assert!(!session.echo().contains("interval(s)"));
        }
        cancel(&mut session);
        action(&mut session, C::ClockOut);
        assert!(session.echo().contains("No active"));
        assert_eq!(std::fs::read(&state).unwrap(), bytes);
        assert_eq!(std::fs::read(&source).unwrap(), file);
    }
}

// Unknown, malformed, non-regular and oversized states are retained. An
// unavailable store cannot undo a completed canonical clock-in or hide that fact.
#[test]
fn clock_state_rejection() {
    let fixture = Fixture::new();
    for (name, bytes) in [
        ("version", b"version=9\n".to_vec()),
        ("unknown", b"version=1\nunknown=true\n".to_vec()),
        ("syntax", b"invalid [".to_vec()),
        ("limit", vec![b'x'; 65537]),
    ] {
        let path = fixture.0.join(name);
        std::fs::write(&path, &bytes).unwrap();
        assert!(ClockStore::open(&path).is_err(), "{name}");
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    assert!(ClockStore::open(&fixture.0).is_err());
    let source = fixture.write("clock.org", "* Task\nbody");
    let path = fixture.write("bad-state.toml", "version=9");
    let mut session = host(&fixture, Some(&source), true);
    assert!(session.restore_clock_state(&path).is_err());
    action(&mut session, C::ClockIn);
    assert!(text(&session).contains("CLOCK:"));
    assert!(session.echo().contains("retained"), "{}", session.echo());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "version=9");
}

struct Failure {
    after: bool,
}
impl rev::file::Output for Failure {
    fn replace(&mut self, temporary: &Path, target: &Path) -> std::io::Result<()> {
        if !self.after {
            return Err(std::io::Error::other("owned replacement failure"));
        }
        std::fs::rename(temporary, target)
    }
    fn sync_directory(&mut self, _: &std::fs::File) -> std::io::Result<()> {
        Err(std::io::Error::other("owned completion failure"))
    }
}
// The existing native output boundary distinguishes pre-publication failure from
// published-but-unresolved completion. Owned temporaries are cleaned in both cases.
#[test]
fn clock_state_publication() {
    let fixture = Fixture::new();
    let (_first, _source, path) = saved(&fixture);
    let mut store = ClockStore::open(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let error = store
        .write_with(None, &mut Failure { after: false })
        .unwrap_err();
    assert!(error.contains("replacement failure"));
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let error = store
        .write_with(None, &mut Failure { after: true })
        .unwrap_err();
    assert!(error.contains("published; completion unresolved"));
    assert!(ClockStore::open(&path).unwrap().clock().is_none());
    assert_eq!(
        std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
        1
    );
}

// A store changed externally never authorizes replacement from cached state.
#[test]
fn clock_state_external_change() {
    let fixture = Fixture::new();
    let (_first, _source, path) = saved(&fixture);
    let mut store = ClockStore::open(&path).unwrap();
    std::fs::write(&path, "externally changed").unwrap();
    assert!(store.write(None).unwrap_err().contains("externally"));
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "externally changed"
    );
    assert_eq!(
        std::fs::read_dir(path.parent().unwrap()).unwrap().count(),
        1
    );
}

// Matching file metadata and source bytes still cannot turn a literal block
// heading into a task. Recovery requires the real native structural observation.
#[test]
fn clock_literal_state() {
    let fixture = Fixture::new();
    let (_first, source, state) = saved(&fixture);
    let body = "* Real\n#+begin_src text\n,* Forged\nCLOCK: [2024-03-01 Fri 14:30]\n#+end_src\n";
    std::fs::write(&source, body).unwrap();
    let mut store = ClockStore::open(&state).unwrap();
    let mut record = store.clock().unwrap().clone();
    record.file = rev::file::observe(&source).unwrap().unwrap();
    record.heading = body.find("* Forged").unwrap();
    record.heading_source = "* Forged".into();
    record.line_start = body.find("CLOCK:").unwrap();
    record.line_end = body.find("#+end_src").unwrap();
    record.source = "CLOCK: [2024-03-01 Fri 14:30]".into();
    store.write(Some(record)).unwrap();
    let bytes = std::fs::read(&state).unwrap();
    let mut session = host(&fixture, None, true).with_org_clock(later);
    session.restore_clock_state(&state).unwrap();
    argument(&mut session, "recover");
    assert!(session.echo().contains("unavailable"), "{}", session.echo());
    assert_eq!(text(&session), body);
    cancel(&mut session);
    action(&mut session, C::ClockOut);
    assert!(session.echo().contains("No active"));
    assert_eq!(std::fs::read(&state).unwrap(), bytes);
    assert_eq!(std::fs::read_to_string(&source).unwrap(), body);
}

// A failed state checkpoint after canonical publication reports its real
// boundary. Clock-out stays committed once, with the external state untouched.
#[test]
fn clock_checkpoint_failure() {
    let fixture = Fixture::new();
    let (mut session, source, state) = saved(&fixture);
    let disk = std::fs::read(&source).unwrap();
    std::fs::write(&state, "external state").unwrap();
    action(&mut session, C::ClockOut);
    assert!(session.echo().contains("externally"), "{}", session.echo());
    assert!(text(&session).contains(" =>  0:00"));
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    assert_eq!(std::fs::read_to_string(&state).unwrap(), "external state");
    assert_eq!(std::fs::read(&source).unwrap(), disk);
    action(&mut session, C::ClockOut);
    assert!(session.echo().contains("No active"));
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}
