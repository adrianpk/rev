#![forbid(unsafe_code)]
#[path = "pi/discovery.rs"]
mod discovery;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "ai/support.rs"]
mod support;

use fixture::Fixture;
use rev::ai::pi::Operation;
use rev::config::Pi;
use std::fs;
use std::time::{Duration, Instant};

fn start(f: &Fixture, mode: &str) -> Operation {
    let executable = support::peer(f, mode);
    let policy = Pi {
        command: executable.to_str().unwrap().into(),
        request_timeout_ms: 1000,
        max_result_bytes: 256,
        ..Pi::default()
    };
    Operation::start(
        policy,
        f.0.clone(),
        if mode == "stdin_stall" {
            "x".repeat(900_000)
        } else {
            "{\"instruction\":\"fixture only\"}".into()
        },
        37,
    )
    .unwrap()
}
fn finish(operation: &mut Operation) -> Result<rev::ai::pi::Output, String> {
    support::until(|| operation.finished());
    operation.take().unwrap()
}

// Real subprocess sockets prove acceptance, final-message and agent_end events
// remain pending until agent_settled, including reversed event/response order.
#[test]
fn settled_completion() {
    let f = Fixture::new();
    let mut operation = start(&f, "settle_gate");
    support::until(|| f.0.join("partial").exists());
    assert!(operation.accepted());
    assert!(!operation.finished());
    fs::write(f.0.join("release"), "").unwrap();
    let result = finish(&mut operation).unwrap();
    assert_eq!(result.text, "A selection explanation.\u{2028}With Unicode.");
    assert_eq!(result.model, "fixture/controlled");
    assert!(operation.take().is_none());
    assert!(f.0.join("closed").exists());
    for mode in ["late_accept", "stderr"] {
        let f = Fixture::new();
        assert!(finish(&mut start(&f, mode)).is_ok(), "{mode}");
    }
}

// Each rejected protocol boundary must preserve a failure rather than publish
// partial output, leak provider diagnostics, or fall back to another backend.
#[test]
fn rejected_boundaries() {
    for mode in [
        "version",
        "history",
        "rejected",
        "correlation",
        "malformed",
        "no_lf",
        "tool",
        "ui",
        "turns",
        "auth",
        "eof",
        "aborted",
        "invalid",
        "oversize",
        "handled",
        "queued",
        "duplicate",
        "empty_settled",
        "truncated",
    ] {
        let f = Fixture::new();
        let result = finish(&mut start(&f, mode));
        assert!(result.is_err(), "{mode}");
        assert!(!result.err().unwrap().contains("SECRET-FIXTURE"), "{mode}");
        if mode == "version" {
            assert!(!f.0.join("rpc-started").exists());
        }
    }
}

// Explicit cancellation sends clear_queue before abort, fences late successful
// records, closes stdin and reaps this owned process. No replacement is queued.
#[test]
fn cancel_and_stop() {
    let f = Fixture::new();
    let mut operation = start(&f, "late_cancel");
    support::until(|| operation.accepted());
    operation.cancel();
    assert!(finish(&mut operation).unwrap_err().contains("canceled"));
    let requests: Vec<serde_json::Value> = fs::read_to_string(f.0.join("requests"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(
        requests
            .iter()
            .map(|v| v["type"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["get_state", "prompt", "clear_queue", "abort"]
    );
    let pid = fs::read_to_string(f.0.join("pid")).unwrap();
    assert!(!std::path::Path::new("/proc").join(pid).exists());
    assert!(f.0.join("closed").exists());
}

// A silent peer has a generation deadline, and a noncooperative abort/EOF peer
// reaches the documented five-second cleanup and two-second forced stop grace.
#[test]
fn deadlines() {
    for mode in ["hold", "stubborn", "version_hang", "stdin_stall"] {
        let f = Fixture::new();
        let started = Instant::now();
        let result = finish(&mut start(&f, mode));
        assert!(result.is_err(), "{mode}");
        assert!(started.elapsed() < Duration::from_secs(11), "{mode}");
        for name in ["pid", "version-pid"] {
            if let Ok(pid) = fs::read_to_string(f.0.join(name)) {
                assert!(!std::path::Path::new("/proc").join(pid).exists());
            }
        }
    }
}

// Logical editor bytes and encoded LF bytes are separate admission limits;
// oversized input must fail synchronously before any external process starts.
#[test]
fn input_admission() {
    let f = Fixture::new();
    let command = support::peer(&f, "normal").to_str().unwrap().to_owned();
    for (max_input_bytes, message) in [(1023, "é".repeat(512)), (4_194_304, "\n".repeat(1_100_000))]
    {
        let policy = Pi {
            command: command.clone(),
            max_input_bytes,
            ..Pi::default()
        };
        assert!(Operation::start(policy, f.0.clone(), message, 1).is_err());
        assert!(!f.0.join("version-pid").exists());
        assert!(!f.0.join("rpc-started").exists());
    }
}
