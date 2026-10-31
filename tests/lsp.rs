#![forbid(unsafe_code)]
#[path = "lsp/controls.rs"]
mod controls;
#[path = "configuration/fixture.rs"]
mod fixture;

use fixture::Fixture;
use rev::config::{Lsp, Server};
use rev::lsp::{Client, Encoding, Status};
use std::path::Path;
use std::time::{Duration, Instant};

fn policy(fixture: &Fixture, mode: &str) -> Lsp {
    let script = fixture.write("server.sh", include_str!("lsp/server.sh"));
    let server = Server {
        command: "/bin/sh".into(),
        args: vec![
            script.to_str().unwrap().into(),
            mode.into(),
            fixture.0.join("launches").to_str().unwrap().into(),
        ],
        languages: vec!["rust".into()],
        root_markers: vec!["Cargo.toml".into()],
        initialization_options: toml::Value::Table(toml::Table::new()),
        settings: toml::Value::Table(toml::Table::new()),
    };
    Lsp {
        enabled: true,
        initialize_timeout_ms: 1000,
        shutdown_timeout_ms: 250,
        servers: std::collections::BTreeMap::from([("fixture".into(), server)]),
        ..Lsp::default()
    }
}

fn wait(client: &Client) -> Status {
    let deadline = Instant::now() + Duration::from_secs(4);
    loop {
        let status = client.status();
        if status != Status::Starting {
            return status;
        }
        assert!(
            Instant::now() < deadline,
            "transport did not finish initialization"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

// A real local child exercises direct argv, fragmentation, combined frames,
// byte-counted UTF-8, root alias sharing and the actual shutdown/exit exchange.
#[test]
fn owned_transport() {
    let fixture = Fixture::new();
    fixture.write("Cargo.toml", "");
    let file = fixture.write("src/main.rs", "fn main() {}\n");
    let alias = fixture.0.join("alias");
    std::os::unix::fs::symlink(&fixture.0, &alias).unwrap();
    let mut client = Client::new(policy(&fixture, "ready"));
    assert_eq!(client.activate(&file, "rust", None), Status::Starting);
    assert!(
        matches!(wait(&client), Status::Ready(c) if c.encoding == Encoding::Utf8 && c.sync == 2 && c.completion)
    );
    assert!(matches!(
        client.activate(&alias.join("src/main.rs"), "rust", None),
        Status::Ready(_)
    ));
    assert_eq!(client.connection_count(), 1);
    drop(client);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("launches"))
            .unwrap()
            .lines()
            .count(),
        1
    );
    let events = std::fs::read_to_string(fixture.0.join("launches.events")).unwrap();
    assert!(events.contains("initialized\nshutdown\nexit\n"), "{events}");
    let text = std::fs::read_to_string(fixture.0.join("launches.initialize")).unwrap();
    let request: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        request["params"]["capabilities"]["textDocument"]["completion"]["completionItem"]["snippetSupport"],
        false
    );
}

// Failure variants must become unavailable once, reap their owned process, and
// remain retained so repeated activation cannot form a respawn loop.
#[test]
fn rejected_peers() {
    for mode in [
        "eof",
        "header",
        "oversized",
        "long-header",
        "bad-json",
        "failure",
        "encoding",
        "no-sync",
        "timeout",
    ] {
        let fixture = Fixture::new();
        let file = fixture.write("main.rs", "");
        let mut policy = policy(&fixture, mode);
        policy.initialize_timeout_ms = 150;
        let mut client = Client::new(policy);
        client.activate(&file, "rust", None);
        assert!(matches!(wait(&client), Status::Failed(_)), "{mode}");
        assert!(
            matches!(client.activate(&file, "rust", None), Status::Failed(_)),
            "{mode}"
        );
        drop(client);
        let launches = std::fs::read_to_string(fixture.0.join("launches")).unwrap();
        assert_eq!(launches.lines().count(), 1, "{mode}");
        let pid: u32 = launches.trim().parse().unwrap();
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "{mode}: child remains"
        );
    }
}

// Disabled activation performs no source discovery or process work. A missing
// configured executable becomes a recoverable status rather than an edit error.
#[test]
fn optional_prerequisites() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "");
    let mut policy = policy(&fixture, "ready");
    policy.enabled = false;
    let mut client = Client::new(policy.clone());
    assert_eq!(
        client.activate(Path::new("/absent/source.rs"), "rust", None),
        Status::Disabled
    );
    assert_eq!(client.connection_count(), 0);
    assert!(!fixture.0.join("launches").exists());
    policy.enabled = true;
    policy.servers.get_mut("fixture").unwrap().command =
        fixture.0.join("absent-server").to_str().unwrap().into();
    let mut client = Client::new(policy);
    client.activate(&file, "rust", None);
    assert!(matches!(wait(&client), Status::Missing(_)));
    assert_eq!(client.connection_count(), 1);
}

// Stderr bursts are drained separately and retain only the bounded diagnostic
// tail; a ready idle connection has no initialization deadline or polling work.
#[test]
fn stderr_bound() {
    let fixture = Fixture::new();
    let file = fixture.write("main.rs", "");
    let mut client = Client::new(policy(&fixture, "stderr"));
    client.activate(&file, "rust", None);
    assert!(matches!(wait(&client), Status::Ready(_)));
    assert_eq!(client.stderr_bytes(), 65536);
    assert!(!client.starting());
}

// Root lookup is nearest-first, bounded by an explicitly opened containing
// directory; unrelated trees and absent markers use individual-file policy.
#[test]
fn root_boundaries() {
    let fixture = Fixture::new();
    fixture.write("Cargo.toml", "");
    let file = fixture.write("project/src/main.rs", "");
    let project = fixture.0.join("project");
    let markers = vec!["Cargo.toml".into()];
    let root = |tree, markers: &[String]| rev::lsp::roots::resolve(&file, tree, markers).unwrap();
    assert_eq!(root(None, &markers), fixture.0);
    assert_eq!(root(Some(project.as_path()), &markers), project);
    assert_eq!(root(None, &[]), file.parent().unwrap());
    fixture.write("project/Cargo.toml", "");
    assert_eq!(root(None, &markers), project);
    assert_eq!(root(Some(Path::new("/unrelated")), &markers), project);
}

// A connection cap is enforced across distinct roots. Disable/enable retires
// the old generation before admitting a new launch, without blocking activation.
#[test]
fn connection_budget() {
    let fixture = Fixture::new();
    let file = fixture.write("one/main.rs", "");
    let other = fixture.write("two/main.rs", "");
    let mut policy = policy(&fixture, "ready");
    policy.max_servers = 1;
    let mut client = Client::new(policy);
    client.activate(&file, "rust", None);
    assert!(matches!(wait(&client), Status::Ready(_)));
    assert_eq!(client.activate(&other, "rust", None), Status::Limited);
    let started = Instant::now();
    client.set_enabled(false);
    assert!(started.elapsed() < Duration::from_millis(100));
    let deadline = Instant::now() + Duration::from_secs(2);
    while client.connection_count() != 0 {
        client.collect();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    client.set_enabled(true);
    assert_eq!(client.activate(&other, "rust", None), Status::Starting);
    assert!(matches!(wait(&client), Status::Ready(_)));
}
