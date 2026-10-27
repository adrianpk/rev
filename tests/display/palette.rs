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
    for (name, start, end) in [("linear keyword", 1, 2), ("rectangle function", 5, 6)] {
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
        let cells = surface::draw(&frame);
        let palette = &startup.snapshot.settings().palette;
        assert_eq!(
            cells[&(0, start)].foreground,
            Some(palette.selection_text),
            "{name}"
        );
        assert_eq!(
            cells[&(0, start)].background,
            Some(palette.selection),
            "{name}"
        );
        let after = if end == 6 {
            palette.syntax.function
        } else {
            palette.foreground
        };
        assert_eq!(cells[&(0, end)].foreground, Some(after), "{name}");
        assert_eq!(
            cells[&(0, end)].background,
            Some(palette.background),
            "{name}"
        );
        assert_eq!(buffer.stamp(), frame.stamp());
        assert!(buffer.history().is_empty());
        assert!(!buffer.is_modified());
    }
}
