use super::*;
use rev::{command::History, session::RefileStatus};
pub(super) fn editor(source: &str, destination: &str) -> (Fixture, Session) {
    let fixture = Fixture::new();
    let startup = prepare(&fixture.write("config.toml", "org.refile.targets=['destination.org']"));
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    fixture.write("destination.org", destination);
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&fixture.write("source.org", source)).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    (fixture, session)
}
// Native refile publishes both actual owners together, creates no disk writes,
// then records each ordinary save independently with one history group per side.
#[test]
fn refile_native() {
    let source = "* Note\n** Child\n* Stay\n";
    let destination = "\u{feff}* Projects\r\nbody EOF";
    let (fixture, mut session) = editor(source, destination);
    action(&mut session, C::Refile);
    assert!(session.echo().contains("1 targets"), "{}", session.echo());
    argument(&mut session, "1");
    assert_eq!(text(&session), "* Stay\n");
    assert_eq!(
        session
            .refile_results()
            .iter()
            .map(|e| e.status)
            .collect::<Vec<_>>(),
        [RefileStatus::ChangedInBuffer; 2]
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("source.org")).unwrap(),
        source
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
        destination
    );
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::SaveBuffer);
    assert_eq!(session.refile_results()[0].status, RefileStatus::Saved);
    session.load(&fixture.0.join("destination.org")).unwrap();
    assert_eq!(
        text(&session),
        "* Projects\r\nbody EOF\r\n** Note\r\n*** Child\r\n"
    );
    assert!(session.buffer().unwrap().has_bom());
    assert_eq!(session.buffer().unwrap().history().len(), 1);
    session.execute(Command::SaveBuffer);
    assert!(
        session
            .refile_results()
            .iter()
            .all(|e| e.status == RefileStatus::Saved)
    );
    session.execute(Command::History(History::Undo));
    assert_eq!(
        session.refile_results()[1].status,
        RefileStatus::ChangedAfter
    );
    assert_eq!(text(&session), "* Projects\r\nbody EOF");
    assert!(
        std::fs::read_to_string(fixture.0.join("destination.org"))
            .unwrap()
            .contains("** Note")
    );
}
// Copy leaves the actual source/history untouched. C-g cancels target choice
// without creating a destination owner, edit, file or background polling loop.
#[test]
fn refile_copy_cancel() {
    let (fixture, mut session) = editor("* Note\n", "* Target\n");
    action(&mut session, C::Refile);
    cancel(&mut session);
    settle(&mut session);
    assert_eq!(text(&session), "* Note\n");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.remaining_input(Instant::now()), None);
    action(&mut session, C::RefileCopy);
    argument(&mut session, "1");
    assert_eq!(text(&session), "* Note\n");
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(session.refile_results()[0].status, RefileStatus::Unchanged);
    session.load(&fixture.0.join("destination.org")).unwrap();
    assert_eq!(text(&session), "* Target\n** Note\n");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
        "* Target\n"
    );
}
