use std::fs;
use std::os::unix::fs::symlink;

use rev::config::{Preference, SettingsEdit, prepare};

use crate::fixture::Fixture;

// Preferences resolve against the complete include graph; neither server
// selection nor environment inheritance depends on the bundled grammar catalog.
#[test]
fn preferences_and_environment() {
    let f = Fixture::new();
    let base = f.write("base.toml", "[lsp.preferred]\nelixir = 'extra'\n[lsp.servers.extra]\nlanguages = ['elixir']\ncommand = 'custom-server'\nenv = { KEEP = 'inherited', REPLACE = 'old' }\n");
    let root = f.write("config.toml", "include = ['base.toml']\n[lsp.servers.extra.env]\nREPLACE = 'literal $HOME $(false)'\n[lsp.preferred]\nrust = false\n");
    let startup = prepare(&root);
    assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
    let lsp = &startup.snapshot.settings().lsp;
    assert_eq!(lsp.selected("elixir").unwrap().0, "extra");
    assert!(lsp.selected("rust").is_none());
    assert_eq!(lsp.servers["extra"].env["KEEP"], "inherited");
    assert_eq!(
        lsp.servers["extra"].env["REPLACE"],
        "literal $HOME $(false)"
    );
    assert_eq!(
        startup.snapshot.origin(&["lsp", "preferred", "elixir"]),
        Some(base.as_path())
    );
    assert_eq!(startup.snapshot.root(), Some(root.as_path()));
    assert_eq!(
        rev::config::Lsp::default().selected("rust").unwrap().0,
        "rust"
    );
}

// Invalid choices and launch environment bounds reject the whole candidate.
// Diagnostic text must not repeat environment values.
#[test]
fn rejected_settings() {
    let f = Fixture::new();
    let oversized = "sensitive-value".repeat(600);
    let cases = [
        ("true", "lsp.preferred.rust = true".to_owned()),
        ("missing", "lsp.preferred.rust = 'unknown'".into()),
        ("incompatible", "lsp.preferred.elixir = 'rust'".into()),
        ("identity", "lsp.preferred.'Bad ID' = false".into()),
        ("type", "lsp.servers.rust.env = { KEY = 8675309 }".into()),
        (
            "name",
            "lsp.servers.rust.env = { 'A=B' = 'private' }".into(),
        ),
        ("nul", "lsp.servers.rust.env = { A = \"\\u0000\" }".into()),
        (
            "value",
            format!("lsp.servers.rust.env = {{ A = '{oversized}' }}"),
        ),
        (
            "entries",
            format!(
                "[lsp.servers.rust.env]\n{}",
                (0..65)
                    .map(|n| format!("K{n} = 'value'\n"))
                    .collect::<String>()
            ),
        ),
        (
            "total",
            format!(
                "[lsp.servers.rust.env]\n{}",
                (0..9)
                    .map(|n| format!("K{n} = '{}'\n", "x".repeat(8192)))
                    .collect::<String>()
            ),
        ),
    ];
    for (name, text) in cases {
        let root = f.write("config.toml", &text);
        let startup = prepare(&root);
        let diagnostic = startup.diagnostic.expect(name).to_string();
        assert!(!diagnostic.contains("sensitive-value"), "{name}");
        assert!(!diagnostic.contains("8675309"), "{name}: {diagnostic}");
        assert_eq!(
            startup.snapshot.settings(),
            &rev::config::Settings::default(),
            "{name}"
        );
        assert!(SettingsEdit::capture(&root).is_err(), "{name}");
    }
}

// Real atomic publication follows the selected root's target, preserving its
// symlink, comments, includes, unrelated settings and the included file's bytes.
#[test]
fn symlink_publication() {
    let f = Fixture::new();
    let base = f.write("base.toml", "# included\n[editor]\nindent_width = 3\n");
    let target = f.write("target.toml", "# root comment\ninclude = ['base.toml'] # source order\n[editor]\ninteraction_profile = 'modal'\n# keep this\nsoft_wrap = false\n");
    let root = f.0.join("config.toml");
    symlink("target.toml", &root).unwrap();
    let mut edit = SettingsEdit::capture(&root).unwrap();
    let mut server = rev::config::Lsp::default().servers["rust"].clone();
    server.languages = vec!["elixir".into()];
    server.command = "custom-server".into();
    server.env.insert("LITERAL".into(), "$HOME $(false)".into());
    edit.register("elixir-custom", &server);
    edit.prefer("elixir", &Preference::Server("elixir-custom".into()));
    edit.associate("*.ex", "elixir");
    let result = edit.publish(&f.0.join("state")).unwrap();
    assert!(result.warning.is_none());
    assert_eq!(
        fs::read_link(&root).unwrap(),
        std::path::Path::new("target.toml")
    );
    let text = fs::read_to_string(target).unwrap();
    for retained in [
        "# root comment",
        "include = ['base.toml'] # source order",
        "# keep this",
        "soft_wrap = false",
    ] {
        assert!(text.contains(retained), "{retained}: {text}");
    }
    assert_eq!(
        fs::read_to_string(base).unwrap(),
        "# included\n[editor]\nindent_width = 3\n"
    );
    assert_eq!(
        result.snapshot.settings().editor.interaction_profile,
        rev::config::InteractionProfile::Modal
    );
    assert_eq!(result.snapshot.settings().editor.indent_width, 3);
    assert!(!result.snapshot.settings().lsp.enabled);
    assert_eq!(
        result.snapshot.language(std::path::Path::new("source.ex")),
        "elixir"
    );
    assert_eq!(
        result.snapshot.grammar("elixir"),
        rev::config::Grammar::Plain
    );
    assert!(prepare(&root).diagnostic.is_none());
}

