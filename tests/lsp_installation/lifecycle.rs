use super::storage::roots;
use crate::fixture::Fixture;
use rev::config::{Preference, SettingsEdit, prepare};
use rev::lsp::{
    Client, Status,
    install::{self, Installation, Registry, State, Storage, Update},
};
use rev::org::Cancellation;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub(super) fn child(name: &str) -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("REV_LIFECYCLE_FIXTURE") {
        return Some(directory.into());
    }
    let f = Fixture::new();
    super::npm::fixtures(&f.0);
    super::acquisition::fixtures(&f.0);
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env("REV_LIFECYCLE_FIXTURE", &f.0)
        .env("REV_NPM_FIXTURE", &f.0)
        .env("REV_RUST_FIXTURE", &f.0)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", f.0.join("bin").display()),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    None
}

fn web(storage: &mut Storage) -> Installation {
    install::install_npm(
        storage,
        "vscode-langservers-extracted",
        BTreeMap::new(),
        &Cancellation::default(),
    )
    .unwrap()
    .installation
}

fn next(directory: &Path) {
    for relative in [
        "bin/npm",
        "web/node_modules/vscode-langservers-extracted/package.json",
        "web-lock.json",
    ] {
        let path = directory.join(relative);
        let text = fs::read_to_string(&path).unwrap();
        fs::write(path, text.replace("4.10.0", "4.10.1")).unwrap();
    }
}

// Two selected launch profiles share one verified update and one settings
// publication. Unselected/custom profiles and their user options stay intact.
#[test]
fn shared_update() {
    let Some(directory) = child("lifecycle::shared_update") else {
        return;
    };
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let previous = web(&mut storage);
    let root = f.write(
        "selected.toml",
        "# preserve selected root\n[editor]\ntab_width=5\n",
    );
    // The storage owner serializes settings; publish through that same lock.
    let mut edit = SettingsEdit::capture(&root).unwrap();
    for language in ["css", "html", "json"] {
        let launch = previous
            .launches
            .iter()
            .find(|l| l.languages.contains(&language.into()))
            .unwrap();
        let mut server = launch.server(&roots.data.join(&previous.payload));
        server.settings = toml::Value::Table(
            [("retained".into(), toml::Value::String(language.into()))]
                .into_iter()
                .collect(),
        );
        server.env.insert("USER_OPTION".into(), language.into());
        edit.register(&format!("owned-{language}"), &server);
        edit.prefer(language, &Preference::Server(format!("owned-{language}")));
    }
    edit.prefer("json", &Preference::Native(false));
    storage.select(&edit).unwrap();
    let before = fs::read_to_string(&root).unwrap();
    let snapshot = prepare(&root).snapshot;
    let unchanged_json = snapshot.settings().lsp.servers["owned-json"].clone();
    let edit = SettingsEdit::capture(&root).unwrap();
    let update = Update::capture(&edit, &previous, &roots.data).unwrap();
    assert_eq!(update.profiles(), ["owned-css", "owned-html"]);
    next(&directory);
    let outcome = update
        .run(&mut storage, edit, &Cancellation::default())
        .unwrap();
    assert_eq!(outcome.installation.version, "4.10.1");
    assert!(outcome.selection.is_some());
    let snapshot = prepare(&root).snapshot;
    assert_eq!(snapshot.settings().editor.tab_width, 5);
    assert_eq!(
        snapshot.settings().lsp.servers["owned-json"],
        unchanged_json
    );
    for language in ["css", "html"] {
        let server = snapshot.settings().lsp.selected(language).unwrap().1;
        assert!(server.command.contains(&outcome.installation.id));
        assert_eq!(server.settings["retained"].as_str(), Some(language));
        assert_eq!(server.env["USER_OPTION"], language);
    }
    assert!(roots.data.join(&previous.payload).is_dir());
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0],
        previous
    );
    assert!(
        fs::read_to_string(&root)
            .unwrap()
            .contains("# preserve selected root")
    );
    assert_ne!(fs::read_to_string(&root).unwrap(), before);
    // A failed probe does not touch the selected replacement or previous data.
    let saved = fs::read(&root).unwrap();
    let current = outcome.installation;
    let edit = SettingsEdit::capture(&root).unwrap();
    let update = Update::capture(&edit, &current, &roots.data).unwrap();
    fs::write(directory.join("probe-fail"), "").unwrap();
    assert!(
        update
            .run(&mut storage, edit, &Cancellation::default())
            .err()
            .unwrap()
            .contains("probe failed")
    );
    assert_eq!(fs::read(&root).unwrap(), saved);
    fs::remove_file(directory.join("probe-fail")).unwrap();
    assert!(
        Registry::load(&roots.data)
            .unwrap()
            .installations
            .iter()
            .any(|i| i == &current)
    );
    assert_eq!(
        Registry::load(&roots.data)
            .unwrap()
            .installations
            .iter()
            .filter(|i| i.status == State::Ready)
            .count(),
        2
    );
}

