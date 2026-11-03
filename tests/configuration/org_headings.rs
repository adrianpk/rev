use super::fixture::Fixture;
use rev::config::{Headings, Settings, prepare};

// Includes replace the complete marker array while omitted siblings retain
// their actual assigning source, including explicit false presentation flags.
#[test]
fn heading_layers() {
    let f = Fixture::new();
    let base = f.write("base.toml", "[org.headings]\npretty=false\nbullets=['a','b']\nhide_leading_stars=false\ncycle_bullets=false");
    let root = f.write(
        "config.toml",
        "include=['base.toml']\norg.headings.bullets=['+']",
    );
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    assert_eq!(
        startup.snapshot.settings().org.headings,
        Headings {
            pretty: false,
            bullets: vec!["+".into()],
            hide_leading_stars: false,
            cycle_bullets: false,
        }
    );
    for (key, source) in [
        ("pretty", &base),
        ("bullets", &root),
        ("hide_leading_stars", &base),
        ("cycle_bullets", &base),
    ] {
        assert_eq!(
            startup.snapshot.origin(&["org", "headings", key]),
            Some(source.as_path()),
            "{key}"
        );
    }
    assert_eq!(Headings::default().bullets, ["◉", "○", "✸", "✿", "◆", "▶"]);
    assert!(Headings::default().pretty);
    assert!(Headings::default().hide_leading_stars);
    assert!(Headings::default().cycle_bullets);
}

// Glyph admission protects cell geometry even when presentation is disabled.
// Failed includes reject the whole candidate rather than leaking valid siblings.
#[test]
fn marker_admission() {
    let f = Fixture::new();
    let max_marker = format!("é{}", "\u{301}".repeat(15));
    let cases = [
        ("single", vec!["+".into()], true),
        ("combined", vec!["e\u{301}".into()], true),
        ("private use", vec!["\u{e0b0}".into()], true),
        ("payload bound", vec![max_marker; 32], true),
        ("empty list", vec![], false),
        ("excess list", vec!["+".into(); 33], false),
        ("empty marker", vec!["".into()], false),
        ("space", vec![" ".into()], false),
        ("tab", vec!["\t".into()], false),
        ("escape", vec!["\u{1b}[m".into()], false),
        ("zero width", vec!["\u{301}".into()], false),
        ("wide", vec!["界".into()], false),
        ("emoji", vec!["😀".into()], false),
        ("two graphemes", vec!["ab".into()], false),
        (
            "excess bytes",
            vec![format!("a{}", "\u{301}".repeat(16))],
            false,
        ),
    ];
    for (name, bullets, valid) in cases {
        let array = serde_json::to_string(&bullets).unwrap();
        f.write(
            "base.toml",
            &format!("editor.tab_width=3\norg.headings.pretty=false\norg.headings.bullets={array}"),
        );
        let root = f.write(
            "config.toml",
            "include=['base.toml']\norg.headings.bullets=['+']",
        );
        let startup = prepare(&root);
        assert_eq!(
            startup.diagnostic.is_none(),
            valid,
            "{name}: {:?}",
            startup.diagnostic
        );
        if !valid {
            assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
            assert_eq!(
                startup.diagnostic.unwrap().key,
                ["org", "headings", "bullets"],
                "{name}"
            );
        }
    }
}
