use super::*;
use rev::command::{History, Motion};
pub(super) fn draft(session: &mut Session) {
    session.execute(Command::Org(OrgCommand::Capture));
    argument(session, "t");
    settle(session);
    assert!(session.capture_draft().is_some(), "{}", session.echo());
}
pub(super) fn finalize(session: &mut Session) {
    session.handle(control('c'));
    session.handle(control('c'));
    settle(session);
}
// An unopened destination is saved exactly once, with BOM and CRLF retained;
// the created canonical owner has one ordinary undo step and a clean save state.
#[test]
fn finalize_disk() {
    let (fixture, mut session) = editor(
        "org.capture.default_file='destination.org'",
        "origin.txt",
        "Origin 界",
    );
    let path = fixture.write("destination.org", "\u{feff}* Tasks\r\nbody EOF");
    let home = session.view().clone();
    draft(&mut session);
    session.handle(Event::Paste("Captured 界".into()));
    finalize(&mut session);
    assert!(session.capture_draft().is_none(), "{}", session.echo());
    assert_eq!(session.view(), &home);
    assert_eq!(session.echo(), "Capture finalized and saved");
    let written = std::fs::read_to_string(&path).unwrap();
    assert!(written.starts_with("\u{feff}* Tasks\r\nbody EOF\r\n** TODO Captured 界\r\n"));
    session.execute(Command::Org(OrgCommand::CaptureFinalize));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), written);
    session.load(&path).unwrap();
    assert!(!session.buffer().unwrap().is_modified());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "* Tasks\r\nbody EOF");
    assert_eq!(std::fs::read_to_string(path).unwrap(), written);
}
// A failed save into a missing parent does not publish an empty canonical
// destination or any insertion. Creating that parent permits one safe retry.
#[test]
fn finalize_retry() {
    let (fixture, mut session) = editor(
        "org.capture.default_file='missing/destination.org'",
        "origin.txt",
        "home",
    );
    draft(&mut session);
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    let value = text(&session);
    finalize(&mut session);
    assert!(
        session.echo().contains("not inserted; draft retained"),
        "{}",
        session.echo()
    );
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&session), value);
    assert!(!fixture.0.join("missing").exists());
    std::fs::create_dir(fixture.0.join("missing")).unwrap();
    finalize(&mut session);
    assert!(session.capture_draft().is_none(), "{}", session.echo());
    let source = std::fs::read_to_string(fixture.0.join("missing/destination.org")).unwrap();
    assert_eq!(source.matches("TODO !").count(), 1);
    assert_eq!(text(&session), "home");
}
// Both revalidation and cancellation retain actual draft text/history. A stale
// destination is never overwritten; abort then leaves the external bytes intact.
#[test]
fn finalize_stale_cancel() {
    let (fixture, mut session) = editor(
        "org.capture.default_file='destination.org'",
        "origin.txt",
        "home",
    );
    let path = fixture.write("destination.org", "* Tasks\n");
    draft(&mut session);
    session.insert('!');
    let stamp = session.buffer().unwrap().stamp();
    session.execute(Command::Org(OrgCommand::CaptureFinalize));
    session.handle(control('g'));
    settle(&mut session);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.echo().contains("draft retained"));
    std::fs::write(&path, "* External\n").unwrap();
    finalize(&mut session);
    assert!(session.capture_draft().is_some());
    assert!(session.echo().contains("destination changed"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    session.execute(Command::Org(OrgCommand::CaptureKill));
    assert_eq!(text(&session), "home");
    assert_eq!(std::fs::read_to_string(path).unwrap(), "* External\n");
    assert_eq!(session.remaining_input(Instant::now()), None);
}
// Capturing into the canonical origin preserves its earlier unsaved edit,
// publishes one additional history group, saves all actual source and maps home.
#[test]
fn finalize_canonical() {
    let (fixture, mut session) = editor(
        "org.capture.default_file='origin.org'",
        "origin.org",
        "* Tasks\r\nbody EOF",
    );
    session.execute(Command::Motion(Motion::BufferEnd));
    session.insert('!');
    let id = session.buffer().unwrap().id();
    draft(&mut session);
    session.insert('C');
    finalize(&mut session);
    assert_eq!(session.buffer().unwrap().id(), id);
    assert_eq!(session.buffer().unwrap().history().len(), 2);
    assert!(!session.buffer().unwrap().is_modified());
    assert!(text(&session).contains("body EOF!\r\n** TODO C"));
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("origin.org")).unwrap(),
        text(&session)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), "* Tasks\r\nbody EOF!");
}
