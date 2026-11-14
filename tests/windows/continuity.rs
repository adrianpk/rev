use super::*;
use rev::buffer::Mode;
use rev::command::{MarkdownCommand, OrgCommand};
use rev::display::Frame;
use std::time::{Duration, Instant};

const SIZE: Size = Size {
    width: 81,
    height: 16,
};

fn settled(mode: Mode) -> Session {
    let marker = if mode == Mode::Org { "**" } else { "##" };
    let source = format!("{marker} Heading\nordinary body with enough words to wrap\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(mode).unwrap();
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.gutter.apply(rev::gutter::Change::Visible(false));
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        session.frame().unwrap();
        if session.remaining_input(Instant::now()).is_none() {
            break;
        }
        assert!(Instant::now() < until);
        session.expire_input(Instant::now() + Duration::from_millis(10));
        std::thread::yield_now();
    }
    split(&mut session, Split::Right);
    split(&mut session, Split::Right);
    session
}

fn toggle(session: &mut Session, mode: Mode) {
    session.execute(if mode == Mode::Org {
        Command::Org(OrgCommand::PrettyHeadings)
    } else {
        Command::Markdown(MarkdownCommand::Pretty)
    });
}

fn check(session: &Session, mode: Mode, source_view: Option<rev::window::ViewId>) {
    let geometry = session.windows().geometry(SIZE).unwrap();
    let mut widths = BTreeSet::new();
    let buffer = session.buffer().unwrap();
    for (id, area) in geometry.panes() {
        widths.insert(area.width());
        let view = session.windows().get(*id).unwrap();
        assert_eq!(view.point.stamp, buffer.stamp());
        view.cell(buffer).unwrap();
        let pretty = if mode == Mode::Org {
            view.pretty_headings()
        } else {
            view.pretty_markdown()
        };
        assert_eq!(pretty, Some(*id) != source_view);
        let frame = Frame::project(
            buffer,
            view,
            Size {
                width: area.width(),
                height: 10,
            },
            "",
        )
        .unwrap();
        assert_eq!(frame.rows()[0].text().contains('○'), pretty);
        assert!(frame.cursor().is_some());
    }
    assert_eq!(widths.len(), 3);
}

// Three unequal panes keep their own pretty policy and current source points
// across five pending keys on two rows. Shared Org indentation remains immediate.
#[test]
fn continuity_pane_policies() {
    for mode in [Mode::Org, Mode::Markdown] {
        let mut session = settled(mode);
        let selected = session.windows().selected_id();
        toggle(&mut session, mode);
        check(&session, mode, Some(selected));
        for (step, key) in ["x", "界", "e\u{301}", "y", "z"].into_iter().enumerate() {
            let at = if step % 2 == 0 {
                5
            } else {
                session.buffer().unwrap().len_chars() - 5
            };
            session.replace(at..at, key).unwrap();
            check(&session, mode, Some(selected));
        }
        let stamp = session.buffer().unwrap().stamp();
        toggle(&mut session, mode);
        check(&session, mode, None);
        toggle(&mut session, mode);
        check(&session, mode, Some(selected));
        assert_eq!(session.buffer().unwrap().stamp(), stamp);
        assert_eq!(session.buffer().unwrap().history().len(), 5);
        if mode == Mode::Org {
            indentation(&mut session);
        }
        assert_eq!(session.remaining_input(Instant::now()), None);
    }
}

fn indentation(session: &mut Session) {
    let buffer = session.buffer().unwrap();
    let old = session.view().wrapped_rows(buffer, 30, 20).unwrap()[1].clone();
    session.execute(Command::Org(OrgCommand::IndentDisable));
    assert!(!session.buffer().unwrap().org_indentation());
    assert_eq!(old.margin, 4);
    for (_, view) in session.windows().iter() {
        assert_eq!(
            view.wrapped_rows(session.buffer().unwrap(), 30, 20)
                .unwrap()[1]
                .margin,
            0
        );
    }
    session.execute(Command::Org(OrgCommand::IndentEnable));
    assert!(session.buffer().unwrap().org_indentation());
    for (_, view) in session.windows().iter() {
        assert_eq!(
            view.wrapped_rows(session.buffer().unwrap(), 30, 20)
                .unwrap()[1]
                .margin,
            4
        );
    }
}
