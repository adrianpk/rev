use super::*;

fn session() -> (crate::fixture::Fixture, Session) {
    let fixture = crate::fixture::Fixture::new();
    let path = fixture.write("sample.rs", "fn sample() {}\n// sa");
    let mut session = Session::empty(crate::display::Size {
        width: 80,
        height: 24,
    })
    .unwrap();
    session.load(&path).unwrap();
    session.execute(Command::Motion(crate::command::Motion::BufferEnd));
    (fixture, session)
}

// A late result from a cancelled invocation or a superseded session cannot
// replace a newer selection, even at the same view, point and source revision.
#[test]
fn retired_results() {
    let (_fixture, mut session) = session();
    let old = session.prepare_completion().unwrap();
    session.dismiss_completion();
    assert!(session.install_completion(old.clone()).is_err());
    session.begin_completion();
    let current = session.completion().unwrap().generation;
    assert!(session.install_completion(old).is_err());
    assert_eq!(session.completion().unwrap().generation, current);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Read-only admission is checked on invocation and again by canonical edit
// preparation. Becoming read-only after presentation cannot publish a proposal.
#[test]
fn readonly_publication() {
    let (_fixture, mut session) = session();
    session.begin_completion();
    let id = session.buffer().unwrap().id();
    let stamp = session.buffer().unwrap().stamp();
    session.buffers.get_mut(id).unwrap().set_readonly(true);
    assert!(
        session
            .publish_completion()
            .unwrap_err()
            .contains("read-only")
    );
    session.begin_completion();
    assert!(session.echo().contains("read-only"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}
