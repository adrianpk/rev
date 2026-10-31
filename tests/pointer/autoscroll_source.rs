use std::time::{Duration, Instant};

use rev::buffer::Buffers;
use rev::command::{Command, Motion, Split};
use rev::coordinates::{Coordinates, Scalar};
use rev::display::Position;
use rev::gutter::{Change, Side};
use rev::session::{Action, Session};
use rev::view::View;

use super::autoscroll::TICK;
use super::scroll::{editor, lines, size};
use super::selection::{drag, range};

// Native tab/Unicode mapping remains the endpoint owner at a scrolled edge,
// under either gutter side and a nonzero horizontal origin, even read-only.
#[test]
fn unicode_edges() {
    let fixture = super::config_fixture::Fixture::new();
    let line = "ab\t界e\u{301}👩‍💻\r\n";
    let source = line.repeat(30);
    for (width, side, wrap) in [1, 4, 8, 32]
        .into_iter()
        .flat_map(|width| [Side::Left, Side::Right].map(move |side| (width, side)))
        .flat_map(|(width, side)| [true, false].map(move |wrap| (width, side, wrap)))
    {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(rev::config::prepare(&root).snapshot).unwrap();
        let id = buffers.create(&source).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        buffer.set_readonly(true);
        if !wrap {
            buffer.toggle_soft_wrap();
        }
        let mut view = View::new(buffer);
        view.bookmark_markers = false;
        view.gutter.apply(Change::Side(side));
        view.point.position = Scalar(3);
        view.left = width.max(3);
        let mut session = Session::new(buffers, view, size()).unwrap();
        let origin = if side == Side::Left { 6 } else { 0 };
        let now = Instant::now();
        session.handle_at(
            super::focus::press(Position {
                column: origin,
                row: 0,
            }),
            now,
        );
        session.handle_at(drag(origin + 3, 10), now);
        assert_eq!(session.expire_input(now + TICK), Action::Redraw);
        let point = session.view().point.position;
        let (anchor, end) = if wrap {
            (0, if width == 1 { 3 } else { 2 })
        } else {
            (3, 6)
        };
        assert_eq!(point, Scalar(12 * line.chars().count() + end));
        assert_eq!(range(&session), Some(anchor..point.0));
        assert!(
            Coordinates::new(session.buffer().unwrap())
                .unwrap()
                .is_boundary(point)
                .unwrap()
        );
        let mut saved = Vec::new();
        rev::file::write(session.buffer().unwrap(), &mut saved).unwrap();
        assert_eq!(saved, source.as_bytes());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// Reversing edges starts a fresh interval; a subsequent inside or horizontal
// crossing pauses the source instead of transferring work to the other pane.
#[test]
fn direction_changes() {
    let mut session = editor(&lines(100));
    session.execute(Command::Split(Split::Below));
    session.select_view(session.windows().order()[1]).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    session.frame().unwrap();
    let top = session.view().top;
    let now = Instant::now();
    session.handle_at(super::focus::press(Position { column: 7, row: 6 }), now);
    session.handle_at(drag(7, 5), now);
    session.handle_at(drag(7, 11), now + Duration::from_millis(50));
    assert_eq!(session.expire_input(now + TICK), Action::Wait);
    assert_eq!(session.view().top, top);
    assert_eq!(
        session.expire_input(now + Duration::from_millis(150)),
        Action::Redraw
    );
    assert_eq!(session.view().top, top + 3);
    session.handle_at(drag(u16::MAX, 6), now + TICK * 2);
    assert_eq!(session.remaining_input(now + TICK * 2), None);
}

// Outside columns and rows clamp to the source's own document. Neither shared
// nor different-buffer peers can acquire focus, mark or a changed viewport.
#[test]
fn peer_isolation() {
    let fixture = super::config_fixture::Fixture::new();
    let path = fixture.write("peer.txt", &lines(50));
    for different in [false, true] {
        let mut session = editor(&lines(50));
        session.execute(Command::Split(Split::Right));
        let source = session.windows().selected_id();
        let peer = session.windows().order()[1];
        if different {
            session.select_view(peer).unwrap();
            session.load(&path).unwrap();
            session.select_view(source).unwrap();
        }
        let other = session.windows().get(peer).unwrap().clone();
        let now = Instant::now();
        super::autoscroll::arm(&mut session, now);
        session.handle_at(drag(u16::MAX, u16::MAX), now);
        session.expire_input(now + TICK);
        assert_eq!(session.windows().selected_id(), source);
        assert_eq!(session.windows().get(peer).unwrap(), &other);
        assert_eq!(session.view().top, 3);
        assert_eq!(range(&session), Some(1..103));
        assert!(session.cancel_pointer_capture());
        assert_eq!(session.remaining_input(now + TICK), None);
        let before = session.view().clone();
        assert_eq!(session.expire_input(now + TICK * 100), Action::Wait);
        assert_eq!(session.view(), &before);
    }
}
