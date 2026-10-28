use rev::buffer::Mode;
use rev::buffer::edit::Point;
use rev::org::{Cancellation, ColorScan, Limits, Role};
use rev::syntax::Style;

use super::*;

fn prepared(source: &str) -> (Buffers, rev::buffer::BufferId) {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(source).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    buffer.select_mode(Mode::Org).unwrap();
    let colors = ColorScan::new(buffer, Limits::default())
        .unwrap()
        .run(&Cancellation::default());
    buffer.install_org_coloring(colors).unwrap();
    (buffers, id)
}

// Source byte captures project complete wide/combining glyphs and logical tab
// stops through clipped views. Blank half-glyph edge cells remain unstyled.
#[test]
fn clipped_source() {
    let (buffers, id) = prepared("* 界e\u{301}\ttext");
    let buffer = buffers.get(id).unwrap();
    for (name, left, width, expected, paints) in [
        ("whole", 0, 5, "* 界e\u{301}", 0..5),
        ("wide edge", 3, 4, " e\u{301}  ", 1..2),
        ("tab", 6, 4, "  te", 2..4),
    ] {
        let mut view = View::new(buffer);
        view.left = left;
        let frame = Frame::project(buffer, &view, Size { width, height: 3 }, "").unwrap();
        assert_eq!(frame.rows()[0].text(), expected, "{name}");
        let syntax = frame.rows()[0].syntax();
        assert_eq!(syntax.len(), 1, "{name}");
        assert_eq!(syntax[0].columns, paints, "{name}");
        assert_eq!(syntax[0].style, Style::Org(Role::Heading), "{name}");
    }
}

// Region foreground wins over the underlying Org role, including a point inside
// the same token. Leaving the selection restores that token's semantic color.
#[test]
fn selected_org() {
    let (mut buffers, id) = prepared("* Heading\n");
    let buffer = buffers.get_mut(id).unwrap();
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(2),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(4);
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 30,
            height: 4,
        },
        "",
    )
    .unwrap();
    let palette = rev::config::Palette::default();
    let cells = surface::draw(&frame);
    assert_eq!(frame.rows()[0].selection(), Some(8..10));
    assert_eq!(cells[&(0, 8)].foreground, Some(palette.selection_text));
    assert_eq!(cells[&(0, 8)].background, Some(palette.selection));
    assert_eq!(cells[&(0, 10)].foreground, Some(palette.org.heading));
    assert_eq!(cells[&(0, 10)].background, Some(palette.background));
    assert!(buffer.history().is_empty());
}

// NO_COLOR is tested in an isolated child so environment/terminal color caches
// cannot race the other renderer tests. Source and reverse-video selection stay
// usable; RGB sequences are absent while current semantic row data is retained.
#[test]
fn no_color_source() {
    if std::env::var_os("REV_ORG_COLOR_CHILD").is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "org_colors::no_color_source", "--nocapture"])
            .env("REV_ORG_COLOR_CHILD", "1")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let source = "* 界e\u{301} *bold*\n#+begin_example\n,* Fake\n#+end_example";
    let (mut buffers, id) = prepared(source);
    let buffer = buffers.get_mut(id).unwrap();
    buffer
        .activate_mark(Point {
            stamp: buffer.stamp(),
            position: Scalar(2),
        })
        .unwrap();
    let mut view = View::new(buffer);
    view.point.position = Scalar(5);
    let frame = Frame::project(
        buffer,
        &view,
        Size {
            width: 50,
            height: 7,
        },
        "",
    )
    .unwrap();
    assert!(!frame.rows()[0].syntax().is_empty());
    let mut bytes = Vec::new();
    frame.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
    assert!(output.contains("\x1b[7m"));
    assert!(output.contains(",* Fake"));
    assert!(output.contains("#+begin_example"));
    assert_eq!(
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string(),
        source
    );
    assert!(buffer.history().is_empty());
    let mut session = rev::session::Session::new(
        buffers,
        view,
        Size {
            width: 50,
            height: 7,
        },
    )
    .unwrap();
    session.execute(rev::command::Command::SearchForward);
    session.handle(crossterm::event::Event::Paste("Fake".into()));
    let searched = session.frame().unwrap();
    assert!(searched.rows().iter().any(|row| row.search().is_some()));
    let mut bytes = Vec::new();
    searched.write(&mut bytes).unwrap();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[4m"));
    assert!(!output.contains("38;2;") && !output.contains("48;2;"));
}
