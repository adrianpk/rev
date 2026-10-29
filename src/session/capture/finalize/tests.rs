use super::*;
use crate::fixture;
use crate::{
    buffer::Buffers,
    config::prepare,
    view::{Size, View},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
struct Uncertain;
impl crate::file::Output for Uncertain {
    fn sync_directory(&mut self, _: &std::fs::File) -> std::io::Result<()> {
        Err(std::io::Error::other("injected directory sync failure"))
    }
}
fn settle(session: &mut Session) {
    let until = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(std::time::Instant::now()).is_none() {
            return;
        }
        assert!(std::time::Instant::now() < until);
        session.expire_input(std::time::Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
}
fn open() -> (fixture::Fixture, Session) {
    let fixture = fixture::Fixture::new();
    let startup = prepare(&fixture.write(
        "config.toml",
        "org.time.zone='UTC'\norg.capture.default_file='destination.org'",
    ));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.visit(&fixture.write("origin.txt", "home")).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 12,
        },
    )
    .unwrap();
    session.execute(Command::Org(C::Capture));
    session.handle(Event::Paste("t".into()));
    session.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    settle(&mut session);
    session.insert('!');
    (fixture, session)
}
fn publish(session: &mut Session) {
    let State { origin, stage } = session.capture.take().unwrap();
    let Stage::Draft(draft, target) = stage else {
        panic!("editable draft")
    };
    let buffer = session.buffers.get(draft).unwrap();
    let stamp = buffer.stamp();
    let work = target
        .finalize(
            &session.buffers,
            buffer,
            session.buffers.snapshot().settings(),
            origin.context.now.date,
        )
        .unwrap();
    let insertion = work.run(&Cancellation::default()).unwrap();
    let applied = session
        .publish_capture_with(insertion, origin.view, &mut Uncertain)
        .unwrap();
    assert_eq!(applied.outcome.publication(), Publication::Uncertain);
    session.capture_publication(
        origin,
        Validation {
            draft,
            stamp,
            target,
            work: None,
        },
        applied,
    );
}
// Real rename and ordinary canonical publication occur before the injected
// directory-sync error. Retry saves the same insertion; edited drafts stay owned.
#[test]
fn capture_unresolved_retry() {
    for scenario in ["retry", "edited draft", "changed source"] {
        let (fixture, mut session) = open();
        publish(&mut session);
        let path = fixture.0.join("destination.org");
        let written = std::fs::read_to_string(&path).unwrap();
        assert_eq!(written.matches("TODO !").count(), 1);
        assert!(session.capture_draft().is_some());
        if scenario == "edited draft" {
            session.insert('?');
        }
        if scenario == "changed source" {
            change_source(&mut session);
        }

        session.execute(Command::Org(C::CaptureFinalize));
        settle(&mut session);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), written);
        if scenario == "retry" {
            assert!(session.capture_draft().is_none(), "{}", session.echo());
            assert_eq!(session.echo(), "Capture finalized and saved");
        } else {
            assert!(session.capture_draft().is_some());
            assert!(session.echo().contains("changed"));
            session.execute(Command::Org(C::CaptureKill));
            assert!(session.echo().contains("already published source retained"));
        }
    }
}

fn change_source(session: &mut Session) {
    let Stage::Published(stored) = &session.capture.as_ref().unwrap().stage else {
        panic!("published receipt")
    };
    let buffer = session.buffers.get_mut(stored.source.buffer).unwrap();
    let edit = crate::buffer::edit::Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "unsaved change\n".into(),
    };
    let points = session.windows.capture_points(buffer).unwrap();
    let prepared = buffer.prepare(&[edit], &points.values, None).unwrap();
    session.windows.publish(buffer, prepared, points).unwrap();
}
