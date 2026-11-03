use super::support::*;
use rev::{ai::pi::Operation, config::Pi};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

// Explicit manual mode only. The repeatable gate never calls this operation.
// Existing Pi authentication stays in Pi; only toy text enters a fresh no-tools
// history. The accompanying guard additionally caps turns at four/calls at eight.
pub(crate) fn run(root: &Path) -> Result<()> {
    if std::env::var_os("REV_PI_LIVE").is_none() {
        return Err("live mode requires explicit REV_PI_LIVE=1".into());
    }
    fs::create_dir(root)?;
    let source = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
    fs::write(root.join("toy.rs"), source)?;
    let message = serde_json::json!({"instruction":"Explain in one short English sentence what the supplied toy Rust function returns. Respond only with a JSON object containing version:1 and text:string.","context":"input","history":"fresh input-only","observations":{"text":source},"dirty_sources":[]}).to_string();
    let policy = Pi {
        command: std::env::current_exe()?
            .to_str()
            .ok_or("non UTF-8 executable")?
            .into(),
        model: None,
        persist_sessions: false,
        idle_timeout_ms: 0,
        request_timeout_ms: 45_000,
        ..Pi::default()
    };
    let started = Instant::now();
    let mut operation = Operation::start(policy, root.canonicalize()?, message, 1)?;
    while !operation.finished() {
        if started.elapsed() >= Duration::from_secs(60) {
            operation.cancel();
            return Err("live operation exceeded one-minute deadline".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let result = operation.take().ok_or("missing live result")??;
    assert!(!result.text.trim().is_empty());
    assert!(!result.directory && result.history.contains("fresh input-only"));
    assert_eq!(fs::read_to_string(root.join("toy.rs"))?, source);
    assert!(!operation.cleanup_failed());
    println!(
        "PASS single live provider operation: model={} elapsed_ms={} fresh input-only; answer_bytes={}; source unchanged",
        result.model,
        started.elapsed().as_millis(),
        result.text.len()
    );
    // Print only counts, never authentication or provider conversation content.
    println!(
        "MEASURE live {}",
        fs::read_to_string(root.join("live-counts"))?
    );
    Ok(())
}
