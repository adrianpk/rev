use super::*;

use rev::config::Snapshot;

// A missing implicit source is silent, even beside malformed project sources.
// Startup creates the complete user root; editing leaves all other roots absent.
#[test]
fn missing_source() {
    if isolated() {
        let mut session = launch(None);
        assert_eq!(session.echo(), "C-x C-c: exit");
        assert!(!session.tree().visible());
        assert!(!session.buffer().unwrap().has_bom());
        assert_eq!(
            session.buffer().unwrap().policy(),
            &rev::config::Editor::default()
        );
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    ");
        return;
    }
    let fixture = Fixture::new();
    fixture.write("project/config.toml", "invalid project configuration");
    fixture.write("project/.rev/config.toml", "invalid hidden configuration");
    child("startup::errors::missing_source", &fixture);
    assert_eq!(
        std::fs::read(fixture.0.join("user/rev/config.toml")).unwrap(),
        rev::config::default_template().as_bytes()
    );
    assert_eq!(
        std::fs::read_dir(fixture.0.join("user/rev"))
            .unwrap()
            .count(),
        1
    );
    for name in ["home", "data", "state", "cache"] {
        assert!(!fixture.0.join(name).exists(), "{name}");
    }
}

// Present invalid leaves reject the whole candidate. The displayed source/key
// survives file ingress, while format, palette, key and editing policies revert.
#[test]
fn rejected_candidate() {
    if isolated() {
        rejected();
        return;
    }
    for invalid in [
        "editor.tab_width = 0",
        "editor.tab_width = 'wide'",
        "keys.editor.M-q = 'absent-command'",
    ] {
        let fixture = Fixture::new();
        let config = format!(
            "{invalid}\neditor.indent_width = 2\nfiles.bom = true\nfiles.newline = 'crlf'\npalette.foreground = '#010203'\nkeys.editor.M-j = 'newline'"
        );
        let root = fixture.write("user/rev/config.toml", &config);
        fixture.write("project/initial.txt", "kept");
        child("startup::errors::rejected_candidate", &fixture);
        assert_eq!(std::fs::read_to_string(root).unwrap(), config);
    }
}

fn rejected() {
    let roots = Roots::from_env().unwrap();
    let startup = Startup::from_env();
    assert_eq!(startup.snapshot, Snapshot::default());
    let diagnostic = startup.diagnostic.unwrap();
    assert_eq!(diagnostic.source, roots.config_file());
    assert!(!diagnostic.key.is_empty());
    let mut session = launch(Some(Path::new("initial.txt")));
    assert!(session.echo().starts_with("Configuration: "));
    assert!(session.echo().contains("config.toml"));
    assert!(session.echo().contains(&diagnostic.key.join(".")));
    assert!(
        session
            .frame()
            .unwrap()
            .rows()
            .last()
            .unwrap()
            .text()
            .starts_with("Configuration: ")
    );
    assert_eq!(
        session.buffer().unwrap().policy(),
        &rev::config::Editor::default()
    );
    assert!(!session.buffer().unwrap().has_bom());
    assert!(!session.buffer().unwrap().is_modified());
    session.handle(meta('j'));
    assert_eq!(text(&session), "kept");
    session.handle(key(KeyCode::Tab));
    assert_eq!(text(&session), "    kept");
}

// Existing broken roots and explicit missing includes produce diagnostics and
// usable built-ins. Source inspection never repairs or replaces those entries.
#[test]
fn unreadable_sources() {
    if isolated() {
        let mut session = launch(None);
        assert!(session.echo().starts_with("Configuration: "));
        assert!(session.echo().contains("config.toml"));
        assert_eq!(Startup::from_env().snapshot, Snapshot::default());
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    ");
        return;
    }
    for kind in ["directory", "dangling", "missing include", "invalid utf8"] {
        let fixture = Fixture::new();
        fixture.write("project/local.txt", "kept");
        let root = fixture.0.join("user/rev/config.toml");
        broken(&fixture, kind);
        child("startup::errors::unreadable_sources", &fixture);
        assert!(std::fs::symlink_metadata(root).is_ok(), "{kind}");
        assert!(!fixture.0.join("user/rev/absent.toml").exists());
    }
}

pub(super) fn broken(fixture: &Fixture, kind: &str) {
    let root = fixture.write("user/rev/config.toml", "");
    match kind {
        "directory" => {
            std::fs::remove_file(&root).unwrap();
            std::fs::create_dir(&root).unwrap();
        }
        "dangling" => {
            std::fs::remove_file(&root).unwrap();
            std::os::unix::fs::symlink("absent.toml", root).unwrap();
        }
        "missing include" => {
            fixture.write("user/rev/config.toml", "include = ['absent.toml']");
        }
        "invalid utf8" => std::fs::write(root, b"\xff").unwrap(),
        _ => unreachable!(),
    }
}

// Unresolvable environment roots cannot redirect discovery to CWD or abort the
// editor. The environment diagnostic identifies the unresolved XDG variable.
#[test]
fn unresolved_environment() {
    if isolated() {
        let mut session = launch(None);
        assert!(session.echo().contains("environment"));
        assert!(session.echo().contains("XDG_CONFIG_HOME"));
        assert_eq!(Startup::from_env().snapshot, Snapshot::default());
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    ");
        return;
    }
    let fixture = Fixture::new();
    fixture.write(
        "project/relative/rev/config.toml",
        "editor.indent_width = 2",
    );
    success(
        process("startup::errors::unresolved_environment", &fixture)
            .env_remove("HOME")
            .env("XDG_CONFIG_HOME", "relative")
            .output()
            .unwrap(),
    );
    assert!(!fixture.0.join("home").exists());
}
