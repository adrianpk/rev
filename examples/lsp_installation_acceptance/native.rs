use crate::{
    semantic,
    support::{Result, answer, children, idle, named, reaped, text, wait},
};
use rev::{
    buffer::Buffers,
    command::{Command, CompletionCommand},
    config::{self, Preference, Roots, SettingsEdit},
    lsp::{
        Status,
        install::{Installation, Registry, State},
    },
    session::{CompletionSource, Session},
    view::{Size, View},
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(crate) fn run(root: &Path) -> Result<()> {
    let started = Instant::now();
    let config = configure(root)?;
    let mut s = startup(root, &config)?;
    s.insert('x');
    let roots = managed(&mut s, &config)?;
    named(&mut s, "lsp-mode");
    semantic::run(&mut s, &semantic::RUST)?;
    go(&mut s, root, &config)?;
    custom(&mut s, root)?;
    finish(s, root, &roots)?;
    println!(
        "PASS native management profile={} elapsed_ms={}: verified install/update/remove, actual Rust/Go, custom language without grammar, unchanged disk and owned shutdown",
        std::env::var("REV_ACCEPTANCE_PROFILE")?,
        started.elapsed().as_millis()
    );
    Ok(())
}

fn configure(root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(root.join("workspace"))?;
    let profile = std::env::var("REV_ACCEPTANCE_PROFILE")?;
    let config = Roots::from_env()?.config.join("config.toml");
    fs::create_dir_all(config.parent().ok_or("missing config parent")?)?;
    fs::write(
        &config,
        format!(
            "# retain user root\n[editor]\ninteraction_profile='{profile}'\ntab_width=5\n[tree]\nvisible=false\n[completion]\nautomatic=false\n[lsp]\nrequest_timeout_ms=2000\n"
        ),
    )?;
    fs::write(
        root.join("workspace/Cargo.toml"),
        "[package]\nname='managed_acceptance'\nversion='0.0.0'\nedition='2024'\n[workspace]\n[[bin]]\nname='managed_acceptance'\npath='main.rs'\n",
    )?;
    fs::write(
        root.join("workspace/go.mod"),
        "module example.test/acceptance\n\ngo 1.27.0\n",
    )?;
    Ok(config)
}

fn session(config: &Path, file: &Path) -> Result<Session> {
    let prepared = config::prepare(config);
    assert!(prepared.diagnostic.is_none(), "{:?}", prepared.diagnostic);
    let buffers = Buffers::with_settings(prepared.snapshot)?;
    let view = View::new(buffers.get(buffers.initial())?);
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 24,
        },
    )?;
    s.load(file)?;
    Ok(s)
}

fn managed(s: &mut Session, config: &Path) -> Result<Roots> {
    let roots = Roots::from_env()?;
    let source = text(s);
    let stamp = s.buffer()?.stamp();
    let point = s.view().point;
    let before = calls()?;
    let old = acquire(s, &roots, config)?;
    let current = replace(s, &roots, &old)?;
    assert!(calls()? > before);
    assert_eq!(s.lsp_status(), Status::Disabled);
    assert_eq!(text(s), source);
    assert_eq!(s.buffer()?.stamp(), stamp);
    assert_eq!(s.view().point, point);
    assert_eq!(
        config::prepare(config).snapshot.settings().editor.tab_width,
        5
    );
    assert!(fs::read_to_string(config)?.contains("# retain user root"));
    println!(
        "MEASURE managed real Rust payload: bytes={} local_version={}; prior version removed only after retarget",
        fs::metadata(
            roots
                .data
                .join(&current.payload)
                .join(&current.launches[0].executable)
        )?
        .len(),
        current.version
    );
    Ok(roots)
}

fn go(s: &mut Session, root: &Path, config: &Path) -> Result<()> {
    let mut server = config::Server {
        languages: vec!["go".into()],
        command: std::env::var("REV_ACCEPTANCE_GO")?,
        args: Vec::new(),
        root_markers: vec!["go.mod".into()],
        env: Default::default(),
        initialization_options: toml::Value::Table(Default::default()),
        settings: toml::Value::Table(Default::default()),
    };
    for (name, value) in [
        ("GOTOOLCHAIN", "local"),
        ("GOENV", "off"),
        ("GOWORK", "off"),
        ("GOPROXY", "off"),
        ("GOSUMDB", "off"),
        ("GOTELEMETRY", "off"),
        ("GOMAXPROCS", "2"),
    ] {
        server.env.insert(name.into(), value.into());
    }
    server.env.insert(
        "GOCACHE".into(),
        root.join("go-cache").to_string_lossy().into_owned(),
    );
    let mut edit = SettingsEdit::capture(config)?;
    edit.register("installed-go", &server);
    edit.prefer("go", &Preference::Server("installed-go".into()));
    edit.publish(&Roots::from_env()?.state)?;
    named(s, "lsp-reload-settings");
    named(s, "lsp-setup");
    answer(s, "go");
    answer(s, "installed-go");
    let file = root.join("workspace/main.go");
    fs::write(&file, semantic::GO.disk)?;
    s.load(&file)?;
    semantic::run(s, &semantic::GO)
}

fn custom(s: &mut Session, root: &Path) -> Result<()> {
    let peer = register_custom(s, root)?;
    custom_semantic(s, &peer)?;
    native_only(s)
}

