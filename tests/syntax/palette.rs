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
// only that role's escape sequences, with identical source/cells/cursor/history.
#[test]
fn syntax_overrides() {
    crossterm::style::force_color_output(true);
    let fixture = Fixture::new();
    let source = "// note\nfn build(value: &i32) { let text = \"hello\"; let count = 42; }";
    let path = fixture.file("main.rs", source);
    let mut defaults = Buffers::new().unwrap();
    let id = defaults.visit(&path).unwrap();
    let baseline = rendered(&frame(defaults.get(id).unwrap()));
    for (role, channels) in [
        ("keyword", "157;124;216"),
        ("string", "158;206;106"),
        ("comment", "86;95;137"),
        ("type", "42;195;222"),
        ("function", "122;162;247"),
        ("number", "255;158;100"),
        ("punctuation", "137;221;255"),
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
        assert_eq!(
            actual,
            baseline.replace(&original, "\x1b[38;2;1;35;69m"),
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
