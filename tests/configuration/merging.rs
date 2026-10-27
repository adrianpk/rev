use std::path::Path;

use rev::config::{Document, IndentStyle, parse};

fn document(text: &str) -> Document {
    parse(Path::new("layer.toml"), text.as_bytes()).unwrap()
}

// Partial concern and language tables merge independently by leaf; a later
// global setting cannot reset an earlier, explicit language override.
#[test]
fn partial_tables() {
    let mut base = document(
        "editor.indent_style = 'tabs'\neditor.tab_width = 8\nlanguage.go.indent_width = 2\npalette.foreground = '#ffffff'",
    );
    let later = document(
        "editor.indent_width = 6\nlanguage.go.tab_width = 3\npalette.background = '#000000'",
    );
    let paths = base.overlay(&later);
    assert_eq!(base.editor.indent_style, Some(IndentStyle::Tabs));
    assert_eq!(base.editor.indent_width, Some(6));
    assert_eq!(base.editor.tab_width, Some(8));
    assert_eq!(base.language["go"].indent_width, Some(2));
    assert_eq!(base.language["go"].tab_width, Some(3));
    assert_eq!(base.language["go"].indent_style, None);
    assert_eq!(base.palette.foreground.as_deref(), Some("#ffffff"));
    assert_eq!(base.palette.background.as_deref(), Some("#000000"));
    assert!(paths.contains(&vec!["language".into(), "go".into(), "tab_width".into()]));
}

// Presence drives replacement: false, zero and an empty array are assignments.
// An omitted array inherits; arrays never concatenate during partial merging.
#[test]
fn present_values() {
    let cases = [
        ("absent", "", true, 64, vec!["base.toml"]),
        (
            "explicit",
            "include = []\nsyntax.enabled = false\nsyntax.max_bytes = 0",
            false,
            0,
            vec![],
        ),
        (
            "replacement",
            "include = ['last.toml']",
            true,
            64,
            vec!["last.toml"],
        ),
    ];
    for (name, text, enabled, max_bytes, includes) in cases {
        let mut base =
            document("include = ['base.toml']\nsyntax.enabled = true\nsyntax.max_bytes = 64");
        base.overlay(&document(text));
        assert_eq!(base.syntax.enabled, Some(enabled), "{name}");
        assert_eq!(base.syntax.max_bytes, Some(max_bytes), "{name}");
        assert_eq!(base.include.unwrap(), includes, "{name}");
    }
}

// Association and context tables replace matching leaves, preserve siblings
// and retain dynamic keys as single path components, even when they contain dots.
#[test]
fn mapping_leaves() {
    let mut base = document(
        "[files.associations]\n'*.h' = 'c'\n'*.cpp' = 'cpp'\n[keys.editor]\n'M-j' = 'next-line'\n'C-x C-s' = 'save-buffer'",
    );
    let later =
        document("[files.associations]\n'*.h' = 'cpp'\n[keys.editor]\n'M-j' = 'previous-line'");
    let paths = base.overlay(&later);
    assert_eq!(base.files.associations["*.h"], "cpp");
    assert_eq!(base.files.associations["*.cpp"], "cpp");
    assert_eq!(base.keys.editor["M-j"], "previous-line");
    assert_eq!(base.keys.editor["C-x C-s"], "save-buffer");
    assert!(paths.contains(&vec!["files".into(), "associations".into(), "*.h".into()]));
    base.overlay(&document("[keys.editor]"));
    assert_eq!(base.keys.editor.len(), 2);
}
