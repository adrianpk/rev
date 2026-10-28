use super::*;

use rev::config::{default_template, prepare};

// Only native session startup provisions. The resulting complete file is loaded
// through ordinary validation; repeated startup leaves its entry and bytes alone.
#[test]
fn first_start() {
    if isolated() {
        let root = Roots::from_env().unwrap().config_file();
        assert!(!root.exists());
        assert_eq!(Startup::from_env(), Startup::default());
        assert!(!root.exists());
        let mut session = launch(None);
        assert_eq!(session.echo(), "C-x C-c: exit");
        assert_eq!(std::fs::read(&root).unwrap(), default_template().as_bytes());
        assert_eq!(
            session.buffer().unwrap().policy(),
            &prepare(&root).snapshot.settings().editor
        );
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    ");
        assert_eq!(launch(None).echo(), "C-x C-c: exit");
        return;
    }
    for fallback in [false, true] {
        let fixture = Fixture::new();
        fixture.write("project/kept.txt", "kept");
        let mut command = process("startup::publication::first_start", &fixture);
        if fallback {
            command.env_remove("XDG_CONFIG_HOME");
        }
        success(command.output().unwrap());
        let relative = if fallback {
            "home/.config/rev/config.toml"
        } else {
            "user/rev/config.toml"
        };
        let root = fixture.0.join(relative);
        assert_eq!(std::fs::read(&root).unwrap(), default_template().as_bytes());
        assert_eq!(
            std::fs::read_dir(root.parent().unwrap()).unwrap().count(),
            1
        );
        for name in ["data", "state", "cache"] {
            assert!(!fixture.0.join(name).exists(), "{name}");
        }
    }
}

// Preserve the identity and bytes of every destination type. Even a dangling
// link counts as present; unreadable/invalid candidates retain whole defaults.
#[test]
fn existing_entries() {
    if isolated() {
        let expected = Startup::from_env();
        let session = launch(None);
        assert_eq!(
            session.echo().starts_with("Configuration: "),
            expected.diagnostic.is_some()
        );
        assert_eq!(
            session.buffer().unwrap().policy(),
            &expected.snapshot.settings().editor
        );
        assert_eq!(Startup::from_env(), expected);
        return;
    }
    for kind in [
        "valid",
        "empty",
        "invalid",
        "symlink",
        "dangling",
        "directory",
        "unreadable",
    ] {
        preserve_entry(kind);
    }
}

fn preserve_entry(kind: &str) {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let fixture = Fixture::new();
    fixture.write("project/kept.txt", "kept");
    let root = fixture.write("user/rev/config.toml", "editor.indent_width = 2");
    configure_entry(&fixture, kind);
    let before = std::fs::symlink_metadata(&root).unwrap();
    child("startup::publication::existing_entries", &fixture);
    let after = std::fs::symlink_metadata(&root).unwrap();
    assert_eq!(
        (
            before.ino(),
            before.mode(),
            before.mtime(),
            before.mtime_nsec()
        ),
        (after.ino(), after.mode(), after.mtime(), after.mtime_nsec()),
        "{kind}"
    );
    if kind == "unreadable" {
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    assert!(!fixture.0.join("user/rev/absent.toml").exists());
    if after.is_file() {
        let expected = match kind {
            "empty" => "",
            "invalid" => "editor.indent_width = 0",
            _ => "editor.indent_width = 2",
        };
        assert_eq!(std::fs::read_to_string(&root).unwrap(), expected);
    }
    if kind == "symlink" {
        assert_eq!(std::fs::read_link(&root).unwrap(), Path::new("winner.toml"));
        assert_eq!(
            std::fs::read_to_string(&root).unwrap(),
            "editor.indent_width = 6"
        );
    }
}

fn configure_entry(fixture: &Fixture, kind: &str) {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture.0.join("user/rev/config.toml");
    match kind {
        "empty" => std::fs::write(root, "").unwrap(),
        "invalid" => std::fs::write(root, "editor.indent_width = 0").unwrap(),
        "symlink" => {
            fixture.write("user/rev/winner.toml", "editor.indent_width = 6");
            std::fs::remove_file(&root).unwrap();
            std::os::unix::fs::symlink("winner.toml", root).unwrap();
        }
        "unreadable" => {
            std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o000)).unwrap()
        }
        "dangling" | "directory" => super::errors::broken(fixture, kind),
        "valid" => (),
        _ => unreachable!(),
    }
}

// Provisioning failures keep the full built-in snapshot, preserve project
// ingress and permit native editing, without creating data/state/cache roots.
#[test]
fn creation_failure() {
    if isolated() {
        let mut session = launch(Some(Path::new("kept.txt")));
        assert!(session.echo().starts_with("Configuration: "));
        assert!(session.echo().contains("provision configuration"));
        assert!(session.echo().contains("config.toml"));
        assert_eq!(
            session.buffer().unwrap().policy(),
            &rev::config::Editor::default()
        );
        assert!(!session.buffer().unwrap().has_bom());
        assert!(!session.buffer().unwrap().is_modified());
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    kept");
        return;
    }
    use std::os::unix::fs::PermissionsExt;
    for kind in ["parent file", "unwritable"] {
        let fixture = Fixture::new();
        fixture.write("project/kept.txt", "kept");
        let user = fixture.0.join("user");
        if kind == "parent file" {
            std::fs::write(&user, "kept parent").unwrap();
        } else {
            std::fs::create_dir(&user).unwrap();
            std::fs::set_permissions(&user, std::fs::Permissions::from_mode(0o500)).unwrap();
        }
        child("startup::publication::creation_failure", &fixture);
        if kind == "unwritable" {
            std::fs::set_permissions(&user, std::fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(std::fs::read_dir(&user).unwrap().count(), 0);
        } else {
            assert_eq!(std::fs::read_to_string(&user).unwrap(), "kept parent");
        }
        for name in ["home", "data", "state", "cache"] {
            assert!(!fixture.0.join(name).exists(), "{name}");
        }
    }
}
