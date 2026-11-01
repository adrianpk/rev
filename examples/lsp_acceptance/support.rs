use crate::support::Result;
use rev::{lsp::Status, session::Session};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub(super) fn wait(mut condition: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        if Instant::now() >= deadline {
            return Err("owned protocol observation exceeded five seconds".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}
pub(super) fn synchronized(session: &mut Session) -> Result<rev::lsp::DocumentVersion> {
    wait(|| {
        session.expire_input(Instant::now());
        session.lsp_document_version().is_some()
    })?;
    session
        .lsp_document_version()
        .ok_or("missing current document identity".into())
}
pub(super) fn peer(root: &Path) -> Result<String> {
    std::fs::create_dir_all(root)?;
    let script = root.join("server.sh");
    std::fs::write(&script, include_str!("../../tests/lsp/candidates.sh"))?;
    let args = [
        script.to_str().ok_or("non-UTF-8 fixture")?.to_owned(),
        "resolve".into(),
        root.join("peer")
            .to_str()
            .ok_or("non-UTF-8 fixture")?
            .to_owned(),
    ];
    Ok(format!(
        "[completion]\nautomatic = false\n[lsp]\nrequest_timeout_ms = 500\nshutdown_timeout_ms = 200\n[lsp.servers.rust]\ncommand = '/bin/sh'\nargs = {args:?}\n"
    ))
}
pub(super) fn frames(root: &Path) -> Vec<Value> {
    std::fs::read_to_string(root.join("peer.frames"))
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}
pub(super) fn count(root: &Path, method: &str) -> usize {
    frames(root)
        .iter()
        .filter(|frame| frame["method"] == method)
        .count()
}
pub(super) fn pid(root: &Path) -> Result<String> {
    let pid = std::fs::read_to_string(root.join("peer.pid"))?
        .trim()
        .to_owned();
    pid.parse::<u32>()?;
    Ok(pid)
}
pub(super) fn reaped(pid: &str) -> Result<()> {
    wait(|| !PathBuf::from(format!("/proc/{pid}")).exists())
}
pub(super) fn failed(session: &Session) -> bool {
    matches!(session.lsp_status(), Status::Failed(_))
}