// A real client in another process holds the shared payload inode until the
// server is reaped. Different settings/state roots cannot bypass removal safety.
#[test]
fn running_removal() {
    if let Some(data) = std::env::var_os("REV_REMOVE_RUNNING") {
        let f = Fixture::new();
        let mut roots = roots(&f);
        roots.data = data.into();
        let mut storage = Storage::acquire(roots).unwrap();
        let installation = storage.registry().installations[0].clone();
        let root = f.write("unreferenced.toml", "# other configuration root\n");
        let error = install::remove(
            &mut storage,
            SettingsEdit::capture(&root).unwrap(),
            &installation,
            &Cancellation::default(),
        )
        .err()
        .unwrap();
        assert!(error.contains("running"), "{error}");
        return;
    }
    let Some(_directory) = child("lifecycle::running_removal") else {
        return;
    };
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let installation = web(&mut storage);
    drop(storage);
    let mut policy = rev::config::Lsp::default();
    policy.enabled = true;
    let mut server = installation.launches[0].server(&roots.data.join(&installation.payload));
    let bins = Path::new(&server.command).parent().unwrap().to_path_buf();
    server.command = Path::new(&server.command)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .into();
    server
        .env
        .insert("PATH".into(), format!("{}:/usr/bin:/bin", bins.display()));
    let language = server.languages[0].clone();
    policy.servers = BTreeMap::from([("running".into(), server)]);
    let file = f.write("workspace/sample.html", "owned\n");
    let mut client = Client::new(policy);
    client.activate(&file, &language, Some(file.parent().unwrap()));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(client.status(), Status::Ready(_)) {
        assert!(Instant::now() < deadline, "{:?}", client.status());
        std::thread::sleep(Duration::from_millis(2));
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "lifecycle::running_removal", "--nocapture"])
        .env("REV_REMOVE_RUNNING", &roots.data)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    client.set_enabled(false);
    while client.connection_count() != 0 {
        assert!(Instant::now() < deadline);
        client.collect();
        std::thread::sleep(Duration::from_millis(2));
    }
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let root = f.write("empty.toml", "# retained\n");
    let outcome = install::remove(
        &mut storage,
        SettingsEdit::capture(&root).unwrap(),
        &installation,
        &Cancellation::default(),
    )
    .unwrap();
    assert!(outcome.message.starts_with("Removed installation"));
    assert!(
        Registry::load(&roots.data)
            .unwrap()
            .installations
            .is_empty()
    );
    assert!(!roots.data.join(&installation.payload).exists());
    assert_eq!(fs::read_to_string(root).unwrap(), "# retained\n");
}

// A source edit during package acquisition preserves the new bytes and all old
// preferences, while the verified replacement is retained ready/unselected.
#[test]
fn update_conflict() {
    let Some(directory) = child("lifecycle::update_conflict") else {
        return;
    };
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let previous = web(&mut storage);
    let base = f.write("base.toml", "# retained include\n[editor]\ntab_width=5\n");
    let root = f.write("selected.toml", "# selected\ninclude=['base.toml']\n");
    let mut edit = SettingsEdit::capture(&root).unwrap();
    let server = previous.launches[0].server(&roots.data.join(&previous.payload));
    let language = server.languages[0].clone();
    edit.register("selected", &server);
    edit.prefer(&language, &Preference::Server("selected".into()));
    storage.select(&edit).unwrap();
    let saved = fs::read(&root).unwrap();
    let edit = SettingsEdit::capture(&root).unwrap();
    let update = Update::capture(&edit, &previous, &roots.data).unwrap();
    let npm = directory.join("bin/npm");
    let script=fs::read_to_string(&npm).unwrap().replace("'install vscode-langservers-extracted@4.10.0')","'install vscode-langservers-extracted@4.10.0')\n        target=$(/bin/cat \"$REV_NPM_FIXTURE/conflict-path\"); printf '# concurrent edit\\n' >> \"$target\"");
    fs::write(&npm, script).unwrap();
    fs::write(directory.join("conflict-path"), base.to_str().unwrap()).unwrap();
    let outcome = update
        .run(&mut storage, edit, &Cancellation::default())
        .unwrap();
    assert_eq!(outcome.installation.status, State::Ready);
    assert!(outcome.selection.is_none());
    assert!(outcome.message.contains("source conflict"));
    assert_eq!(fs::read(&root).unwrap(), saved);
    assert!(
        fs::read_to_string(&base)
            .unwrap()
            .ends_with("# concurrent edit\n")
    );
    assert_eq!(
        prepare(&root)
            .snapshot
            .settings()
            .lsp
            .selected(&language)
            .unwrap()
            .1
            .command,
        server.command
    );
    assert!(roots.data.join(&previous.payload).is_dir());
}

