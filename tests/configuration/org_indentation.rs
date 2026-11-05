use super::fixture::Fixture;
use rev::buffer::{Buffers, Mode};
use rev::config::{Settings, prepare};

// The boolean composes by leaf, retains an explicit false and its actual source,
// and is copied into the frozen buffer policy without rereading the file.
#[test]
fn indentation_layers() {
    let f = Fixture::new();
    let base = f.write("base.toml", "org.indent=false\norg.headings.pretty=false");
    for (name, later, expected, assigned_by_root) in [
        ("inherit", "", false, false),
        ("explicit enable", "org.indent=true", true, true),
        ("explicit disable", "org.indent=false", false, true),
    ] {
        let root = f.write("config.toml", &format!("include=['base.toml']\n{later}"));
        let startup = prepare(&root);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        assert_eq!(startup.snapshot.settings().org.indent, expected, "{name}");
        assert!(!startup.snapshot.settings().org.headings.pretty);
        assert_eq!(
            startup.snapshot.origin(&["org", "indent"]),
            Some(if assigned_by_root {
                root.as_path()
            } else {
                base.as_path()
            })
        );
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        f.write("config.toml", "org.indent=false\norg.enabled=false");
        let id = buffers.create("* H\nbody").unwrap();
        let b = buffers.get_mut(id).unwrap();
        b.select_mode(Mode::Org).unwrap();
        assert_eq!(b.org_indentation(), expected, "{name}");
        b.select_mode(Mode::Text).unwrap();
        assert!(!b.org_indentation());
        assert!(b.history().is_empty());
        assert!(!b.is_modified());
    }
    assert!(Settings::default().org.indent);
}

// Invalid included types reject the whole candidate, even when a valid later
// layer would mask them; no valid sibling or language override can leak through.
#[test]
fn indentation_admission() {
    let f = Fixture::new();
    for (name, value) in [
        ("integer", "1"),
        ("string", "'false'"),
        ("array", "[]"),
        ("table", "{}"),
    ] {
        f.write(
            "base.toml",
            &format!("editor.tab_width=3\norg.indent={value}"),
        );
        let root = f.write("config.toml", "include=['base.toml']\norg.indent=true");
        let startup = prepare(&root);
        assert!(startup.diagnostic.is_some(), "{name}");
        assert_eq!(startup.snapshot.settings(), &Settings::default(), "{name}");
    }
    let root = f.write("config.toml", "language.org.indent=false");
    assert!(prepare(&root).diagnostic.is_some());
}
