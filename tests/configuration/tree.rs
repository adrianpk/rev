use rev::config::{Document, Rgb, Settings, TreePalette, prepare};

use super::fixture::Fixture;

fn roles(palette: &TreePalette) -> [(&'static str, Rgb); 7] {
    [
        ("root", palette.root),
        ("directory", palette.directory),
        ("source", palette.source),
        ("document", palette.document),
        ("configuration", palette.configuration),
        ("file", palette.file),
        ("symlink", palette.symlink),
    ]
}

// Omission adopts decoration without showing the tree or enabling font support.
// The independent default catalog protects all seven semantic colors.
#[test]
fn defaults() {
    let absent: Document = toml::from_str("").unwrap();
    assert_eq!(absent.tree.decorated, None);
    let settings = Settings::default();
    assert!(settings.tree.decorated);
    assert!(!settings.tree.visible);
    assert!(!settings.gutter.nerd_fonts);
    for ((name, actual), expected) in roles(&settings.palette.tree_entries).into_iter().zip([
        "#7aa2f7", "#e0af68", "#7aa2f7", "#e0af68", "#bb9af7", "#c0caf5", "#7dcfff",
    ]) {
        assert_eq!(Some(actual), Rgb::parse(expected), "{name}");
    }
}

// Both explicit booleans and inherited values preserve provenance; unrelated
// font, visibility, background and selection preferences remain independent.
#[test]
fn precedence() {
    for (name, inherited, explicit, expected) in [
        ("omitted", "", "", true),
        ("included false", "tree.decorated=false", "", false),
        (
            "root false",
            "tree.decorated=true",
            "tree.decorated=false",
            false,
        ),
        (
            "root true",
            "tree.decorated=false",
            "tree.decorated=true",
            true,
        ),
    ] {
        let fixture = Fixture::new();
        let base = fixture.write("base.toml", inherited);
        let text = format!("include=['base.toml']\ngutter.nerd_fonts=true\n{explicit}");
        let root = fixture.write("config.toml", &text);
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        let settings = startup.snapshot.settings();
        assert_eq!(settings.tree.decorated, expected, "{name}");
        assert!(!settings.tree.visible, "{name}");
        assert!(settings.gutter.nerd_fonts, "{name}");
        assert_eq!(settings.palette, Settings::default().palette, "{name}");
        let origin = if !explicit.is_empty() {
            Some(root.as_path())
        } else if !inherited.is_empty() {
            Some(base.as_path())
        } else {
            None
        };
        assert_eq!(
            startup.snapshot.origin(&["tree", "decorated"]),
            origin,
            "{name}"
        );
        assert_eq!(std::fs::read_to_string(root).unwrap(), text);
    }
}

// Every leaf participates in merge, projection and origin tracking, including
// preservation of siblings when only one root role overrides the include.
#[test]
fn palette_layers() {
    let fixture = Fixture::new();
    let names = roles(&TreePalette::default());
    let colors = names
        .iter()
        .map(|(name, _)| format!("palette.tree_entries.{name}='#010203'\n"))
        .collect::<String>();
    let base = fixture.write("base.toml", &colors);
    let root = fixture.write("config.toml", "include=['base.toml']\npalette.tree='#040506'\npalette.tree_selected='#070809'\npalette.tree_selected_text='#0a0b0c'\npalette.tree_entries.source='#102030'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let palette = &startup.snapshot.settings().palette;
    assert_eq!(
        (
            palette.tree,
            palette.tree_selected,
            palette.tree_selected_text
        ),
        (Rgb(4, 5, 6), Rgb(7, 8, 9), Rgb(10, 11, 12))
    );
    for (name, value) in roles(&palette.tree_entries) {
        let overridden = name == "source";
        assert_eq!(
            value,
            if overridden {
                Rgb(16, 32, 48)
            } else {
                Rgb(1, 2, 3)
            },
            "{name}"
        );
        assert_eq!(
            startup.snapshot.origin(&["palette", "tree_entries", name]),
            Some(if overridden {
                root.as_path()
            } else {
                base.as_path()
            }),
            "{name}"
        );
    }
}

// Invalid new leaves reject the whole candidate; valid root overrides cannot
// conceal a malformed include or publish unrelated settings from that graph.
#[test]
fn rejected_leaves() {
    let mut cases = ["1", "'false'", "[]", "{}"]
        .map(|value| {
            (
                "tree.decorated".to_owned(),
                value.to_owned(),
                "false".to_owned(),
            )
        })
        .to_vec();
    for (name, _) in roles(&TreePalette::default()) {
        for invalid in ["'#12345g'", "'#fff'", "1"] {
            cases.push((
                format!("palette.tree_entries.{name}"),
                invalid.into(),
                "'#123456'".into(),
            ));
        }
    }
    cases.push((
        "palette.tree_entries.unknown".into(),
        "'#123456'".into(),
        "'#123456'".into(),
    ));
    for (key, invalid, valid) in cases {
        let fixture = Fixture::new();
        let base = fixture.write("base.toml", &format!("editor.tab_width=8\n{key}={invalid}"));
        let override_text = if key == "palette.tree_entries.unknown" {
            "tree.decorated=false".to_owned()
        } else {
            format!("{key}={valid}")
        };
        let root = fixture.write(
            "config.toml",
            &format!("include=['base.toml']\n{override_text}"),
        );
        let startup = prepare(&root);
        let diagnostic = startup
            .diagnostic
            .unwrap_or_else(|| panic!("{key}={invalid}"));
        assert_eq!(diagnostic.source, base, "{key}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{key}");
    }
}
