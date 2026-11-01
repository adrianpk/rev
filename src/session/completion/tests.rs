use super::*;
use crate::buffer::edit::Replacement;

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

// Debounce and exclusive parser work must not remove current source colors.
// Observe both phases before reaping so a fast worker cannot hide the flicker.
// A request can start before the edited revision has had its first frame.
#[test]
fn completion_colors() {
    for paint in [false, true] {
        let (_fixture, mut session) = session();
        session.frame().unwrap();
        session.handle(crossterm::event::Event::Key(
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Char('m'),
                crossterm::event::KeyModifiers::NONE,
            ),
        ));
        assert!(session.completion_work.pending.is_some());
        if paint {
            let pending = session.frame().unwrap();
            assert_eq!(session.buffer().unwrap().mode(), "Rust", "debounce colors");
            assert!(!pending.rows()[0].syntax().is_empty());
        }
        let request = session.capture_completion().unwrap();
        let id = request.point.stamp.buffer;
        session.completion_work.queue(request);
        let (sources, _) = session.completion_sources(id).unwrap();
        session
            .completion_work
            .start(sources, std::time::Instant::now())
            .unwrap();
        let working = session.frame().unwrap();
        assert_eq!(
            session.buffer().unwrap().mode(),
            "Rust",
            "worker colors: {paint}"
        );
        assert!(!working.rows()[0].syntax().is_empty());
        settle(&mut session);
    }
}

