use rev::config::{IndentStyle, Match, Newline, Settings, TreeRoot};

// Pin the accepted catalog independently of any distributed file. Preparatory
// defaults must not acquire language conventions or user mapping overrides.
#[test]
fn scalar_defaults() {
    let settings = Settings::default();
    assert_eq!(settings.editor.indent_style, IndentStyle::Spaces);
    assert_eq!(
        (settings.editor.indent_width, settings.editor.tab_width),
        (4, 4)
    );
    assert_eq!(settings.scroll.page_overlap, 2);
    assert_eq!(settings.windows.split_ratio, 0.5);
    assert_eq!(settings.minibuffer.r#match, Match::PrefixThenSubstring);
    assert_eq!(
        (
            settings.minibuffer.candidate_limit,
            settings.minibuffer.history_limit,
            settings.minibuffer.input_limit,
        ),
        (8, 128, 4096),
    );
    assert_eq!(
        (
            settings.history.group_limit,
            settings.history.payload_target_bytes
        ),
        (512, 67_108_864),
    );
    assert_eq!(
        (settings.kill_ring.capacity, settings.kill_ring.coalesce),
        (32, true)
    );
    assert_eq!(
        (settings.syntax.enabled, settings.syntax.max_bytes),
        (true, 0)
    );
    assert_eq!(
        (settings.files.newline, settings.files.bom),
        (Newline::Lf, false)
    );
    assert_eq!((settings.tree.visible, settings.tree.width), (false, 24));
    assert_eq!(settings.tree.max_fraction, 0.3333333333333333);
    assert_eq!(settings.tree.root, TreeRoot::Auto);
    assert!(settings.tree.show_hidden);
    assert!(settings.tree.directories_first);
    assert_eq!(settings.gutter, rev::gutter::Settings::default());
    assert!(settings.language.is_empty());
    assert!(settings.files.associations.is_empty());
    assert!(settings.keys.editor.is_empty());
    assert!(settings.keys.minibuffer.is_empty());
    assert!(settings.keys.tree.is_empty());
}

// Keep all 29 accepted semantic colors, including inherited tree text and the
// existing combined syntax capture groups, pinned to the accepted night palette.
#[test]
fn palette_defaults() {
    let palette = Settings::default().palette;
    let cases = [
        ("background", palette.background, "#1a1b26"),
        ("foreground", palette.foreground, "#c0caf5"),
        ("cursor", palette.cursor, "#c0caf5"),
        ("selection", palette.selection, "#283457"),
        ("selection_text", palette.selection_text, "#c0caf5"),
        ("search", palette.search, "#3d59a1"),
        ("search_text", palette.search_text, "#c0caf5"),
        ("status", palette.status, "#16161e"),
        ("status_text", palette.status_text, "#a9b1d6"),
        ("status_inactive", palette.status_inactive, "#1a1b26"),
        ("border", palette.border, "#3b4261"),
        ("border_active", palette.border_active, "#7aa2f7"),
        ("tree", palette.tree, "#16161e"),
        ("tree_selected", palette.tree_selected, "#283457"),
        ("tree_selected_text", palette.tree_selected_text, "#c0caf5"),
        ("message", palette.message, "#c0caf5"),
        ("warning", palette.warning, "#e0af68"),
        ("error", palette.error, "#db4b4b"),
        ("syntax.keyword", palette.syntax.keyword, "#9d7cd8"),
        ("syntax.string", palette.syntax.string, "#9ece6a"),
        ("syntax.comment", palette.syntax.comment, "#565f89"),
        ("syntax.type", palette.syntax.r#type, "#2ac3de"),
        ("syntax.function", palette.syntax.function, "#7aa2f7"),
        ("syntax.number", palette.syntax.number, "#ff9e64"),
        ("syntax.punctuation", palette.syntax.punctuation, "#89ddff"),
        ("gutter.background", palette.gutter.background, "#1a1b26"),
        ("gutter.foreground", palette.gutter.foreground, "#565f89"),
        (
            "gutter.current_line",
            palette.gutter.current_line,
            "#7aa2f7",
        ),
        ("gutter.separator", palette.gutter.separator, "#3b4261"),
    ];
    for (name, color, expected) in cases {
        let actual = format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2);
        assert_eq!(actual, expected, "{name}");
    }
}