// References include unselected profiles, absolute arguments/environment and
// executable aliases. Incomplete cleanup refuses aliases and escaping links;
// only positively marked, unlocked cache directories may be discarded.
#[test]
fn guarded_cleanup() {
    let Some(_directory) = child("lifecycle::guarded_cleanup") else {
        return;
    };
    let f = Fixture::new();
    let roots = roots(&f);
    let mut storage = Storage::acquire(roots.clone()).unwrap();
    let previous = web(&mut storage);
    let root = f.write("selected.toml", "# unchanged\n");
    let server = previous.launches[0].server(&roots.data.join(&previous.payload));
    let mut edit = SettingsEdit::capture(&root).unwrap();
    edit.register("unselected", &server);
    edit.prefer(&server.languages[0], &Preference::Native(false));
    storage.select(&edit).unwrap();
    let error = install::remove(
        &mut storage,
        SettingsEdit::capture(&root).unwrap(),
        &previous,
        &Cancellation::default(),
    )
    .err()
    .unwrap();
    assert!(error.contains("unselected"));
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0],
        previous
    );
    let alias = f.0.join("external-alias");
    std::os::unix::fs::symlink(&server.command, &alias).unwrap();
    for mode in ["arguments", "environment", "alias", "search-path"] {
        let mut custom = server.clone();
        custom.command = "/bin/sh".into();
        custom.args.clear();
        custom.env.clear();
        match mode {
            "arguments" => custom.args.push(server.command.clone()),
            "environment" => {
                custom.env.insert(
                    "SERVER_DATA".into(),
                    roots.data.join(&previous.payload).to_str().unwrap().into(),
                );
            }
            "alias" => custom.command = alias.to_str().unwrap().into(),
            "search-path" => {
                custom.command = Path::new(&server.command)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .into();
                custom.env.insert(
                    "PATH".into(),
                    Path::new(&server.command)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .into(),
                );
            }
            _ => unreachable!(),
        }
        let mut edit = SettingsEdit::capture(&root).unwrap();
        edit.register("unselected", &custom);
        storage.select(&edit).unwrap();
        let error = install::remove(
            &mut storage,
            SettingsEdit::capture(&root).unwrap(),
            &previous,
            &Cancellation::default(),
        )
        .err()
        .unwrap();
        assert!(error.contains("unselected"), "{mode}: {error}");
    }
    let mut incomplete = storage
        .candidate(
            "rust-analyzer",
            "interrupted",
            "fixture",
            "https://example.test",
        )
        .unwrap()
        .installation;
    let payload = roots.data.join(&incomplete.payload);
    fs::remove_dir(&payload).unwrap();
    std::os::unix::fs::symlink(roots.data.join(&previous.payload), &payload).unwrap();
    assert!(
        install::remove(
            &mut storage,
            SettingsEdit::capture(&root).unwrap(),
            &incomplete,
            &Cancellation::default()
        )
        .err()
        .unwrap()
        .contains("aliased")
    );
    assert!(roots.data.join(&previous.payload).is_dir());
    fs::remove_file(&payload).unwrap();
    fs::create_dir(&payload).unwrap();
    std::os::unix::fs::symlink(&root, payload.join("escape")).unwrap();
    assert!(
        install::remove(
            &mut storage,
            SettingsEdit::capture(&root).unwrap(),
            &incomplete,
            &Cancellation::default()
        )
        .err()
        .unwrap()
        .contains("escapes")
    );
    fs::remove_file(payload.join("escape")).unwrap();
    // Interrupted records can have absent data, and remain removable.
    fs::remove_dir(&payload).unwrap();
    incomplete.status = State::Incomplete;
    storage.save(incomplete.clone()).unwrap();
    let cache = roots.cache.join("lsp");
    fs::create_dir_all(&cache).unwrap();
    let owner = b"rev-lsp-acquisition-cache-v1\n";
    for name in ["job-idle", "job-active", "job-unknown"] {
        fs::create_dir(cache.join(name)).unwrap();
        fs::write(cache.join(name).join("data"), "owned fixture").unwrap();
    }
    for name in ["job-idle", "job-active"] {
        fs::write(cache.join(name).join("rev-cache-owner"), owner).unwrap();
    }
    let active = fs::File::open(cache.join("job-active")).unwrap();
    active.try_lock().unwrap();
    let outcome = install::remove(
        &mut storage,
        SettingsEdit::capture(&root).unwrap(),
        &incomplete,
        &Cancellation::default(),
    )
    .unwrap();
    assert!(
        outcome.message.contains("cleaned 1 idle"),
        "{}",
        outcome.message
    );
    assert!(!cache.join("job-idle").exists());
    assert!(cache.join("job-active/data").is_file());
    assert!(cache.join("job-unknown/data").is_file());
    assert!(roots.data.join(&previous.payload).is_dir());
    assert_eq!(Registry::load(&roots.data).unwrap().installations.len(), 1);
}
