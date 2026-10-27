use rev::buffer::edit::Point;
use rev::config::prepare;

use super::*;

// A selected glyph suppresses its capture foreground, including a selection
// starting/ending inside one token. Leaving the region restores the capture.
#[test]
fn selected_syntax() {
    crossterm::style::force_color_output(true);
    let fixture = config_fixture::Fixture::new();
    let path = fixture.write("main.rs", "fn alpha() {}\n");
    let startup = prepare(&fixture.write("config.toml", "palette.syntax.function = '#012345'"));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    for (name, start, end, expected) in [
        (
            "linear keyword",
            1,
            2,
            "\x1b[38;2;157;124;216mf\x1b[39m\x1b[7mn\x1b[27m ",
        ),
        (
            "rectangle function",
            5,
            6,
            "\x1b[38;2;1;35;69mal\x1b[39m\x1b[7mp\x1b[38;2;1;35;69m\x1b[27mha",
        ),
    ] {
        let mark = Point {
            stamp: buffer.stamp(),
            position: Scalar(start),
        };
        buffer.activate_mark(mark).unwrap();
        if name.starts_with("rectangle") {
            buffer.toggle_rectangle(mark).unwrap();
        }
        let mut view = View::new(buffer);
        view.point.position = Scalar(end);
        let frame = Frame::project(
            buffer,
            &view,
            Size {
                width: 30,
                height: 4,
            },
            "",
        )
        .unwrap()
        .with_palette(&startup.snapshot.settings().palette);
        assert_eq!(frame.rows()[0].selection(), Some(start as u16..end as u16));
        assert!(!frame.rows()[0].syntax().is_empty());
        let mut terminal = Vec::new();
        frame.write(&mut terminal).unwrap();
        let output = String::from_utf8(terminal).unwrap();
        assert!(output.contains(expected), "{name}: {output:?}");
        assert_eq!(buffer.stamp(), frame.stamp());
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}
