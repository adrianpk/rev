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
    let old = prepared(&mut session);
    session.dismiss_completion();
    assert!(session.install_completion(old.clone()).is_err());
    session.begin_completion();
    settle(&mut session);
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
    settle(&mut session);
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
    settle(&mut session);
    assert!(session.echo().contains("read-only"));
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert!(session.buffer().unwrap().history().is_empty());
}

// Real cancelled work keeps its single slot until reaped. A newer request waits
// in the one pending slot and alone may publish its current selection.
#[test]
fn worker_replacement() {
    let (_fixture, mut session) = session();
    let request = session.capture_completion().unwrap();
    let old = request.generation;
    let id = request.point.stamp.buffer;
    session.completion_work.queue(request);
    session.expire_completion(std::time::Instant::now() + std::time::Duration::from_millis(1));
    assert!(session.completion_work.owns(id));
    session.begin_completion();
    assert!(session.completion_work.pending.is_some());
    settle(&mut session);
    assert!(session.completion().unwrap().generation > old);
    session.execute(Command::Completion(C::Cancel));
    settle(&mut session);
    assert!(!session.completion_work.owns(id));
    assert!(session.remaining_input(std::time::Instant::now()).is_none());
    assert!(session.buffer().unwrap().history().is_empty());
    assert_eq!(
        session.frame().unwrap().stamp(),
        session.buffer().unwrap().stamp()
    );
}

// Reusing the same revision/point after native syntax admission changes cannot
// resurrect an old proposal. This is independent of source history and work ID.
#[test]
fn changed_syntax() {
    let (_fixture, mut session) = session();
    let old = prepared(&mut session);
    let id = session.buffer().unwrap().id();
    session
        .buffers
        .get_mut(id)
        .unwrap()
        .select_mode(crate::buffer::Mode::Text)
        .unwrap();
    assert!(session.install_completion(old).is_err());
    assert!(session.buffer().unwrap().history().is_empty());
}

fn prepared(session: &mut Session) -> Completion {
    let request = session.capture_completion().unwrap();
    let buffer = session.buffers.get_mut(request.point.stamp.buffer).unwrap();
    let symbols = buffer
        .completion_symbols(std::time::Instant::now() + crate::completion::WORK_TIME)
        .unwrap();
    request.complete(symbols)
}
fn settle(session: &mut Session) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let now = std::time::Instant::now();
        session.expire_input(now);
        if session.completion_work.remaining(now).is_none() {
            break;
        }
        assert!(now < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
