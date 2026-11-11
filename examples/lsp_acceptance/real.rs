use crate::support::{Result, cpu_ticks, idle, key, open, peak_rss, select, source};
use crossterm::event::KeyCode;
use rev::{
    command::{Command, CompletionCommand as C, History, Motion},
    session::{CompletionSource, Session},
};
use std::{
    path::Path,
    process::Command as Process,
    time::{Duration, Instant},
};

const DISK: &str = "struct Widget { disk_field: u32 }\nfn main() { let item = Widget { disk_field: 7 }; let _ = item.disk_field; }\n";
const UNSAVED: &str = "struct Widget { unsaved_field: u32 }\nfn main() { let item = Widget { unsaved_field: 7 }; let _ = item.uns; }\n";

pub(crate) fn run(root: &Path) -> Result<()> {
    let root = root.join("real");
    std::fs::create_dir(&root)?;
    let mut process = Process::new("timeout");
    process
        .args(["--kill-after=1", "60"])
        .arg(std::env::current_exe()?)
        .arg("--real")
        .arg(&root)
        .env("CARGO_NET_OFFLINE", "true");
    isolate(&mut process, &root)?;
    assert!(
        process.status()?.success(),
        "isolated real Rust journey failed"
    );
    Ok(())
}
fn isolate(process: &mut Process, root: &Path) -> Result<()> {
    for (key, name) in [
        ("HOME", "home"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_STATE_HOME", "state"),
        ("XDG_CACHE_HOME", "cache"),
        ("CARGO_HOME", "cargo"),
        ("CARGO_TARGET_DIR", "target"),
    ] {
        let path = root.join(name);
        std::fs::create_dir(&path)?;
        process.env(key, path);
    }
    Ok(())
}
pub(crate) fn exercise(root: &Path) -> Result<()> {
    std::fs::create_dir(root.join("workspace"))?;
    std::fs::write(
        root.join("workspace/Cargo.toml"),
        "[package]\nname = 'owned_lsp'\nversion = '0.0.0'\nedition = '2024'\n[workspace]\n[[bin]]\nname = 'owned_lsp'\npath = 'main.rs'\n",
    )?;
    std::fs::write(
        root.join("config.toml"),
        "[completion]\nautomatic = false\n[lsp]\nenabled = true\n",
    )?;
    let settings = rev::config::prepare(&root.join("config.toml"));
    assert!(settings.diagnostic.is_none());
    let mut session = open(
        root,
        "main.rs",
        &(DISK.to_owned() + "|"),
        &settings.snapshot,
    )?;
    session.replace(0..DISK.chars().count(), UNSAVED)?;
    session.execute(Command::Motion(Motion::BufferEnd));
    for _ in 0..4 {
        session.execute(Command::Motion(Motion::BackwardChar));
    }
    semantic(&mut session)?;
    accept_undo(&mut session)?;
    lifetime(session, root)
}
fn semantic(session: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(35);
    let start = Instant::now();
    loop {
        session.execute(Command::Completion(C::AtPoint));
        finish(session, deadline)?;
        if matches!(
            session.completion_source(),
            Some(CompletionSource::Semantic { .. })
        ) {
            select(session, "unsaved_field");
            println!(
                "MEASURE installed unsaved Rust semantic readiness_us={}",
                start.elapsed().as_micros()
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "real semantic readiness expired: {:?}",
                session.lsp_status()
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
fn finish(session: &mut Session, deadline: Instant) -> Result<()> {
    loop {
        session.expire_input(Instant::now());
        session.poll_frame()?;
        if session.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("real Rust work exceeded its deadline".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn accept_undo(session: &mut Session) -> Result<()> {
    let history = session.buffer()?.history().len();
    key(session, KeyCode::Tab);
    finish(session, Instant::now() + Duration::from_secs(5))?;
    assert_eq!(
        source(session),
        UNSAVED.replace("item.uns;", "item.unsaved_field;")
    );
    assert_eq!(session.buffer()?.history().len(), history + 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(source(session), UNSAVED);
    session.execute(Command::Completion(C::Cancel));
    finish(session, Instant::now() + Duration::from_secs(5))?;
    idle(session);
    Ok(())
}
fn lifetime(session: Session, root: &Path) -> Result<()> {
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/main.rs"))?,
        DISK
    );
    let pids = children()?;
    assert_eq!(pids.len(), 1, "one owned installed server per Rust root");
    let pid = &pids[0];
    println!(
        "MEASURE separate installed server: peak_rss={}, cpu_ticks={}, threads={}",
        peak_rss(pid)?,
        cpu_ticks(pid)?,
        threads(pid)?
    );
    println!(
        "MEASURE semantic driver peak_rss={}; server memory is separate",
        peak_rss("self")?
    );
    let start = Instant::now();
    drop(session);
    super::support::reaped(pid)?;
    assert!(start.elapsed() < Duration::from_secs(5));
    println!(
        "PASS installed rust-analyzer: semantic unsaved_field, one undo, disk unchanged, no idle deadline, owned server reaped"
    );
    Ok(())
}
fn threads(pid: &str) -> Result<String> {
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("Threads:"))
        .ok_or("missing server thread count")?
        .trim()
        .to_owned())
}
fn children() -> Result<Vec<String>> {
    let mut result = std::collections::BTreeSet::new();
    for task in std::fs::read_dir("/proc/self/task")?.flatten() {
        let source = std::fs::read_to_string(task.path().join("children"))?;
        for pid in source.split_whitespace() {
            if analyzer(pid)? {
                result.insert(pid.to_owned());
            }
        }
    }
    Ok(result.into_iter().collect())
}
fn analyzer(pid: &str) -> Result<bool> {
    let cmd = std::fs::read(format!("/proc/{pid}/cmdline"))?;
    let executable = cmd.split(|b| *b == 0).next().ok_or("empty child argv")?;
    Ok(Path::new(std::str::from_utf8(executable)?)
        .file_name()
        .is_some_and(|name| name == "rust-analyzer"))
}
