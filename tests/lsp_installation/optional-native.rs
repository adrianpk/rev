use super::workflows::{answer, key, named, session, text};
use crate::fixture::Fixture;
use crossterm::event::{KeyCode, KeyModifiers};
use rev::config::{Preference, Roots, SettingsEdit, prepare};
use rev::lsp::{Status, install::Registry};
use std::fs;
use std::time::{Duration, Instant};

fn wait(s: &mut rev::session::Session) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !s.echo().contains("selected and saved") {
        assert!(Instant::now() < deadline, "{}", s.echo());
        s.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(2));
    }
}

// Both native command profiles preserve editing and disabled analysis across
// all four acquisitions. Reuse must restore required runtime paths/options
// while retaining unrelated user options and running no acquisition helpers.
#[test]
fn native_optional() {
    let Some(profile) = std::env::var_os("REV_OPTIONAL_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            super::ols::tools(&f.0);
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "optional_native::native_optional", "--nocapture"])
                .env("REV_OPTIONAL_PROFILE", profile)
                .env("REV_OPTIONAL_FIXTURE", &f.0)
                .env(
                    "PATH",
                    format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
                )
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
    let f = Fixture::new();
    let root = f.write(
        "selected.toml",
        &format!(
            "# preserve root\n[editor]\ninteraction_profile = '{}'\ntab_width = 5\n",
            profile.to_str().unwrap()
        ),
    );
    let file = f.write("owned.txt", "retained text\n");
    let mut s = session(&root, &file);
    let roots = Roots::from_env().unwrap();
    let fixture = std::path::PathBuf::from(std::env::var_os("REV_OPTIONAL_FIXTURE").unwrap());
    if modal {
        key(&mut s, KeyCode::Char('i'), KeyModifiers::NONE);
    }
    key(&mut s, KeyCode::Char('x'), KeyModifiers::NONE);
    if modal {
        key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
        s.expire_input(Instant::now() + Duration::from_secs(1));
    }
    let edited = text(&s);
    let point = s.view().point;
    for (recipe, language) in [
        ("marksman", "markdown"),
        ("ruby-lsp", "ruby"),
        ("python-lsp-server", "python"),
        ("ols", "odin"),
    ] {
        named(&mut s, "lsp-setup", modal);
        answer(&mut s, language);
        answer(&mut s, &format!("install:{recipe}"));
        if prepare(&root).snapshot.patterns(language).is_empty() {
            answer(&mut s, "[\"*.odin\"]");
        }
        wait(&mut s);
        let registry = Registry::load(&roots.data).unwrap();
        let installation = registry
            .installations
            .iter()
            .find(|i| i.recipe == recipe)
            .unwrap();
        let snapshot = prepare(&root).snapshot;
        let (_, server) = snapshot.settings().lsp.selected(language).unwrap();
        assert!(server.command.starts_with(roots.data.to_str().unwrap()));
        let mut changed = server.clone();
        changed.env.insert("USER_OPTION".into(), "retained".into());
        if recipe == "ruby-lsp" {
            changed
                .env
                .insert("GEM_HOME".into(), "/outside/gems".into());
        }
        if recipe == "ols" {
            changed
                .env
                .insert("OLS_BUILTIN_FOLDER".into(), "/outside/builtin".into());
            changed.initialization_options["enable_snippets"] = toml::Value::Boolean(true);
            changed.initialization_options["enable_fake_methods"] = toml::Value::Boolean(true);
            changed
                .initialization_options
                .as_table_mut()
                .unwrap()
                .insert("retained".into(), toml::Value::Boolean(true));
        }
        let mut edit = SettingsEdit::capture(&root).unwrap();
        let current = format!("user-{language}");
        edit.register(&current, &changed);
        edit.prefer(language, &Preference::Server(current));
        edit.publish(&roots.state).unwrap();
        named(&mut s, "lsp-reload-settings", modal);
        let calls = fs::read_to_string(fixture.join("calls")).unwrap();
        named(&mut s, "lsp-select-server", modal);
        answer(&mut s, language);
        answer(&mut s, &format!("ready:{}", installation.id));
        wait(&mut s);
        assert_eq!(fs::read_to_string(fixture.join("calls")).unwrap(), calls);
        let snapshot = prepare(&root).snapshot;
        let (_, selected) = snapshot.settings().lsp.selected(language).unwrap();
        assert_eq!(selected.env["USER_OPTION"], "retained");
        if recipe == "ruby-lsp" {
            assert_eq!(
                selected.env["GEM_HOME"],
                roots
                    .data
                    .join(&installation.payload)
                    .join("gems")
                    .to_str()
                    .unwrap()
            );
            assert_eq!(selected.env["BUNDLE_FROZEN"], "true");
        }
        if recipe == "ols" {
            assert_eq!(
                selected.env["OLS_BUILTIN_FOLDER"],
                roots
                    .data
                    .join(&installation.payload)
                    .join("source/builtin")
                    .to_str()
                    .unwrap()
            );
            for option in ["enable_snippets", "enable_fake_methods"] {
                assert_eq!(
                    selected.initialization_options[option].as_bool(),
                    Some(false)
                );
            }
            assert_eq!(
                selected.initialization_options["retained"].as_bool(),
                Some(true)
            );
        }
        assert_eq!(text(&s), edited);
        assert_eq!(s.view().point, point);
        assert_eq!(s.lsp_status(), Status::Disabled);
        assert_eq!(snapshot.settings().editor.tab_width, 5);
    }
    assert!(
        fs::read_to_string(&root)
            .unwrap()
            .contains("# preserve root")
    );
    assert!(
        rev::lsp::catalog::RECIPES
            .iter()
            .all(|r| r.launch("org").is_none())
    );
    for (language, pattern) in [("org", "*.org"), ("science-x", "*.labx")] {
        named(&mut s, "lsp-setup", modal);
        for answer_text in [
            language,
            &format!("custom-{language}"),
            "/bin/sh",
            "[]",
            "[]",
        ] {
            answer(&mut s, answer_text);
        }
        if prepare(&root).snapshot.patterns(language).is_empty() {
            answer(&mut s, &format!("[\"{pattern}\"]"));
        }
        assert!(s.echo().contains("saved and applied"), "{}", s.echo());
        let snapshot = prepare(&root).snapshot;
        assert_eq!(
            snapshot
                .settings()
                .lsp
                .selected(language)
                .unwrap()
                .1
                .command,
            "/bin/sh"
        );
        assert!(snapshot.patterns(language).contains(&pattern.to_owned()));
    }
    named(&mut s, "lsp-servers", modal);
    let inventory = text(&s);
    assert!(!inventory.contains("/outside/gems"));
    assert!(!inventory.contains("USER_OPTION"));
}
