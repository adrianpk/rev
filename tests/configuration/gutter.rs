use rev::config::{Document, PartialGutter, Rgb, Settings, prepare};
use rev::gutter::{CurrentLine, Mode, Side};

use super::fixture::Fixture;

// Missing leaves remain absent, while explicit false/zero survive layering.
// Native enums are shared with the renderer rather than translated into strings.
#[test]
fn partial_leaves() {
    let absent: Document = toml::from_str("").unwrap();
    assert_eq!(absent.gutter, PartialGutter::default());
    let explicit: Document =
        toml::from_str("gutter.visible = false\ngutter.left_padding = 0\ngutter.mode = 'relative'")
            .unwrap();
    assert_eq!(explicit.gutter.visible, Some(false));
    assert_eq!(explicit.gutter.left_padding, Some(0));
    assert_eq!(explicit.gutter.mode, Some(Mode::Relative));
    assert_eq!(explicit.gutter.current_line, None);
    assert_eq!(explicit.gutter.minimum_digits, None);
}

// Root leaves override only their included siblings, including zero/false.
// Every gutter/color leaf resolves through the same snapshot and provenance map.
#[test]
fn layered_gutter() {
    let fixture = Fixture::new();
    let base = fixture.write("base.toml", "[gutter]\nvisible = true\nmode = 'relative'\ncurrent_line = 'zero'\nside = 'right'\nminimum_digits = 6\nleft_padding = 2\nright_padding = 3\n[palette.gutter]\nbackground = '#010203'\nforeground = '#040506'\ncurrent_line = '#070809'\nseparator = '#0a0b0c'");
    let root = fixture.write("config.toml", "include = ['base.toml']\ngutter.visible = false\ngutter.left_padding = 0\n[palette.gutter]\nseparator = '#102030'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let settings = startup.snapshot.settings();
    assert!(!settings.gutter.visible);
    assert_eq!(settings.gutter.mode, Mode::Relative);
    assert_eq!(settings.gutter.current_line, CurrentLine::Zero);
    assert_eq!(settings.gutter.side, Side::Right);
    let format = settings.gutter.format;
    assert_eq!(
        (
            format.minimum_digits(),
            format.left_padding(),
            format.right_padding()
        ),
        (6, 0, 3)
    );
    let colors = &settings.palette.gutter;
    assert_eq!(
        (
            colors.background,
            colors.foreground,
            colors.current_line,
            colors.separator
        ),
        (Rgb(1, 2, 3), Rgb(4, 5, 6), Rgb(7, 8, 9), Rgb(16, 32, 48))
    );
    for key in [
        "mode",
        "current_line",
        "side",
        "minimum_digits",
        "right_padding",
    ] {
        assert_eq!(
            startup.snapshot.origin(&["gutter", key]),
            Some(base.as_path())
        );
    }
    for key in ["visible", "left_padding"] {
        assert_eq!(
            startup.snapshot.origin(&["gutter", key]),
            Some(root.as_path())
        );
    }
    assert_eq!(
        startup
            .snapshot
            .origin(&["palette", "gutter", "foreground"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["palette", "gutter", "separator"]),
        Some(root.as_path())
    );
}

// Bad types, enums, bounds, colors and unknown leaves reject the whole candidate,
// preserving complete built-ins even when unrelated valid settings were supplied.
#[test]
fn invalid_candidates() {
    for (name, invalid) in [
        ("zero digits", "gutter.minimum_digits = 0"),
        ("excess digits", "gutter.minimum_digits = 21"),
        ("negative digits", "gutter.minimum_digits = -1"),
        ("negative left", "gutter.left_padding = -1"),
        ("excess right", "gutter.right_padding = 5"),
        ("float digits", "gutter.minimum_digits = 4.0"),
        ("wrong boolean", "gutter.visible = 1"),
        ("mode", "gutter.mode = 'visual'"),
        ("current", "gutter.current_line = 'relative'"),
        ("side", "gutter.side = 'center'"),
        ("color", "palette.gutter.current_line = '#12345g'"),
        ("unknown color", "palette.gutter.future = '#123456'"),
        ("unknown gutter", "gutter.separator = '|'"),
        ("language gutter", "language.rust.gutter.visible = false"),
    ] {
        let fixture = Fixture::new();
        let text = format!("editor.tab_width = 8\n{invalid}");
        let root = fixture.write("config.toml", &text);
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
        assert_eq!(std::fs::read_to_string(&root).unwrap(), text);
    }
}

// A valid parent cannot hide invalid included data. The existing diagnostic
// identifies the original source/leaf; no projected settings escape admission.
#[test]
fn included_rejection() {
    let fixture = Fixture::new();
    let base = fixture.write("base.toml", "gutter.minimum_digits = 21");
    let root = fixture.write(
        "config.toml",
        "include = ['base.toml']\ngutter.minimum_digits = 4",
    );
    let startup = prepare(&root);
    let diagnostic = startup.diagnostic.unwrap();
    assert_eq!(diagnostic.source, base);
    assert_eq!(diagnostic.key, ["gutter", "minimum_digits"]);
    assert_eq!(startup.snapshot.settings(), &Settings::default());
}
