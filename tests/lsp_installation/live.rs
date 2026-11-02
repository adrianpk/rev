//! Explicit network smoke checks. Ordinary regressions use owned offline
//! boundaries; these checks acquire actual packages only when selected manually.
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use rev::buffer::Buffers;
use rev::command::{Command, CompletionCommand, History, Motion};
use rev::config::{Preference, SettingsEdit, Snapshot};
use rev::lsp::install::{self, Manager, Outcome, Registry, State};
use rev::session::CompletionSource;

use super::storage::roots;
use super::workflows::{key, session, text};
use crate::fixture::Fixture;

fn acquire(f: &Fixture, recipe: &'static str, language: &'static str) -> Outcome {
    let roots = roots(f);
    let buffers = Buffers::with_settings(Snapshot::default()).unwrap();
    let stamp = buffers.get(buffers.initial()).unwrap().stamp();
    let mut manager = Manager::default();
    manager
        .start(stamp, roots.clone(), move |mut storage, cancel| {
            install::install(&mut storage, recipe, language, BTreeMap::new(), cancel)
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(915);
    let start = Instant::now();
    let outcome = loop {
        if let Some(result) = manager.poll() {
            break result.unwrap();
        }
        assert!(
            Instant::now() < deadline,
            "owned live acquisition exceeded its deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(!manager.active());
    assert_eq!(outcome.installation.status, State::Ready);
    assert_eq!(
        Registry::load(&roots.data).unwrap().installations[0],
        outcome.installation
    );
    assert!(
        fs::read_dir(roots.cache.join("lsp"))
            .unwrap()
            .next()
            .is_none()
    );
    assert!(outcome.selection.is_none());
    println!(
        "LIVE {recipe}: version={}; runtime={}; evidence={}; elapsed_ms={}",
        outcome.installation.version,
        outcome.installation.runtime,
        outcome.installation.evidence,
        start.elapsed().as_millis()
    );
    outcome
}

// This explicit network/build check is excluded from ordinary offline gates.
// It exercises actual managed gopls and a semantic field absent from disk,
// including single-source results, acceptance, undo and owned process shutdown.
#[test]
#[ignore = "explicit managed upstream acquisition; run separately from offline regressions"]
fn managed_go() {
    let f = Fixture::new();
    let outcome = acquire(&f, "gopls", "go");
    let roots = roots(&f);
    let payload = roots.data.join(&outcome.installation.payload);
    let mut server = outcome.installation.launches[0].server(&payload);
    let analysis = f.0.join("analysis-cache");
    fs::create_dir(&analysis).unwrap();
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
    server
        .env
        .insert("GOCACHE".into(), analysis.to_str().unwrap().into());
    let root = f.write(
        "selected.toml",
        "[completion]\nautomatic = false\n[lsp]\nenabled = true\n",
    );
    let mut edit = SettingsEdit::capture(&root).unwrap();
    edit.register("managed-go", &server);
    edit.prefer("go", &Preference::Server("managed-go".into()));
    edit.publish(&roots.state).unwrap();
    f.write(
        "workspace/go.mod",
        "module example.test/owned\n\ngo 1.27.0\n",
    );
    const DISK: &str = "package main\ntype Widget struct { DiskField int }\nfunc main() { item := Widget{DiskField: 1}; _ = item.DiskField }\n";
    const UNSAVED: &str = "package main\ntype Widget struct { UnsavedField int }\nfunc main() { item := Widget{UnsavedField: 1}; _ = item.Uns }\n";
    let file = f.write("workspace/main.go", DISK);
    let mut s = session(&root, &file);
    s.replace(0..DISK.chars().count(), UNSAVED).unwrap();
    s.execute(Command::Motion(Motion::BufferEnd));
    let point = UNSAVED.find("item.Uns }").unwrap() + "item.Uns".len();
    for _ in point..UNSAVED.chars().count() {
        s.execute(Command::Motion(Motion::BackwardChar));
    }
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        s.execute(Command::Completion(CompletionCommand::AtPoint));
        finish(&mut s, deadline);
        if s.completion()
            .is_some_and(|p| p.candidates().iter().any(|c| c.text() == "UnsavedField"))
        {
            assert!(matches!(
                s.completion_source(),
                Some(CompletionSource::Semantic { .. })
            ));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "actual Go semantic readiness expired: {:?}, {}",
            s.lsp_status(),
            s.echo()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let labels = s
        .completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|c| c.text().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(labels.iter().filter(|s| *s == "UnsavedField").count(), 1);
    assert!(!labels.iter().any(|s| s == "DiskField"));
    let selected = labels.iter().position(|s| s == "UnsavedField").unwrap();
    for _ in 0..selected {
        key(&mut s, KeyCode::Down, KeyModifiers::NONE);
    }
    let history = s.buffer().unwrap().history().len();
    key(&mut s, KeyCode::Tab, KeyModifiers::NONE);
    finish(&mut s, deadline);
    assert_eq!(
        text(&s),
        UNSAVED.replace("item.Uns }", "item.UnsavedField }")
    );
    assert_eq!(s.buffer().unwrap().history().len(), history + 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), UNSAVED);
    assert_eq!(fs::read_to_string(file).unwrap(), DISK);
    let children = owned_children(&server.command);
    assert_eq!(children.len(), 1, "one actual managed Go server");
    let stop = Instant::now();
    drop(s);
    assert!(stop.elapsed() < Duration::from_secs(5));
    for pid in children {
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
    }
    println!(
        "LIVE gopls semantic: unsaved-only field, one semantic source, one undo, disk preserved, owned server reaped"
    );
}

// Actual npm installs verify scripts-disabled compatible packages and every
// delivered launch against the native empty-root exchange. This establishes
// launch/transport compatibility, without claiming web semantic quality.
#[test]
#[ignore = "explicit upstream npm acquisition; run separately from offline regressions"]
fn managed_npm() {
    for (recipe, language, count) in [
        ("typescript-language-server", "typescript", 1),
        ("vscode-langservers-extracted", "html", 3),
    ] {
        let f = Fixture::new();
        let outcome = acquire(&f, recipe, language);
        assert_eq!(outcome.installation.launches.len(), count);
        assert!(
            outcome
                .installation
                .evidence
                .starts_with("npm-lock:sha256:")
        );
        if count == 1 {
            assert!(outcome.installation.runtime.contains("typescript@6."));
        }
    }
}

fn finish(s: &mut rev::session::Session, deadline: Instant) {
    loop {
        s.expire_input(Instant::now());
        if s.remaining_input(Instant::now()).is_none() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "owned Go completion deadline expired"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn owned_children(command: &str) -> Vec<u32> {
    let mut children = std::collections::BTreeSet::new();
    for task in fs::read_dir("/proc/self/task").unwrap().flatten() {
        let pids = fs::read_to_string(task.path().join("children")).unwrap_or_default();
        for pid in pids.split_whitespace() {
            let args = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
            if args.split(|b| *b == 0).next() == Some(command.as_bytes()) {
                children.insert(pid.parse().unwrap());
            }
        }
    }
    children.into_iter().collect()
}
