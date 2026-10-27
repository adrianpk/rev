use rev::buffer::edit::{History, Replacement};
use rev::config::prepare;
use rev::coordinates::Scalar;
use rev::file::Rename;
use rev::window::Windows;

use super::*;

fn rendered(frame: &Frame) -> String {
    let mut output = Vec::new();
    frame.write(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

// Real captures must resolve through every RGB leaf. Replacing one role changes
// only that role's cells, retaining other roles even when their RGB values match.
// Registry and standalone output, source, cursor and history remain identical.
#[test]
fn syntax_overrides() {
    crossterm::style::force_color_output(true);
    let fixture = Fixture::new();
    let source = "// note\nfn build(value: &i32) { let text = \"hello\"; let count = 42; }";
    let path = fixture.file("main.rs", source);
    let mut defaults = Buffers::new().unwrap();
    let id = defaults.visit(&path).unwrap();
    let baseline_frame = frame(defaults.get(id).unwrap());
    let baseline = rendered(&baseline_frame);
    for (role, style, channels) in [
        ("keyword", rev::syntax::Style::Keyword, "157;124;216"),
        ("string", rev::syntax::Style::String, "158;206;106"),
        ("comment", rev::syntax::Style::Comment, "86;95;137"),
        ("type", rev::syntax::Style::Type, "42;195;222"),
        ("function", rev::syntax::Style::Function, "122;162;247"),
        ("number", rev::syntax::Style::Number, "255;158;100"),
        (
            "punctuation",
            rev::syntax::Style::Punctuation,
            "137;221;255",
        ),
    ] {
        let original = format!("\x1b[38;2;{channels}m");
        assert!(
            baseline.contains(&original),
            "{role} has no rendered capture"
        );
        let config = fixture.file("config.toml", &format!("palette.syntax.{role} = '#012345'"));
        let startup = prepare(&config);
        assert!(startup.diagnostic.is_none());
        let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get(id).unwrap();
        let windows = Windows::new(View::new(buffer)).unwrap();
        let composition = Frame::compose(
            &buffers,
            &windows,
            Size {
                width: 80,
                height: 8,
            },
            "ready",
        )
        .unwrap();
        let actual = rendered(&composition);
        assert!(
            baseline_frame
                .rows()
                .iter()
                .flat_map(|row| row.syntax())
                .any(|paint| paint.style == style)
        );
        assert_eq!(
            super::surface::draw(&composition),
            recolored(&baseline_frame, style),
            "{role}"
        );
        assert_eq!(
            actual,
            rendered(&frame(buffer).with_palette(&startup.snapshot.settings().palette))
        );
        assert_eq!(composition.cursor(), frame(buffer).cursor());
        assert_eq!(composition.stamp(), buffer.stamp());
        assert_eq!(text(buffer), source);
        assert!(!buffer.is_modified());
        assert!(buffer.history().is_empty());
    }
}

// Path policy changes may discard/rebuild derived colors, never a dirty edit,
// mark, saved identity or retained undo/redo. Palette ownership stays global.
#[test]
fn mode_transitions() {
    crossterm::style::force_color_output(true);
    let fixture = Fixture::new();
    let source = "fn main() {}\n";
    let mut path = fixture.file("main.rs", source);
    let startup = prepare(&fixture.file("config.toml", "palette.syntax.keyword = '#012345'"));
    assert!(startup.diagnostic.is_none());
    let mut buffers = Buffers::with_settings(startup.snapshot.clone()).unwrap();
    let id = buffers.visit(&path).unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let saved = buffer.state();
    let edit = Replacement {
        source: buffer
            .span(source.chars().count()..source.chars().count())
            .unwrap(),
        text: "// changed\n".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    buffer.set_mark(Some(Scalar(3))).unwrap();
    buffer.refresh_syntax();
    let stamp = buffer.stamp();
    let state = buffer.state();
    let expected = text(buffer);
    for (name, mode) in [
        ("main.txt", "Text"),
        ("again.rs", "Rust"),
        ("again.txt", "Text"),
    ] {
        buffers
            .rename(Rename::capture(&path).unwrap(), name)
            .unwrap();
        path = fixture.path(name);
        let buffer = buffers.get(id).unwrap();
        let output = rendered(&frame(buffer).with_palette(&startup.snapshot.settings().palette));
        assert_eq!(buffer.mode(), mode);
        assert_eq!(output.contains("\x1b[38;2;1;35;69m"), mode == "Rust");
        assert_eq!(buffer.stamp(), stamp);
        assert_eq!(buffer.state(), state);
        assert_eq!(buffer.mark(), Some(Scalar(3)));
        assert_eq!(buffer.history().len(), 1);
        assert!(buffer.is_modified());
        assert_eq!(text(buffer), expected);
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    }
    let buffer = buffers.get_mut(id).unwrap();
    buffer.navigate(History::Undo, &mut [], None).unwrap();
    assert_eq!(text(buffer), source);
    assert_eq!(buffer.state(), saved);
    assert!(!buffer.is_modified());
    buffer.navigate(History::Redo, &mut [], None).unwrap();
    assert_eq!(text(buffer), expected);
    assert_eq!(buffer.state(), state);
    assert!(buffer.is_modified());
}

// Recolor only captured source cells; matching gutter RGB is an independent role.
fn recolored(
    frame: &Frame,
    style: rev::syntax::Style,
) -> std::collections::BTreeMap<(usize, usize), super::surface::Cell> {
    let mut cells = super::surface::draw(frame);
    for row in frame.rows() {
        for paint in row.syntax().iter().filter(|paint| paint.style == style) {
            for column in paint.columns.clone() {
                cells
                    .get_mut(&(usize::from(row.row()), usize::from(column)))
                    .unwrap()
                    .foreground = Some(rev::config::Rgb(1, 35, 69));
            }
        }
    }
    cells
}
