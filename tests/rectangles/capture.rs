use rev::config::prepare;

use super::*;
use crate::config_fixture::Fixture;

fn configured(width: usize, source: &str, a: Cell, b: Cell, readonly: bool) -> Session {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", &format!("editor.tab_width = {width}"));
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let coordinates = Coordinates::new(buffer).unwrap();
    let start = coordinates.from_cell(a).unwrap();
    let end = coordinates.from_cell(b).unwrap();
    buffer
        .toggle_rectangle(Point {
            stamp: buffer.stamp(),
            position: start,
        })
        .unwrap();
    buffer.set_readonly(readonly);
    let mut view = View::new(buffer);
    view.point.position = end;
    Session::new(
        buffers,
        view,
        Size {
            width: 48,
            height: 8,
        },
    )
    .unwrap()
}

// Copy uses the source policy at each admitted representative width. Short rows
// gain no right padding, CRLF remains source-only, and touched wide/combining/ZWJ
// clusters remain complete even when the right edge cuts through their cells.
#[test]
fn source_policy() {
    let anchor = "0".repeat(40);
    let source = format!("{anchor}\r\na\t界e\u{301}👩\u{200d}💻Z\r\nx\r\nzz\r\n\r\n{anchor}");
    for (width, stop) in [(1, 2), (2, 2), (4, 4), (8, 8), (32, 32)] {
        let mut session = configured(width, &source, cell(0, 1), cell(5, stop + 4), true);
        let before = (
            session.buffer().unwrap().stamp(),
            session.buffer().unwrap().state(),
            session.view().point,
            session.buffer().unwrap().mark(),
        );
        copy(&mut session);
        let edge = "0".repeat(stop + 3);
        assert_eq!(
            session.rectangle().unwrap().rows(),
            [
                edge.clone(),
                format!("{}界e\u{301}👩\u{200d}💻", " ".repeat(stop - 1)),
                String::new(),
                "z".into(),
                String::new(),
                edge
            ]
        );
        assert_eq!(session.rectangle().unwrap().width(), stop + 3);
        assert_eq!(session.buffer().unwrap().policy().tab_width, width);
        assert_eq!(text(&session), source);
        assert_eq!(
            (
                session.buffer().unwrap().stamp(),
                session.buffer().unwrap().state(),
                session.view().point,
                session.buffer().unwrap().mark()
            ),
            before
        );
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(!session.buffer().unwrap().is_modified());
        assert_eq!(session.buffer().unwrap().newline(), "\r\n");
        let frame = session.frame().unwrap();
        assert_eq!(
            frame.rows()[1].selection(),
            Some(1..u16::try_from(stop + 5).unwrap())
        );
        assert!(frame.rows()[2].selection().is_none());
        assert!(frame.rows()[4].selection().is_none());
    }
}