// Compare source bytes and entry identities, even when the effective settings
// stay the same. A concurrent writer never loses its new file or comment.
#[test]
fn source_conflicts() {
    for change in [
        "root-comment",
        "include-comment",
        "root-inode",
        "root-link",
        "include-link",
    ] {
        let f = Fixture::new();
        let base = f.write("base.toml", "# base\n");
        let target = f.write("target.toml", "include = ['base-link.toml']\n");
        let root = f.0.join("config.toml");
        symlink("target.toml", &root).unwrap();
        symlink("base.toml", f.0.join("base-link.toml")).unwrap();
        let mut edit = SettingsEdit::capture(&root).unwrap();
        edit.prefer("rust", &Preference::Native(false));
        match change {
            "root-comment" => {
                fs::write(&target, "include = ['base-link.toml'] # new\n").unwrap();
            }
            "include-comment" => {
                fs::write(&base, "# new\n").unwrap();
            }
            "root-inode" => {
                let next = f.write("replacement", "include = ['base-link.toml']\n");
                fs::rename(next, &target).unwrap();
            }
            "root-link" => {
                let next = f.0.join("new-link");
                symlink("target.toml", &next).unwrap();
                fs::rename(next, &root).unwrap();
            }
            "include-link" => {
                let next = f.0.join("new-link");
                symlink("base.toml", &next).unwrap();
                fs::rename(next, f.0.join("base-link.toml")).unwrap();
            }
            _ => unreachable!(),
        }
        let before = fs::read(&target).unwrap();
        assert!(
            edit.publish(&f.0.join("state"))
                .err()
                .unwrap()
                .to_string()
                .contains("conflict"),
            "{change}"
        );
        assert_eq!(fs::read(target).unwrap(), before, "{change}");
    }
}

// Absent roots can be created, but invalid choices fail before publication and
// a held OS lock refuses another manager without blocking the caller.
#[test]
fn publication_admission() {
    let f = Fixture::new();
    let root = f.0.join("new/config.toml");
    let state = f.0.join("state");
    let mut edit = SettingsEdit::capture(&root).unwrap();
    edit.prefer("rust", &Preference::Server("absent".into()));
    assert!(edit.publish(&state).is_err());
    assert!(!root.exists());
    let mut edit = SettingsEdit::capture(&root).unwrap();
    edit.prefer("rust", &Preference::Native(false));
    edit.publish(&state).unwrap();
    assert!(
        prepare(&root)
            .snapshot
            .settings()
            .lsp
            .selected("rust")
            .is_none()
    );
    let edit = SettingsEdit::capture(&root).unwrap();
    let lock = fs::OpenOptions::new()
        .write(true)
        .open(state.join("lsp/mutation.lock"))
        .unwrap();
    lock.try_lock().unwrap();
    assert!(
        edit.publish(&state)
            .err()
            .unwrap()
            .to_string()
            .contains("mutation lock")
    );
}

// Updating a profile preserves comments on replaced values and leaves the
// original representation of unchanged arrays and nested settings intact.
#[test]
fn profile_comments() {
    let f = Fixture::new();
    let root = f.write("config.toml", "[lsp.preferred]\nrust = 'rust' # choice\n[lsp.servers.rust]\ncommand = 'old' # executable\nargs = ['--stdio'] # literal argv\n[lsp.servers.rust.settings]\n# nested comment\ncheck = true\n");
    let mut edit = SettingsEdit::capture(&root).unwrap();
    let mut server = edit.snapshot().unwrap().settings().lsp.servers["rust"].clone();
    server.command = "new-server".into();
    edit.register("rust", &server);
    edit.prefer("rust", &Preference::Server("rust".into()));
    edit.publish(&f.0.join("state")).unwrap();
    let text = fs::read_to_string(root).unwrap();
    for retained in [
        "# executable",
        "args = ['--stdio'] # literal argv",
        "rust = 'rust' # choice",
        "[lsp.servers.rust.settings]\n# nested comment\ncheck = true",
    ] {
        assert!(text.contains(retained), "{retained}: {text}");
    }
}
