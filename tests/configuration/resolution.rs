use rev::config::{Grammar, IndentStyle, Rgb, Sources, load, resolve};

use crate::fixture::Fixture;

// Nested includes precede their parents, later siblings win at the same scope,
// and root leaves retain unrelated built-ins and included palette siblings.
#[test]
fn source_precedence() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['first.toml', 'last.toml']\neditor.indent_width = 6\npalette.foreground = '#ffffff'");
    let first = fixture.write("first.toml", "include = ['leaf.toml']\neditor.indent_width = 2\neditor.tab_width = 8\npalette.background = '#000000'");
    fixture.write("leaf.toml", "editor.indent_width = 1\neditor.tab_width = 3");
    let last = fixture.write(
        "last.toml",
        "editor.indent_width = 4\nsyntax.enabled = false",
    );
    let snapshot = resolve(&load(&root).unwrap()).unwrap();
    let settings = snapshot.settings();
    assert_eq!(settings.editor.indent_width, 6);
    assert_eq!(settings.editor.tab_width, 8);
    assert!(!settings.syntax.enabled);
    assert_eq!(settings.palette.background, Rgb(0, 0, 0));
    assert_eq!(settings.palette.foreground, Rgb(255, 255, 255));
    assert_eq!(settings.history.group_limit, 512);
    assert_eq!(
        snapshot.origin(&["editor", "indent_width"]),
        Some(root.as_path())
    );
    assert_eq!(
        snapshot.origin(&["editor", "tab_width"]),
        Some(first.as_path())
    );
    assert_eq!(
        snapshot.origin(&["syntax", "enabled"]),
        Some(last.as_path())
    );
    assert_eq!(snapshot.origin(&["history", "group_limit"]), None);
}

// Global changes in the root are resolved before the merged language partial.
// An omitted language leaf inherits the final global value, never an early default.
#[test]
fn language_precedence() {
    let fixture = Fixture::new();
    let root = fixture.write("config.toml", "include = ['languages.toml']\neditor.indent_style = 'tabs'\neditor.indent_width = 6\neditor.tab_width = 8\nlanguage.rust.grammar = 'plain'");
    fixture.write("languages.toml", "language.go.indent_width = 2");
    let snapshot = resolve(&load(&root).unwrap()).unwrap();
    let go = snapshot.editor("go");
    assert_eq!(
        (go.indent_style, go.indent_width, go.tab_width),
        (IndentStyle::Tabs, 2, 8)
    );
    assert_eq!(snapshot.editor("unknown"), snapshot.settings().editor);
    assert_eq!(snapshot.grammar("go"), Grammar::Plain);
    assert_eq!(snapshot.grammar("rust"), Grammar::Plain);
    assert_eq!(
        resolve(&Sources::default()).unwrap().grammar("rust"),
        Grammar::Rust
    );
}
