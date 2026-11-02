use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use rev::command::{Command, LspCommand};
use rev::config::prepare;
use rev::session::Session;

use crate::fixture::Fixture;
use crate::workflows::{answer, key, named, session, text};

fn cancel(s: &mut Session, modal: bool) {
    if modal {
        key(s, KeyCode::Esc, KeyModifiers::NONE);
        s.expire_input(Instant::now() + Duration::from_secs(1));
    } else {
        key(s, KeyCode::Char('g'), KeyModifiers::CONTROL);
    }
}

// Actual M-x/colon selectors run in owned child environments. Recipe shortcuts
// inherit user options and associations, while local observations and selection
// never execute helpers or parse an unrelated/corrupt installation registry.
#[test]
fn catalog_workflow() {
    let Some(profile) = std::env::var_os("REV_CATALOG_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "selectors::catalog_workflow", "--nocapture"])
                .env("REV_CATALOG_PROFILE", profile)
                .env("REV_CATALOG_ROOT", &f.0)
                .env("PATH", f.0.join("bin"))
                .env("XDG_CONFIG_HOME", f.0.join("config"))
                .env("XDG_DATA_HOME", f.0.join("data"))
                .env("XDG_STATE_HOME", f.0.join("state"))
                .env("XDG_CACHE_HOME", f.0.join("cache"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{profile}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    };
    let modal = profile == "modal";
    let root_dir = PathBuf::from(std::env::var_os("REV_CATALOG_ROOT").unwrap());
    fs::create_dir_all(root_dir.join("bin")).unwrap();
    let marker = root_dir.join("helper-ran");
    for name in ["go", "gopls"] {
        let program = root_dir.join("bin").join(name);
        fs::write(
            &program,
            format!("#!/bin/sh\nprintf ran > '{}'\nexit 99\n", marker.display()),
        )
        .unwrap();
        fs::set_permissions(program, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let root = root_dir.join("selected.toml");
    fs::write(root_dir.join("base.toml"), "[files.associations]\n'*.gox' = 'go'\n[lsp.servers.existing]\nlanguages = ['go']\ncommand = '/bin/sh'\nargs = ['--old']\nroot_markers = ['project.marker']\ninitialization_options = { preserved = true }\nsettings = { check = true }\nenv = { SECRET = 'do-not-print-this-value' }\n[lsp.servers.other]\nlanguages = ['unlisted']\ncommand = '/bin/sh'\n").unwrap();
    fs::write(&root, format!("# selected root\ninclude = ['base.toml']\n[editor]\ninteraction_profile = '{}'\n[lsp.preferred]\ngo = 'existing'\n", profile.to_str().unwrap())).unwrap();
    let registry = root_dir.join("data/rev/lsp/installations.toml");
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(&registry, "schema = 'corrupt'\n").unwrap();
    let file = root_dir.join("source.go");
    fs::write(&file, "package sample\n").unwrap();
    let mut s = session(&root, &file);
    let original = fs::read(&root).unwrap();
    // Both selector stages cancel without changing the captured configuration.
    for answers in [Vec::<&str>::new(), vec!["go"]] {
        named(&mut s, "lsp-select-server", modal);
        for value in answers {
            answer(&mut s, value);
        }
        cancel(&mut s, modal);
        assert_eq!(fs::read(&root).unwrap(), original);
    }
    // A recipe points to an observed installed command; user options and the
    // known *.go/*.gox associations survive without another patterns prompt.
    named(&mut s, "lsp-setup", modal);
    for value in ["go", "recipe:gopls", "", ""] {
        answer(&mut s, value);
    }
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
    let saved = prepare(&root);
    assert!(saved.diagnostic.is_none(), "{:?}", saved.diagnostic);
    let server = &saved.snapshot.settings().lsp.servers["gopls"];
    assert_eq!(server.command, "gopls");
    assert_eq!(server.root_markers, ["project.marker"]);
    assert_eq!(
        server.initialization_options["preserved"].as_bool(),
        Some(true)
    );
    assert_eq!(server.settings["check"].as_bool(), Some(true));
    assert_eq!(server.env["SECRET"], "do-not-print-this-value");
    assert!(
        !saved
            .snapshot
            .settings()
            .files
            .associations
            .contains_key("*.go")
    );
    assert_eq!(saved.snapshot.settings().files.associations["*.gox"], "go");
    assert!(
        fs::read_to_string(&root)
            .unwrap()
            .contains("include = ['base.toml']")
    );
    assert_eq!(s.lsp_status(), rev::lsp::Status::Disabled);
    // Custom identities not present in the finite catalog can select an existing
    // profile and add their association through the same native interaction.
    named(&mut s, "lsp-select-server", modal);
    for value in ["unlisted", "other", "[\"*.custom\"]"] {
        answer(&mut s, value);
    }
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
    named(&mut s, "lsp-select-server", modal);
    for value in ["go", "native-only"] {
        answer(&mut s, value);
    }
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
    named(&mut s, "lsp-servers", modal);
    let listing = text(&s);
    assert!(s.buffer().unwrap().is_readonly());
    assert!(s.buffer().unwrap().is_transient());
    assert!(listing.contains("go | native-only"));
    assert!(listing.contains("unlisted | other | external/configured"));
    assert!(listing.contains("gopls | go | gopls | available; protocol unverified"));
    assert!(listing.contains("Managed payload readiness/version: unverified"));
    assert!(!listing.contains("do-not-print-this-value"));
    s.insert('x');
    assert_eq!(text(&s), listing);
    s.load(&file).unwrap();
    assert_eq!(text(&s), "package sample\n");
    assert_eq!(
        s.interaction_profile(),
        if modal {
            rev::config::InteractionProfile::Modal
        } else {
            rev::config::InteractionProfile::NonModal
        }
    );
    assert!(!marker.exists());
    assert_eq!(
        fs::read_to_string(&registry).unwrap(),
        "schema = 'corrupt'\n"
    );
    // Selection cannot silently register an unknown profile.
    let before = fs::read(&root).unwrap();
    s.execute(Command::Lsp(LspCommand::SelectServer));
    answer(&mut s, "go");
    answer(&mut s, "unknown-profile");
    assert!(s.echo().contains("compatible configured server"));
    cancel(&mut s, modal);
    assert_eq!(fs::read(&root).unwrap(), before);
}
