use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use rev::buffer::{Buffers, Mode};
use rev::command::{Command, LinearCommand};
use rev::coordinates::Scalar;
use rev::display::{Frame, Position, Size};
use rev::gutter::Change;
use rev::session::Session;
use rev::view::View;
use rev::window::Area;
use std::time::{Duration, Instant};

const SIZE: Size = Size {
    width: 28,
    height: 12,
};

fn editor(mode: Mode, wrapped: bool) -> Session {
    let marker = if mode == Mode::Org { "***" } else { "###" };
    let source = format!("{marker} Heading 界e\u{301}\twith words to wrap\nordinary body\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(mode).unwrap();
    if !wrapped {
        buffer.toggle_soft_wrap();
    }
    let mut view = View::new(buffer);
    view.bookmark_markers = false;
    view.gutter.apply(Change::Visible(false));
    let mut session = Session::new(buffers, view, SIZE).unwrap();
    settle(&mut session);
    session
}

fn settle(session: &mut Session) {
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
}

fn projection(session: &Session, width: u16, left: usize) -> (Vec<String>, Vec<Option<Scalar>>) {
    let buffer = session.buffer().unwrap();
    let mut view = session.view().clone();
    view.left = left;
    let size = Size { width, ..SIZE };
    let frame = Frame::project(buffer, &view, size, "").unwrap();
    let area = Area::new(0, 0, width, size.text_rows()).unwrap();
    let mut hits = Vec::new();
    for row in 0..size.text_rows() {
        for column in 0..width {
            hits.push(
                view.hit(buffer, area, Position { column, row })
                    .unwrap()
                    .map(|point| {
                        assert_eq!(point.stamp, buffer.stamp());
                        point.position
                    }),
            );
        }
    }
    (
        frame
            .rows()
            .iter()
            .take(usize::from(size.text_rows()))
            .map(|row| row.text().to_owned())
            .collect(),
        hits,
    )
}

// Every wrapped/clipped hit in two native formats agrees with exact replacement.
// Unicode deltas on two rows cannot revive old fragments or change canonical hits.
#[test]
fn continuity_projection_hits() {
    for mode in [Mode::Org, Mode::Markdown] {
        for wrapped in [true, false] {
            compare_replacement(mode, wrapped);
        }
    }
}

fn compare_replacement(mode: Mode, wrapped: bool) {
    let mut session = editor(mode, wrapped);
    let old = session
        .view()
        .wrapped_rows(session.buffer().unwrap(), 28, 20)
        .unwrap()[0]
        .clone();
    session.replace(7..7, "界").unwrap();
    let at = session.buffer().unwrap().len_chars() - 4;
    session.replace(at..at, "e\u{301}").unwrap();
    assert!(
        session
            .buffer()
            .unwrap()
            .slice(&rev::buffer::Span {
                stamp: old.stamp,
                range: old.source.start.0..old.source.end.0
            })
            .is_err()
    );
    let before = Frame::project(session.buffer().unwrap(), session.view(), SIZE, "").unwrap();
    assert!(before.rows()[0].text().contains('✸'));
    let cases = [(11, 0), (28, 0), (11, 1), (11, 3), (11, 4), (11, 8)];
    let pending: Vec<_> = cases
        .iter()
        .map(|&(width, left)| projection(&session, width, left))
        .collect();
    assert!(
        pending
            .iter()
            .any(|(_, hits)| hits.iter().any(Option::is_none))
    );
    settle(&mut session);
    for ((width, left), expected) in cases.into_iter().zip(pending) {
        assert_eq!(
            projection(&session, width, left),
            expected,
            "{mode:?}/{wrapped}/{width}/{left}"
        );
    }
    assert_eq!(session.buffer().unwrap().history().len(), 2);
}

fn mouse(kind: MouseEventKind, column: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row: 0,
        modifiers: KeyModifiers::NONE,
    })
}

// The real drag/copy owner consumes the carried hit map. Copied text comes from
// current canonical scalars, including source heading punctuation and Unicode.
#[test]
fn continuity_selection() {
    for mode in [Mode::Org, Mode::Markdown] {
        let mut session = editor(mode, false);
        session.replace(7..7, "界").unwrap();
        let buffer = session.buffer().unwrap();
        let area = Area::new(0, 0, SIZE.width, SIZE.text_rows()).unwrap();
        let start = session
            .view()
            .hit(buffer, area, Position { row: 0, column: 2 })
            .unwrap()
            .unwrap();
        let end = session
            .view()
            .hit(buffer, area, Position { row: 0, column: 16 })
            .unwrap()
            .unwrap();
        let expected = buffer
            .slice(&buffer.span(start.position.0..end.position.0).unwrap())
            .unwrap()
            .to_string();
        assert!(expected.contains(if mode == Mode::Org { '*' } else { '#' }));
        assert!(expected.contains('界'));
        session.handle(mouse(MouseEventKind::Down(MouseButton::Left), 2));
        session.handle(mouse(MouseEventKind::Drag(MouseButton::Left), 16));
        session.handle(mouse(MouseEventKind::Up(MouseButton::Left), 16));
        assert_eq!(session.view().point, end);
        let frame = Frame::project(session.buffer().unwrap(), session.view(), SIZE, "").unwrap();
        assert!(frame.rows()[0].selection().is_some());
        assert!(frame.rows()[0].text().contains('✸'));
        session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(session.killed_text().unwrap().to_string(), expected);
        assert_eq!(session.buffer().unwrap().history().len(), 1);
    }
}
