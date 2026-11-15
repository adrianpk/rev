use rev::config::{PairPalette, Rgb, prepare};

use super::{Fixture, settings};

// Missing pair colors follow the fully merged semantic palette, rather than
// factory RGBs captured before an include/root overlay changes those roles.
#[test]
fn fallback_after_merge() {
    let f = Fixture::new();
    f.write("base.toml", "palette.selection='#010203'\npalette.foreground='#040506'\npalette.warning='#070809'\npalette.syntax.function='#101112'\npalette.syntax.number='#131415'\npalette.syntax.keyword='#161718'");
    let root = f.write("config.toml", "include=['base.toml']\npalette.syntax.type='#191a1b'\npalette.syntax.string='#1c1d1e'\npalette.syntax.punctuation='#1f2021'\npalette.selection='#222324'");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none());
    assert_eq!(
        startup.snapshot.settings().palette.pairs,
        PairPalette {
            match_background: Rgb(0x22, 0x23, 0x24),
            match_text: Rgb(4, 5, 6),
            mismatch: Rgb(7, 8, 9),
            depth_colors: vec![
                Rgb(0x10, 0x11, 0x12),
                Rgb(0x13, 0x14, 0x15),
                Rgb(0x16, 0x17, 0x18),
                Rgb(0x19, 0x1a, 0x1b),
                Rgb(0x1c, 0x1d, 0x1e),
                Rgb(0x1f, 0x20, 0x21)
            ],
        }
    );
    assert_eq!(
        startup
            .snapshot
            .origin(&["palette", "pairs", "match_background"]),
        None
    );
}

// Explicit roles survive unrelated palette overrides; a later depth array
// replaces the cycle as one leaf with independent provenance and admitted RGBs.
#[test]
fn explicit_roles_and_origins() {
    let f = Fixture::new();
    let base = f.write("base.toml", "palette.pairs.match_background='#abcdef'\npalette.pairs.match_text='#123456'\npalette.pairs.mismatch='#654321'\npalette.pairs.depth_colors=['#ffffff','#000000']");
    for count in [1, 8] {
        let colors = vec!["'#AbCdEf'"; count].join(",");
        let root = f.write("config.toml", &format!("include=['base.toml']\npalette.foreground='#000001'\npalette.pairs.depth_colors=[{colors}]"));
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_none());
        let palette = &startup.snapshot.settings().palette.pairs;
        assert_eq!(palette.match_background, Rgb(0xab, 0xcd, 0xef));
        assert_eq!(palette.match_text, Rgb(0x12, 0x34, 0x56));
        assert_eq!(palette.mismatch, Rgb(0x65, 0x43, 0x21));
        assert_eq!(palette.depth_colors, vec![Rgb(0xab, 0xcd, 0xef); count]);
        for leaf in ["match_background", "match_text", "mismatch"] {
            assert_eq!(
                startup.snapshot.origin(&["palette", "pairs", leaf]),
                Some(base.as_path())
            );
        }
        assert_eq!(
            startup
                .snapshot
                .origin(&["palette", "pairs", "depth_colors"]),
            Some(root.as_path())
        );
    }
    let palette = settings("").settings().palette.clone();
    assert_eq!(palette.pairs.match_background, palette.selection);
    assert_eq!(palette.pairs.match_text, palette.foreground);
    assert_eq!(palette.pairs.mismatch, palette.warning);
    assert_eq!(palette.pairs.depth_colors.len(), 6);
}

// Invalid colors, array shape and cardinality are candidate errors, including
// an invalid include masked by valid root roles. No partial UI settings escape.
#[test]
fn malformed_palette() {
    let f = Fixture::new();
    for (name, leaf) in [
        ("empty", "depth_colors=[]"),
        (
            "too many",
            "depth_colors=['#000000','#000000','#000000','#000000','#000000','#000000','#000000','#000000','#000000']",
        ),
        ("array string", "depth_colors='#ffffff'"),
        ("array boolean", "depth_colors=[true]"),
        ("invalid entry", "depth_colors=['#ffffff','#ffff']"),
        ("alpha", "match_background='#11223344'"),
        ("missing hash", "match_text='112233'"),
        ("nonhex", "mismatch='#gggggg'"),
        ("wrong type", "match_text=false"),
        ("unknown", "unknown='#ffffff'"),
    ] {
        f.write(
            "bad.toml",
            &format!("editor.tab_width=8\npalette.pairs.{leaf}"),
        );
        let root=f.write("config.toml", "include=['bad.toml']\npalette.pairs.depth_colors=['#ffffff']\npalette.pairs.match_text='#ffffff'");
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(
            startup.snapshot.settings(),
            rev::config::Snapshot::default().settings(),
            "{name}"
        );
    }
}

// An empty sequence must not deserialize as an all-absent palette struct.
#[test]
fn palette_table_shape() {
    for source in [
        "palette.pairs=[]",
        "palette.pairs=false",
        "palette.pairs='off'",
    ] {
        assert!(
            rev::config::parse(std::path::Path::new("invalid.toml"), source.as_bytes()).is_err()
        );
    }
}
