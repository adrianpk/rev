use crate::support::{Result, cpu_ticks, idle, key, open, peak_rss, select, source};
use crossterm::event::KeyCode;
use rev::{
    command::{Command, CompletionCommand, History, Motion},
    input::EditingState,
    session::{CompletionSource, Session},
};
use std::{
    path::Path,
    process::Command as Process,
    time::{Duration, Instant},
};

const DISK: &str = "struct Widget { disk_field: u32 }\nfn main() { let item = Widget { disk_field: 7 }; let _ = item.disk_field; }\n";
const UNSAVED: &str = "struct Widget { unsaved_field: u32 }\nfn main() { let item = Widget { unsaved_field: 7 }; let _ = item.uns; }\n";

pub(super) fn run(root: &Path) -> Result<()> {
    let root = root.join("semantic");
    std::fs::create_dir(&root)?;
    let mut process = Process::new("timeout");
    process
        .args(["--kill-after=1", "60"])
        .arg(std::env::current_exe()?)
        .arg("--semantic")
        .arg(&root)
        .env("CARGO_NET_OFFLINE", "true");
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
    assert!(
        process.status()?.success(),
        "owned modal Rust semantic journey failed"
    );
    Ok(())
}

pub(super) fn exercise(root: &Path) -> Result<()> {
    std::fs::create_dir(root.join("workspace"))?;
    std::fs::write(
        root.join("workspace/Cargo.toml"),
        "[package]\nname = 'modal_fixture'\nversion = '0.0.0'\nedition = '2024'\n[workspace]\n[[bin]]\nname = 'modal_fixture'\npath = 'main.rs'\n",
    )?;
    std::fs::write(
        root.join("config.toml"),
        "[editor]\ninteraction_profile = 'modal'\n[tree]\nvisible = false\n[completion]\nautomatic = false\n[lsp]\nenabled = true\n",
    )?;
    let config = rev::config::prepare(&root.join("config.toml"));
    assert!(config.diagnostic.is_none());
    let mut editor = open(root, "main.rs", &format!("{DISK}|"), &config.snapshot)?;
    editor.replace(0..DISK.chars().count(), UNSAVED)?;
    editor.execute(Command::Motion(Motion::BufferEnd));
    for _ in 0..4 {
        editor.execute(Command::Motion(Motion::BackwardChar));
    }
    key(&mut editor, KeyCode::Char('R'));
    assert_eq!(editor.editing_state(), Some(EditingState::Replace));
    semantic(&mut editor)?;
    accept(&mut editor)?;
    lifetime(root, editor)
}

fn semantic(editor: &mut Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(35);
    let started = Instant::now();
    loop {
        editor.execute(Command::Completion(CompletionCommand::AtPoint));
        finish(editor, deadline)?;
        if matches!(
            editor.completion_source(),
            Some(CompletionSource::Semantic { .. })
        ) {
            select(editor, "unsaved_field");
            println!(
                "MEASURE installed modal Rust readiness_us={}",
                started.elapsed().as_micros()
            );
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "modal semantic readiness expired: {:?}",
                editor.lsp_status()
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn finish(editor: &mut Session, deadline: Instant) -> Result<()> {
    loop {
        editor.expire_input(Instant::now());
        editor.poll_frame()?;
        if editor.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("owned modal semantic work exceeded its deadline".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn accept(editor: &mut Session) -> Result<()> {
    let history = editor.buffer()?.history().len();
    key(editor, KeyCode::Tab);
    finish(editor, Instant::now() + Duration::from_secs(5))?;
    assert_eq!(
        source(editor),
        UNSAVED.replace("item.uns;", "item.unsaved_field;")
    );
    assert_eq!(editor.editing_state(), Some(EditingState::Replace));
    assert_eq!(editor.buffer()?.history().len(), history + 1);
    editor.execute(Command::History(History::Undo));
    assert_eq!(source(editor), UNSAVED);
    editor.execute(Command::Completion(CompletionCommand::Cancel));
    finish(editor, Instant::now() + Duration::from_secs(5))?;
    idle(editor);
    Ok(())
}

fn lifetime(root: &Path, editor: Session) -> Result<()> {
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/main.rs"))?,
        DISK
    );
    let pids = children()?;
    assert_eq!(
        pids.len(),
        1,
        "one owned installed server for the Rust root"
    );
    let pid = &pids[0];
    println!(
        "MEASURE separate installed server: peak_rss={} cpu_ticks={}; modal driver peak_rss={}",
        peak_rss(pid)?,
        cpu_ticks(pid)?,
        peak_rss("self")?
    );
    let started = Instant::now();
    drop(editor);
    while Path::new(&format!("/proc/{pid}")).exists() {
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "owned server was not reaped"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    println!(
        "PASS installed rust-analyzer: unsaved semantic field in Replace, one undo, disk unchanged, no idle deadline and owned server reaped"
    );
    Ok(())
}

fn children() -> Result<Vec<String>> {
    let mut result = std::collections::BTreeSet::new();
    for task in std::fs::read_dir("/proc/self/task")?.flatten() {
        let children = std::fs::read_to_string(task.path().join("children"))?;
        for pid in children.split_whitespace() {
            let cmd = std::fs::read(format!("/proc/{pid}/cmdline"))?;
            let executable = cmd
                .split(|byte| *byte == 0)
                .next()
                .ok_or("empty owned child argv")?;
            if Path::new(std::str::from_utf8(executable)?)
                .file_name()
                .is_some_and(|name| name == "rust-analyzer")
            {
                result.insert(pid.to_owned());
            }
        }
    }
    Ok(result.into_iter().collect())
}