// An edit during borrowed parser work gets fresh current colors. Returning
// the old syntax must not overwrite that newer text's parser or projection.
#[test]
fn edited_colors() {
    let (_fixture, mut session) = session();
    session.frame().unwrap();
    let id = session.buffer().unwrap().id();
    let (sources, _) = session.completion_sources(id).unwrap();
    assert!(session.buffer().unwrap().syntax().working());
    session.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('m'),
            crossterm::event::KeyModifiers::NONE,
        ),
    ));
    let stamp = session.buffer().unwrap().stamp();
    assert!(!session.buffer().unwrap().syntax().working());
    assert!(!session.frame().unwrap().rows()[0].syntax().is_empty());
    session.restore_completion_sources(sources);
    assert_eq!(session.buffer().unwrap().stamp(), stamp);
    assert_eq!(session.buffer().unwrap().mode(), "Rust");
    assert!(!session.frame().unwrap().rows()[0].syntax().is_empty());
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
    for org in [false, true] {
        let (fixture, mut session) = session();
        if org {
            session.load(&fixture.write("sample.org", "* TO")).unwrap();
            session.execute(Command::Motion(crate::command::Motion::BufferEnd));
        }
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
    // Queue before polling: begin_completion may eagerly reap an already
    // finished worker, so its pending-slot lifetime is not a timing guarantee.
    let newest = session.capture_completion().unwrap();
    session.completion_work.queue(newest);
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

// Text revisions, rename paths, language/grammar changes and close all retire
// other-buffer observations even while the current target/view stays identical.
#[test]
fn changed_observations() {
    for action in ["edit", "rename", "grammar", "identity", "language", "close"] {
        let (fixture, mut session) = session();
        let path = fixture.write("other.rs", "fn sample_other() {}\n");
        let id = session.buffers.visit(&path).unwrap();
        session.begin_completion();
        settle(&mut session);
        let old = session.completion().unwrap().clone();
        assert!(
            old.candidates()
                .iter()
                .any(|candidate| candidate.text() == "sample_other"),
            "{action}"
        );
        change_observation(&mut session, id, action);
        assert!(session.completion().is_none(), "{action}");
        assert!(session.install_completion(old).is_err(), "{action}");
        assert!(session.buffer().unwrap().history().is_empty(), "{action}");
    }
}

fn change_observation(session: &mut Session, id: crate::buffer::BufferId, action: &str) {
    match action {
        "edit" => {
            let buffer = session.buffers.get_mut(id).unwrap();
            let edit = Replacement {
                source: buffer.span(0..buffer.len_chars()).unwrap(),
                text: "fn sample_new() {}\n".into(),
            };
            let prepared = buffer.prepare(&[edit], &[], None).unwrap();
            buffer.publish(prepared, &mut []).unwrap();
        }
        "rename" => {
            let path = session.buffers.get(id).unwrap().file().unwrap().path();
            let rename = crate::file::Rename::capture(path).unwrap();
            session.buffers.rename(rename, "renamed.rs").unwrap();
        }
        "grammar" => {
            let fixture = crate::fixture::Fixture::new();
            let config = fixture.write("config.toml", "language.rust.grammar = 'c'");
            let settings = crate::config::resolve(&crate::config::load(&config).unwrap()).unwrap();
            let buffer = session.buffers.get_mut(id).unwrap();
            let generation = buffer.syntax_generation();
            buffer.apply_settings(&settings);
            assert!(buffer.syntax_generation() > generation);
            assert_eq!(buffer.mode(), "C");
        }
        "identity" => {
            let fixture = crate::fixture::Fixture::new();
            let config = fixture.write(
                "config.toml",
                "[files.associations]\n'*.rs' = 'custom'\n[language.custom]\ngrammar = 'rust'",
            );
            let settings = crate::config::resolve(&crate::config::load(&config).unwrap()).unwrap();
            let buffer = session.buffers.get_mut(id).unwrap();
            buffer.apply_settings(&settings);
            assert_eq!(buffer.syntax_language(), "custom");
            assert_eq!(buffer.mode(), "Rust");
        }
        "language" => {
            let path = session.buffers.get(id).unwrap().file().unwrap().path();
            let rename = crate::file::Rename::capture(path).unwrap();
            session.buffers.rename(rename, "other.go").unwrap();
        }
        "close" => {
            let kill = session.buffers.prepare_kill(id).unwrap();
            session.buffers.kill(kill).unwrap();
        }
        _ => unreachable!(),
    }
}

// Snapshot admission prioritizes the current source, then recent observations.
// Overflow excludes the least recently used sources before borrowing any parser.
#[test]
fn snapshot_inventory() {
    let (fixture, mut session) = session();
    let current = session.buffer().unwrap().id();
    let mut ids = Vec::new();
    for n in 0..35 {
        let path = fixture.write(&format!("source-{n}.rs"), "fn sample_other() {}\n");
        ids.push(session.buffers.visit(&path).unwrap());
    }
    session.touch_completion_source(ids[0]);
    let (sources, limited) = session.completion_sources(current).unwrap();
    assert!(limited);
    assert_eq!(sources.len(), crate::completion::source::MAX_BUFFERS);
    assert_eq!(sources[0].observation.stamp.buffer, current);
    assert_eq!(sources[1].observation.stamp.buffer, ids[0]);
    assert!(
        !sources
            .iter()
            .any(|source| source.observation.stamp.buffer == ids[1])
    );
    session.restore_completion_sources(sources);
    assert!(
        session
            .buffers
            .iter()
            .all(|buffer| !buffer.syntax().working())
    );
    assert!(session.buffer().unwrap().history().is_empty());
}

// Aggregate snapshot bytes and the per-buffer byte limit are independent of
// symbol count; large comment payloads cannot bypass either admission boundary.
#[test]
fn snapshot_bytes() {
    let (fixture, mut session) = session();
    let current = session.buffer().unwrap().id();
    let size = crate::completion::MAX_BYTES;
    let document = format!("//{}", "x".repeat(size - 2));
    for n in 0..5 {
        let path = fixture.write(&format!("large-{n}.rs"), &document);
        session.buffers.visit(&path).unwrap();
    }
    let path = fixture.write("oversize.rs", &format!("{document}x"));
    let oversize = session.buffers.visit(&path).unwrap();
    let (sources, limited) = session.completion_sources(current).unwrap();
    assert!(limited);
    assert_eq!(sources[0].observation.stamp.buffer, current);
    assert_eq!(sources.len(), 4);
    assert!(
        sources
            .iter()
            .map(|source| source.text.len_bytes())
            .sum::<usize>()
            <= crate::completion::source::MAX_SNAPSHOTS
    );
    assert!(
        !sources
            .iter()
            .any(|source| source.observation.stamp.buffer == oversize)
    );
    session.restore_completion_sources(sources);
    assert!(
        session
            .buffers
            .iter()
            .all(|buffer| !buffer.syntax().working())
    );
}

// An edit after snapshot transfer retires the old proposal. A returned parser
// cannot replace a newer current parser or relabel stale observations as current.
#[test]
fn changed_in_flight() {
    let (fixture, mut session) = session();
    let path = fixture.write("other.rs", "fn sample_other() {}\n");
    let other = session.buffers.visit(&path).unwrap();
    session.begin_completion();
    assert!(session.completion_work.owns(other));
    change_observation(&mut session, other, "edit");
    settle(&mut session);
    assert!(session.completion().is_none());
    session.begin_completion();
    settle(&mut session);
    let names: Vec<_> = session
        .completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|candidate| candidate.text())
        .collect();
    assert!(names.contains(&"sample_new"));
    assert!(!names.contains(&"sample_other"));
    assert!(session.buffer().unwrap().history().is_empty());
}

// Resolved additional edits remain proposals until final admission. A read-only
// transition after presentation must preserve every edit, point and history.
#[test]
fn readonly_resolved() {
    let (_fixture, mut session) = session();
    let mut completion = prepared(&mut session);
    let point = session.view().point;
    let buffer = session.buffer().unwrap();
    let original = buffer
        .slice(&buffer.span(0..buffer.len_chars()).unwrap())
        .unwrap()
        .to_string();
    completion.resolved(crate::lsp::completion::Item {
        label: "sample".into(),
        kind: Some(3),
        text: "sample".into(),
        range: point.position.0 - 2..point.position.0,
        additional: vec![crate::lsp::completion::Edit {
            range: 0..0,
            text: "use demo::Thing;\n".into(),
        }],
        proposal: None,
    });
    session
        .buffers
        .get_mut(point.stamp.buffer)
        .unwrap()
        .set_readonly(true);
    assert_eq!(
        session.finish_completion(completion).unwrap_err(),
        "buffer is read-only"
    );
    let buffer = session.buffer().unwrap();
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        original
    );
    assert_eq!(session.view().point, point);
    assert!(buffer.history().is_empty());
}
