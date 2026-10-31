use std::path::Path;

use rev::config::{Document, Mouse, PartialMouse, Settings, parse, prepare};

use super::fixture::Fixture;

// Explicit false must survive partial parsing and include layering without
// defaulting its absent sibling; both leaves retain their actual source origin.
#[test]
fn layered_controls() {
    assert_eq!(
        Settings::default().mouse,
        Mouse {
            enabled: true,
            scroll_lines: 3,
            tree_activate_on_click: true
        }
    );
    assert_eq!(Document::default().mouse, PartialMouse::default());
    let fixture = Fixture::new();
    let base = fixture.write(
        "base.toml",
        "mouse.enabled = true\nmouse.scroll_lines = 17\nmouse.tree_activate_on_click = true",
    );
    let root = fixture.write(
        "config.toml",
        "include = ['base.toml']\nmouse.enabled = false\nmouse.tree_activate_on_click = false",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        startup.snapshot.settings().mouse,
        Mouse {
            enabled: false,
            scroll_lines: 17,
            tree_activate_on_click: false
        }
    );
    assert_eq!(
        startup.snapshot.origin(&["mouse", "enabled"]),
        Some(root.as_path())
    );
    assert_eq!(
        startup.snapshot.origin(&["mouse", "scroll_lines"]),
        Some(base.as_path())
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["mouse", "tree_activate_on_click"]),
        Some(root.as_path())
    );
    let partial = parse(Path::new("mouse.toml"), b"mouse.enabled = false").unwrap();
    assert_eq!(partial.mouse.enabled, Some(false));
    assert_eq!(partial.mouse.scroll_lines, None);
    assert_eq!(partial.mouse.tree_activate_on_click, None);
}

// The fixed command is admitted through real context-map validation at both
// session owners, and bounded distances include both endpoints of 1..=100.
#[test]
fn admitted_controls() {
    for distance in [1, 3, 100] {
        let fixture = Fixture::new();
        let root = fixture.write("config.toml", &format!("mouse.scroll_lines = {distance}\n[keys.editor]\n'M-j' = 'mouse-mode'\n[keys.tree]\n'M-j' = 'mouse-mode'"));
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        assert_eq!(startup.snapshot.settings().mouse.scroll_lines, distance);
        assert!(startup.snapshot.settings().mouse.enabled);
    }
}

// Wrong types, out-of-range integers, unsupported leaves and language-local
// mouse tables reject the complete candidate with a source/key diagnostic.
#[test]
fn rejected_controls() {
    for (name, invalid) in [
        ("zero", "mouse.scroll_lines = 0"),
        ("negative", "mouse.scroll_lines = -1"),
        ("excess", "mouse.scroll_lines = 101"),
        ("float", "mouse.scroll_lines = 3.0"),
        ("boolean", "mouse.enabled = 'true'"),
        ("tree boolean", "mouse.tree_activate_on_click = 1"),
        (
            "language tree",
            "language.rust.mouse.tree_activate_on_click = false",
        ),
        ("unknown", "mouse.hover = true"),
        ("language", "language.rust.mouse.enabled = false"),
    ] {
        let fixture = Fixture::new();
        let root = fixture.write(
            "config.toml",
            &format!("editor.indent_width = 2\n{invalid}"),
        );
        let startup = prepare(&root);
        let diagnostic = startup.diagnostic.unwrap();
        assert_eq!(diagnostic.source, root, "{name}");
        assert!(!diagnostic.key.is_empty(), "{name}");
        assert!(diagnostic.position.is_some(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
    }
    let fixture = Fixture::new();
    fixture.write("bad.toml", "mouse.scroll_lines = 0");
    let root = fixture.write(
        "config.toml",
        "include = ['bad.toml']\nmouse.scroll_lines = 1",
    );
    assert!(prepare(&root).diagnostic.is_some());
}
