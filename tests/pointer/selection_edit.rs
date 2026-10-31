use rev::buffer::Buffers;
use rev::command::{Command, History, LinearCommand, Motion};
use rev::coordinates::Scalar;
use rev::display::{Position, Size};
use rev::gutter::{Change, Side};
use rev::session::Session;
use rev::view::View;

use super::scroll::{editor, size};
use super::selection::{drag, range, release};

const SOURCE: &str = "a\t界e\u{301}👩‍💻\r\nlast\n";

fn bytes(session: &Session) -> Vec<u8> {
    let mut bytes = Vec::new();
    rev::file::write(session.buffer().unwrap(), &mut bytes).unwrap();
    bytes
}

fn selected(read_only: bool) -> Session {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(SOURCE).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.set_readonly(read_only);
    let view = View::new(buffer);
    let mut session = Session::new(buffers, view, size()).unwrap();
    session.handle(super::focus::press(Position { column: 7, row: 0 }));
    session.handle(drag(8, 1));
    session.handle(release(8, 1));
    session
}

// Native copy/kill/yank consume only canonical selected text. A release does
// not populate the ring, read-only refusal retains region, and history is exact.
#[test]
fn editing_commands() {
    for read_only in [false, true] {
        let mut session = selected(read_only);
        assert_eq!(range(&session), Some(1..12));
        assert_eq!(bytes(&session), SOURCE.as_bytes());
        assert!(session.buffer().unwrap().history().is_empty());
        session.execute(Command::Linear(LinearCommand::Yank));
        assert_eq!(session.echo(), "Kill ring is empty");
        session.execute(Command::Linear(LinearCommand::Copy));
        assert_eq!(session.echo(), "Region copied");
        assert_eq!(bytes(&session), SOURCE.as_bytes());
        session.execute(Command::Linear(LinearCommand::Kill));
        if read_only {
            assert_eq!(bytes(&session), SOURCE.as_bytes());
            assert_eq!(range(&session), Some(1..12));
            assert!(session.buffer().unwrap().history().is_empty());
            continue;
        }
        assert_eq!(bytes(&session), b"ast\n");
        assert_eq!(session.buffer().unwrap().history().len(), 1);
        session.execute(Command::History(History::Undo));
        assert_eq!(bytes(&session), SOURCE.as_bytes());
        assert!(!session.buffer().unwrap().is_modified());
        session.execute(Command::History(History::Redo));
        assert_eq!(bytes(&session), b"ast\n");
        session.execute(Command::History(History::Undo));
        session.execute(Command::KeyboardQuit);
        session.execute(Command::Motion(Motion::BufferEnd));
        session.execute(Command::Linear(LinearCommand::Yank));
        assert_eq!(
            bytes(&session),
            format!("{SOURCE}\t界e\u{301}👩‍💻\r\nla").as_bytes()
        );
        session.execute(Command::History(History::Undo));
        assert_eq!(bytes(&session), SOURCE.as_bytes());
    }
}

// The dragged endpoints stay at whole Unicode clusters under every selected
// tab width, either gutter side and a nonzero horizontal viewport origin.
#[test]
fn unicode_endpoints() {
    let fixture = super::config_fixture::Fixture::new();
    for (width, side, wrap) in [1, 4, 8, 32]
        .into_iter()
        .flat_map(|width| [Side::Left, Side::Right].map(move |side| (width, side)))
        .flat_map(|(width, side)| [true, false].map(move |wrap| (width, side, wrap)))
    {
        let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
        let mut buffers = Buffers::with_settings(rev::config::prepare(&root).snapshot).unwrap();
        let id = buffers.create(SOURCE).unwrap();
        if !wrap {
            buffers.get_mut(id).unwrap().toggle_soft_wrap();
        }
        let mut view = View::new(buffers.get(id).unwrap());
        view.gutter.apply(Change::Side(side));
        view.point.position = Scalar(2);
        view.left = width.max(2);
        let mut session = Session::new(buffers, view, size()).unwrap();
        let origin = if side == Side::Left { 6 } else { 0 };
        session.handle(super::focus::press(Position {
            column: origin,
            row: 0,
        }));
        session.handle(drag(origin + 3, 0));
        session.handle(release(origin + 4, 0));
        let end = if wrap {
            match width {
                1 => 3,
                4 => 2,
                _ => 1,
            }
        } else {
            5
        };
        assert_eq!(range(&session), Some(if wrap { 0..end } else { 2..end }));
        assert_eq!(session.view().point.position, Scalar(end));
        assert_eq!(session.view().left, if wrap { 0 } else { width.max(2) });
        assert_eq!(
            session.frame().unwrap().rows()[0].selection(),
            Some(
                origin
                    ..origin
                        + if wrap {
                            match width {
                                1 => 4,
                                4 => 4,
                                _ => 1,
                            }
                        } else {
                            3
                        }
            )
        );
        assert_eq!(bytes(&session), SOURCE.as_bytes());
        assert!(session.buffer().unwrap().history().is_empty());
    }
}

// CRLF/final-empty boundaries remain whole; blank fill and clipped wide-cell
// fragments cannot create endpoints or a selection from an empty document.
#[test]
fn source_edges() {
    for (source, start, end, expected) in [
        ("", (6, 0), (7, 0), None),
        ("e\u{301}", (7, 0), (6, 0), Some(0..2)),
        ("\r\n", (6, 1), (6, 0), Some(0..2)),
        ("界", (8, 0), (6, 0), Some(0..1)),
        ("a\n", (6, 1), (6, 0), Some(0..2)),
    ] {
        let mut session = editor(source);
        session.handle(super::focus::press(Position {
            column: start.0,
            row: start.1,
        }));
        session.handle(drag(end.0, end.1));
        session.handle(release(end.0, end.1));
        assert_eq!(range(&session), expected);
        assert_eq!(bytes(&session), source.as_bytes());
    }
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("ab界").unwrap();
    let view = View::new(buffers.get(id).unwrap());
    let mut session = Session::new(
        buffers,
        view,
        Size {
            width: 9,
            height: 5,
        },
    )
    .unwrap();
    session.handle(super::focus::press(Position { column: 6, row: 0 }));
    session.handle(drag(7, 0));
    session.handle(drag(8, 0));
    session.handle(release(8, 0));
    assert_eq!(range(&session), Some(0..1));
    assert_eq!(bytes(&session), "ab界".as_bytes());
}
