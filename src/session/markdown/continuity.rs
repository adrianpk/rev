use super::*;
use crate::buffer::{Buffers, Mode};
use crate::display::{Frame, Size};
use crate::view::View;

fn editor() -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("## Heading\nordinary body\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Markdown).unwrap();
    let work = buffer.markdown_work().unwrap();
    buffer.install_markdown(work.run(&crate::process::Cancellation::default()).unwrap());
    let view = View::new(buffer);
    Session::new(
        buffers,
        view,
        Size {
            width: 40,
            height: 8,
        },
    )
    .unwrap()
}

fn owned(session: &mut Session, limited: bool) {
    let id = session.view().point.stamp.buffer;
    let buffer = session.buffers.get_mut(id).unwrap();
    let source = (buffer.stamp(), buffer.markdown_generation());
    let result = if limited {
        let work = buffer.markdown_work().unwrap();
        let cancel = crate::process::Cancellation::default();
        cancel.cancel();
        work.run(&cancel)
    } else {
        crate::markdown::Work::new(
            source.0,
            ropey::Rope::from_str(
                &buffer
                    .slice(&buffer.span(0..buffer.len_chars()).unwrap())
                    .unwrap()
                    .to_string(),
            ),
        )
        .run(&crate::process::Cancellation::default())
    };
    session
        .markdown
        .job
        .start(source.0, move |_| result)
        .unwrap();
    session.markdown.source = Some(source);
    session.markdown.deadline = Some(Instant::now() + POLL);
}

fn reap(session: &mut Session) {
    let until = Instant::now() + Duration::from_secs(5);
    while !session.poll_markdown() {
        assert!(
            Instant::now() < until,
            "owned Markdown result did not finish"
        );
        std::thread::yield_now();
    }
    assert!(!session.markdown.job.is_active());
    assert!(session.markdown.source.is_none());
    assert!(session.markdown.deadline.is_none());
}

fn decorated(session: &Session) -> bool {
    Frame::project(session.buffer().unwrap(), session.view(), session.size, "")
        .unwrap()
        .rows()[0]
        .text()
        .contains('○')
}

// A failed/limited current result retires the facade through the real poll
// owner. Reaping an older revision instead preserves a newer admitted chain.
#[test]
fn limited_and_obsolete() {
    for obsolete in [false, true] {
        let mut session = editor();
        session.replace(16..16, "界").unwrap();
        if obsolete {
            owned(&mut session, true);
        }
        session.replace(17..17, "z").unwrap();
        assert!(decorated(&session));
        if !obsolete {
            owned(&mut session, true);
        }
        reap(&mut session);
        assert_eq!(decorated(&session), obsolete);
        assert!(session.buffer().unwrap().markdown_observation().is_none());
    }
}

// Same-revision mode suspension changes the existing generation. A result from
// the previous generation cannot restore either exact or carried presentation.
#[test]
fn generation_and_close() {
    for close in [false, true] {
        let mut session = editor();
        owned(&mut session, false);
        let old = session.buffer().unwrap().id();
        if close {
            let kill = session.buffers.prepare_kill(old).unwrap();
            session.buffers.kill(kill).unwrap();
        } else {
            let buffer = session.buffers.get_mut(old).unwrap();
            buffer.select_mode(Mode::Text).unwrap();
            buffer.select_mode(Mode::Markdown).unwrap();
        }
        reap(&mut session);
        if close {
            assert!(session.buffers.get(old).is_err());
        } else {
            assert!(!decorated(&session));
        }
    }
}