fn custom_semantic(s: &mut Session, peer: &Path) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        s.execute(Command::Completion(CompletionCommand::AtPoint));
        semantic::settle(s, deadline)?;
        if matches!(
            s.completion_source(),
            Some(CompletionSource::Semantic { .. })
        ) {
            break;
        }
        if Instant::now() >= deadline {
            return Err("custom language did not attach".into());
        }
    }
    let labels: Vec<_> = s
        .completion()
        .ok_or("missing custom popup")?
        .candidates()
        .iter()
        .map(|c| c.text())
        .collect();
    assert_eq!(labels, ["custom_semantic"]);
    let frames = fs::read_to_string(peer.with_extension("frames"))?;
    assert!(
        frames
            .lines()
            .map(serde_json::from_str::<serde_json::Value>)
            .collect::<std::result::Result<Vec<_>, _>>()?
            .iter()
            .any(|v| v["method"] == "textDocument/didOpen"
                && v["params"]["textDocument"]["languageId"] == "fixture-language")
    );
    Ok(())
}

fn native_only(s: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    s.execute(Command::Completion(CompletionCommand::Cancel));
    named(s, "lsp-select-server");
    answer(s, "fixture-language");
    answer(s, "native-only");
    s.execute(Command::Completion(CompletionCommand::AtPoint));
    semantic::settle(s, deadline)?;
    assert!(s.completion().is_none());
    assert_eq!(s.completion_source(), None);
    assert_eq!(text(s), "candidate");
    let pids = children()?;
    named(s, "lsp-mode");
    semantic::settle(s, Instant::now() + Duration::from_secs(5))?;
    reaped(&pids)?;
    assert!(children()?.is_empty());
    idle(s)
}

fn calls() -> Result<usize> {
    let path =
        PathBuf::from(std::env::var_os("REV_INSTALLATION_FIXTURE").ok_or("missing fixture")?)
            .join("calls");
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.lines().count()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e.into()),
    }
}

fn startup(root: &Path, config: &Path) -> Result<Session> {
    let file = root.join("workspace/main.rs");
    fs::write(&file, semantic::RUST.disk)?;
    let before = calls()?;
    let mut s = session(config, &file)?;
    idle(&mut s)?;
    assert_eq!(calls()?, before, "startup or idle acquired a package");
    assert!(children()?.is_empty(), "disabled startup launched a child");
    Ok(s)
}

fn finish(s: Session, root: &Path, roots: &Roots) -> Result<()> {
    let pids = children()?;
    drop(s);
    reaped(&pids)?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/main.rs"))?,
        semantic::RUST.disk
    );
    assert_eq!(
        fs::read_to_string(root.join("workspace/main.go"))?,
        semantic::GO.disk
    );
    assert!(fs::read_dir(roots.cache.join("lsp"))?.next().is_none());
    Ok(())
}

fn acquire(s: &mut Session, roots: &Roots, config: &Path) -> Result<Installation> {
    let initial = fs::read(config)?;
    let before = calls()?;
    named(s, "lsp-install-server");
    s.handle(crossterm::event::Event::Key(
        crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('g'),
            crossterm::event::KeyModifiers::CONTROL,
        ),
    ));
    assert_eq!(fs::read(config)?, initial);
    assert_eq!(calls()?, before);
    named(s, "lsp-install-server");
    answer(s, "rust-analyzer");
    wait(s, "selected and saved")?;
    let old = Registry::load(&roots.data)?.installations.remove(0);
    assert_eq!(old.status, State::Ready);
    Ok(old)
}

fn replace(s: &mut Session, roots: &Roots, old: &Installation) -> Result<Installation> {
    named(s, "lsp-update-server");
    answer(s, &old.id);
    answer(s, "yes");
    wait(s, "updated 1 selected profiles")?;
    let registry = Registry::load(&roots.data)?;
    assert_eq!(registry.installations.len(), 2);
    let current = registry
        .installations
        .iter()
        .find(|i| i.id != old.id)
        .ok_or("missing replacement")?;
    named(s, "lsp-remove-server");
    answer(s, &old.id);
    answer(s, "yes");
    wait(s, "Removed installation")?;
    assert!(!roots.data.join(&old.payload).exists());
    assert_eq!(Registry::load(&roots.data)?.installations.len(), 1);
    Ok(current.clone())
}

fn register_custom(s: &mut Session, root: &Path) -> Result<PathBuf> {
    let fixture =
        PathBuf::from(std::env::var_os("REV_INSTALLATION_FIXTURE").ok_or("missing local fixture")?);
    let peer = root.join("custom-peer");
    fs::write(
        peer.with_extension("result"),
        "[{\"label\":\"custom_semantic\"}]",
    )?;
    let args = serde_json::to_string(&[
        fixture.join("peer.sh").to_string_lossy().into_owned(),
        "ready".into(),
        peer.to_string_lossy().into_owned(),
    ])?;
    named(s, "lsp-setup");
    for value in [
        "fixture-language",
        "custom-peer",
        "/bin/sh",
        &args,
        "[]",
        "[\"*.unbundled\"]",
    ] {
        answer(s, value);
    }
    let file = root.join("workspace/document.unbundled");
    fs::write(&file, "candidate")?;
    s.load(&file)?;
    assert_eq!(s.buffer()?.mode(), "Text");
    Ok(peer)
}
