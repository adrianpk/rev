use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use rev::config::{Roots, prepare};
use rev::lsp::{
    Status,
    install::{Registry, State},
};
use rev::session::Session;

use super::workflows::{answer, key, named, session, text};
use crate::fixture::Fixture;

fn wait(session: &mut Session, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !session.echo().contains(expected) {
        assert!(
            Instant::now() < deadline,
            "expected {expected}: {}",
            session.echo()
        );
        session.expire_input(Instant::now());
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn insert(session: &mut Session, character: char, modal: bool) {
    if modal {
        key(session, KeyCode::Char('i'), KeyModifiers::NONE);
    }
    key(session, KeyCode::Char(character), KeyModifiers::NONE);
    if modal {
        key(session, KeyCode::Esc, KeyModifiers::NONE);
        session.expire_input(Instant::now() + Duration::from_secs(1));
    }
}

// Native M-x/colon entry points run in owned XDG subprocesses for both startup
// profiles. The acquisition fixture proves persistence/transport ownership and
// disabled-mode isolation; it does not establish upstream Rust semantics.
#[test]
fn native_installation() {
    let Some(profile) = std::env::var_os("REV_MANAGED_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            super::acquisition::fixtures(&f.0);
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "managed::native_installation", "--nocapture"])
                .env("REV_MANAGED_PROFILE", profile)
                .env("REV_RUST_FIXTURE", &f.0)
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
    let base_text = "[lsp.servers.rust]\nroot_markers = ['Workspace']\nenv = { SECRET = 'private-literal-value' }\ninitialization_options = { retained = true }\nsettings = { retained = 'setting' }\n[files.associations]\n'*.special' = 'rust'\n";
    let base = f.write("base.toml", base_text);
    let initial = format!(
        "# keep selected root\ninclude = ['base.toml']\n[editor]\ninteraction_profile = '{}'\ntab_width = 5\n[lsp.preferred]\nrust = 'rust'\n[lsp.servers.rust-analyzer]\nlanguages = ['rust']\ncommand = '/already/external'\n",
        profile.to_str().unwrap()
    );
    let root = f.write("selected.toml", &initial);
    let file = f.write("sample.rs", "fn main() {}\n");
    let mut s = session(&root, &file);
    let roots = Roots::from_env().unwrap();
    let fixture = std::path::PathBuf::from(std::env::var_os("REV_RUST_FIXTURE").unwrap());
    for cancel in [false, true].into_iter().filter(|escape| !escape || modal) {
        named(&mut s, "lsp-install-server", modal);
        if cancel {
            key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
            s.expire_input(Instant::now() + Duration::from_secs(1));
        } else {
            key(&mut s, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        assert!(s.echo().contains("cancelled"), "{}", s.echo());
        assert_eq!(fs::read_to_string(&root).unwrap(), initial);
        assert!(!fixture.join("helper-cwd").exists());
    }
    named(&mut s, "lsp-setup", modal);
    answer(&mut s, "rust");
    answer(&mut s, "install:rust-analyzer");
    assert!(s.echo().contains("started"), "{}", s.echo());
    insert(&mut s, 'x', modal);
    let edited = text(&s);
    let point = s.view().point;
    wait(&mut s, "selected and saved");
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    assert_eq!(s.lsp_status(), Status::Disabled);
    let installed = Registry::load(&roots.data).unwrap();
    assert_eq!(installed.installations.len(), 1);
    assert_eq!(installed.installations[0].status, State::Ready);
    let snapshot = prepare(&root).snapshot;
    assert!(!snapshot.settings().lsp.enabled);
    let (selected, server) = snapshot.settings().lsp.selected("rust").unwrap();
    assert_ne!(selected, "rust-analyzer");
    assert_eq!(
        snapshot.settings().lsp.servers["rust-analyzer"].command,
        "/already/external"
    );
    assert!(Path::new(&server.command).is_absolute());
    assert!(server.command.starts_with(roots.data.to_str().unwrap()));
    assert_eq!(server.env["SECRET"], "private-literal-value");
    assert_eq!(server.root_markers, vec!["Workspace"]);
    assert_eq!(
        server.initialization_options["retained"].as_bool(),
        Some(true)
    );
    assert_eq!(server.settings["retained"].as_str(), Some("setting"));
    assert_eq!(snapshot.settings().editor.tab_width, 5);
    assert!(snapshot.patterns("rust").contains(&"*.special".to_owned()));
    assert_eq!(fs::read_to_string(&base).unwrap(), base_text);
    assert!(
        fs::read_to_string(&root)
            .unwrap()
            .contains("# keep selected root")
    );
    let saved = fs::read_to_string(&root).unwrap();
    named(&mut s, "lsp-install-server", modal);
    fs::write(&base, format!("{base_text}\n# concurrent include edit\n")).unwrap();
    answer(&mut s, "rust-analyzer");
    wait(&mut s, "not selected");
    assert!(s.echo().contains("source conflict"), "{}", s.echo());
    assert_eq!(fs::read_to_string(&root).unwrap(), saved);
    assert!(
        fs::read_to_string(&base)
            .unwrap()
            .contains("concurrent include edit")
    );
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    let installations = Registry::load(&roots.data).unwrap();
    assert_eq!(installations.installations.len(), 2);
    assert!(
        installations
            .installations
            .iter()
            .all(|i| i.status == State::Ready)
    );
    assert_eq!(installations.installations[0], installed.installations[0]);
    let calls = fs::read_to_string(fixture.join("helper-cwd")).unwrap();
    named(&mut s, "lsp-select-server", modal);
    answer(&mut s, "rust");
    answer(
        &mut s,
        &format!("ready:{}", installations.installations[1].id),
    );
    wait(&mut s, "selected and saved");
    assert_eq!(
        fs::read_to_string(fixture.join("helper-cwd")).unwrap(),
        calls
    );
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    let snapshot = prepare(&root).snapshot;
    let (_, selected) = snapshot.settings().lsp.selected("rust").unwrap();
    assert!(
        selected
            .command
            .contains(&installations.installations[1].id)
    );
    named(&mut s, "lsp-servers", modal);
    let inventory = text(&s);
    assert!(
        inventory.contains("managed/ready") && inventory.contains("fixture-1"),
        "{inventory}"
    );
    assert!(inventory.contains("ready (verified transport)"));
    assert!(!inventory.contains("private-literal-value"));
    assert!(s.buffer().unwrap().is_readonly());
    s.load(&file).unwrap();
    assert_eq!(text(&s), edited);
    let registry = roots.data.join("lsp/installations.toml");
    fs::write(&registry, "invalid registry").unwrap();
    named(&mut s, "lsp-install-server", modal);
    answer(&mut s, "rust-analyzer");
    wait(&mut s, "invalid installation registry");
    assert_eq!(fs::read_to_string(&registry).unwrap(), "invalid registry");
    assert_eq!(
        fs::read_to_string(fixture.join("helper-cwd")).unwrap(),
        calls
    );
    assert_eq!(text(&s), edited);
    named(&mut s, "lsp-select-server", modal);
    answer(&mut s, "rust");
    answer(&mut s, "rust-analyzer");
    assert!(s.echo().contains("saved and applied"), "{}", s.echo());
    assert_eq!(
        prepare(&root)
            .snapshot
            .settings()
            .lsp
            .selected("rust")
            .unwrap()
            .1
            .command,
        "/already/external"
    );
    assert_eq!(fs::read_to_string(&registry).unwrap(), "invalid registry");
    let frames = fs::read_to_string(fixture.join("probe.frames")).unwrap();
    assert_eq!(frames.matches("\"method\":\"initialize\"").count(), 2);
    assert!(!frames.contains("textDocument/"));
    let pid = fs::read_to_string(fixture.join("probe.pid")).unwrap();
    assert!(!Path::new(&format!("/proc/{}", pid.trim())).exists());
}

// Both native entry profiles exercise Go, multi-language npm selection, shared
// web records and helper-free reuse. Controlled tools isolate configuration and
// UI contracts; separate live probes establish actual package compatibility.
#[test]
fn native_packages() {
    let Some(profile) = std::env::var_os("REV_PACKAGES_PROFILE") else {
        for profile in ["modal", "non-modal"] {
            let f = Fixture::new();
            super::go::fixtures(&f.0);
            super::npm::fixtures(&f.0);
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "managed::native_packages", "--nocapture"])
                .env("REV_PACKAGES_PROFILE", profile)
                .env("REV_GO_FIXTURE", &f.0)
                .env("REV_NPM_FIXTURE", &f.0)
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
            r#"# preserve this root
[editor]
interaction_profile = '{}'
[lsp.servers.gopls]
languages = ['go']
command = '/external/go-server'
root_markers = ['Owned.work']
[lsp.servers.ts-existing]
languages = ['typescript']
command = '/external/typescript'
settings = {{ retained = 'typescript' }}
[lsp.servers.js-existing]
languages = ['javascript']
command = '/external/javascript'
settings = {{ retained = 'javascript' }}
[lsp.servers.css-existing]
languages = ['css']
command = '/external/css'
settings = {{ retained = 'css' }}
env = {{ EXAMPLE_OPTION = 'private-example-value' }}
[lsp.servers.html-existing]
languages = ['html']
command = '/external/html'
settings = {{ retained = 'html' }}
"#,
            profile.to_str().unwrap()
        ),
    );
    let file = f.write("sample.go", "package main\n");
    let mut s = session(&root, &file);
    let roots = Roots::from_env().unwrap();
    let fixture = std::path::PathBuf::from(std::env::var_os("REV_NPM_FIXTURE").unwrap());
    named(&mut s, "lsp-setup", modal);
    answer(&mut s, "go");
    answer(&mut s, "install:gopls");
    insert(&mut s, 'x', modal);
    let edited = text(&s);
    let point = s.view().point;
    wait(&mut s, "selected and saved");
    let snapshot = prepare(&root).snapshot;
    let (id, go) = snapshot.settings().lsp.selected("go").unwrap();
    assert_ne!(id, "gopls");
    assert_eq!(
        snapshot.settings().lsp.servers["gopls"].command,
        "/external/go-server"
    );
    assert_eq!(go.root_markers, vec!["Owned.work"]);
    let before_cancel = fs::read_to_string(fixture.join("calls")).unwrap();
    for escape in [false, true].into_iter().filter(|escape| !escape || modal) {
        named(&mut s, "lsp-install-server", modal);
        answer(&mut s, "typescript-language-server");
        if escape {
            key(&mut s, KeyCode::Esc, KeyModifiers::NONE);
            s.expire_input(Instant::now() + Duration::from_secs(1));
        } else {
            key(&mut s, KeyCode::Char('g'), KeyModifiers::CONTROL);
        }
        assert!(s.echo().contains("cancelled"));
        assert_eq!(
            fs::read_to_string(fixture.join("calls")).unwrap(),
            before_cancel
        );
    }
    named(&mut s, "lsp-install-server", modal);
    answer(&mut s, "typescript-language-server");
    answer(&mut s, "javascript");
    wait(&mut s, "selected and saved");
    let snapshot = prepare(&root).snapshot;
    let (_, js) = snapshot.settings().lsp.selected("javascript").unwrap();
    assert_eq!(js.settings["retained"].as_str(), Some("javascript"));
    assert!(
        js.command
            .contains("node_modules/.bin/typescript-language-server")
    );
    assert_eq!(
        snapshot.settings().lsp.selected("typescript").unwrap().0,
        "ts-existing"
    );
    named(&mut s, "lsp-install-server", modal);
    answer(&mut s, "vscode-langservers-extracted");
    answer(&mut s, "css");
    wait(&mut s, "selected and saved");
    let snapshot = prepare(&root).snapshot;
    let (_, css) = snapshot.settings().lsp.selected("css").unwrap();
    assert_eq!(css.settings["retained"].as_str(), Some("css"));
    assert_eq!(css.env["EXAMPLE_OPTION"], "private-example-value");
    let registry = Registry::load(&roots.data).unwrap();
    let web = registry
        .installations
        .iter()
        .find(|i| i.recipe == "vscode-langservers-extracted")
        .unwrap();
    assert_eq!(web.launches.len(), 3);
    let calls = fs::read_to_string(fixture.join("calls")).unwrap();
    named(&mut s, "lsp-install-server", modal);
    answer(&mut s, &format!("ready:{}", web.id));
    answer(&mut s, "html");
    wait(&mut s, "selected and saved");
    assert_eq!(fs::read_to_string(fixture.join("calls")).unwrap(), calls);
    let snapshot = prepare(&root).snapshot;
    let (_, html) = snapshot.settings().lsp.selected("html").unwrap();
    assert_eq!(html.settings["retained"].as_str(), Some("html"));
    for (language, pattern) in [
        ("go", "*.go"),
        ("javascript", "*.js"),
        ("css", "*.css"),
        ("html", "*.html"),
    ] {
        assert!(
            snapshot.patterns(language).contains(&pattern.to_owned()),
            "{language}"
        );
    }
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(text(&s), edited);
    assert_eq!(s.view().point, point);
    assert!(
        fs::read_to_string(&root)
            .unwrap()
            .contains("# preserve this root")
    );
    named(&mut s, "lsp-servers", modal);
    let inventory = text(&s);
    for version in ["v0.23.0", "6.0.2", "4.10.0"] {
        assert!(inventory.contains(version));
    }
    assert!(!inventory.contains("private-example-value"));
    assert!(s.buffer().unwrap().is_readonly());
}
