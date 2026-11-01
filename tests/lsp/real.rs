use super::{
    Fixture, Status,
    candidates::{key, text},
};
use crossterm::event::KeyCode;
use rev::{
    buffer::Buffers,
    command::{Command, CompletionCommand, History, Motion},
    session::{CompletionSource, Session},
    view::{Size, View},
};
use std::os::unix::process::CommandExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Process,
    time::{Duration, Instant},
};

const CHILD_ROOT: &str = "REV_LSP_REAL_ROOT";
const DISK: &str = "struct Widget { disk_field: u32 }\nfn main() { let item = Widget { disk_field: 7 }; let _ = item.disk_field; }\n";
const UNSAVED: &str = "struct Widget { unsaved_field: u32 }\nfn main() { let item = Widget { unsaved_field: 7 }; let _ = item.uns; }\n";

// The installed server must return a field present only in unsaved canonical
// text. A child test process isolates HOME/XDG/Cargo before any library starts;
// its owned process group bounds failed startup and prevents leaked descendants.
#[test]
fn installed_rust() {
    if let Some(root) = std::env::var_os(CHILD_ROOT) {
        exercise(Path::new(&root));
        return;
    }
    let f = Fixture::new();
    let log = fs::File::create(f.0.join("real.log")).unwrap();
    let mut command = Process::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "real::installed_rust",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_ROOT, &f.0)
        .env("CARGO_NET_OFFLINE", "true")
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .process_group(0);
    for (key, directory) in [
        ("HOME", "home"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_STATE_HOME", "state"),
        ("XDG_CACHE_HOME", "cache"),
        ("CARGO_HOME", "cargo"),
        ("CARGO_TARGET_DIR", "target"),
    ] {
        let path = f.0.join(directory);
        fs::create_dir(&path).unwrap();
        command.env(key, path);
    }
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            assert!(
                Process::new("/usr/bin/kill")
                    .args(["-KILL", "--", &format!("-{}", child.id())])
                    .status()
                    .unwrap()
                    .success()
            );
            break child.wait().unwrap();
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(f.0.join("real.log")).unwrap();
    println!(
        "{}",
        String::from_utf8_lossy(&output.as_bytes()[output.len().saturating_sub(16384)..])
    );
    assert!(
        status.success(),
        "isolated installed rust-analyzer fixture failed: {status}"
    );
}

fn exercise(root: &Path) {
    for program in ["rust-analyzer", "rustc", "cargo"] {
        let version = Process::new(program).arg("--version").output().unwrap();
        assert!(
            version.status.success(),
            "required installed {program} is unavailable"
        );
        println!("{}", String::from_utf8_lossy(&version.stdout).trim());
    }
    fs::create_dir(root.join("src")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = 'owned_lsp'\nversion = '0.0.0'\nedition = '2024'\n[workspace]\n",
    )
    .unwrap();
    let file = root.join("src/main.rs");
    fs::write(&file, DISK).unwrap();
    let mut session = real_session(root, &file);
    session.replace(0..DISK.chars().count(), UNSAVED).unwrap();
    session.execute(Command::Motion(Motion::BufferEnd));
    for _ in 0..4 {
        session.execute(Command::Motion(Motion::BackwardChar));
    }
    let deadline = Instant::now() + Duration::from_secs(35);
    let mut attempts = 0;
    loop {
        attempts += 1;
        session.execute(Command::Completion(CompletionCommand::AtPoint));
        finish(&mut session, deadline);
        let admitted = session.completion().is_some_and(|popup| {
            popup
                .candidates()
                .iter()
                .any(|item| item.text() == "unsaved_field")
        });
        if admitted
            && matches!(
                session.completion_source(),
                Some(CompletionSource::Semantic { .. })
            )
        {
            assert!(matches!(
                session.completion_source(),
                Some(CompletionSource::Semantic { .. })
            ));
            break;
        }
        println!(
            "attempt {attempts}: {:?}, {:?}",
            session.lsp_status(),
            session.completion_source()
        );
        assert!(
            Instant::now() < deadline,
            "real unsaved semantic field was never admitted"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let labels: Vec<_> = session
        .completion()
        .unwrap()
        .candidates()
        .iter()
        .map(|item| item.text().to_owned())
        .collect();
    assert!(!labels.iter().any(|label| label == "disk_field"));
    let selected = labels
        .iter()
        .position(|label| label == "unsaved_field")
        .unwrap();
    for _ in 0..selected {
        key(&mut session, KeyCode::Down);
    }
    let history = session.buffer().unwrap().history().len();
    key(&mut session, KeyCode::Tab);
    finish(&mut session, deadline);
    assert_eq!(
        text(&session),
        UNSAVED.replace("item.uns;", "item.unsaved_field;")
    );
    assert_eq!(session.buffer().unwrap().history().len(), history + 1);
    session.execute(Command::History(History::Undo));
    assert_eq!(text(&session), UNSAVED);
    assert_eq!(fs::read_to_string(file).unwrap(), DISK);
    assert!(matches!(session.lsp_status(), Status::Ready(_)));
    let pids = analyzer_children();
    assert_eq!(pids.len(), 1, "one owned real server per root");
    let start = Instant::now();
    drop(session);
    assert!(start.elapsed() < Duration::from_secs(5));
    for pid in &pids {
        assert!(
            !Path::new(&format!("/proc/{pid}")).exists(),
            "owned rust-analyzer survived exit"
        );
    }
    println!(
        "installed semantic completion: unsaved_field; attempts={attempts}; undo restored unsaved text; disk preserved; owned PIDs {pids:?} reaped"
    );
}

fn real_session(root: &Path, file: &Path) -> Session {
    let config = root.join("config.toml");
    fs::write(
        &config,
        "[completion]\nautomatic = false\n[lsp]\nenabled = true\n",
    )
    .unwrap();
    let settings = rev::config::prepare(&config);
    assert!(settings.diagnostic.is_none(), "{:?}", settings.diagnostic);
    let mut buffers = Buffers::with_settings(settings.snapshot).unwrap();
    let id = buffers.visit(file).unwrap();
    let view = View::new(buffers.get(id).unwrap());
    Session::new(
        buffers,
        view,
        Size {
            width: 80,
            height: 24,
        },
    )
    .unwrap()
}
fn finish(session: &mut Session, deadline: Instant) {
    loop {
        let now = Instant::now();
        session.expire_input(now);
        if session.remaining_input(now).is_none() {
            return;
        }
        assert!(now < deadline, "bounded real-server request expired");
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn analyzer_children() -> Vec<u32> {
    let mut pids = std::collections::BTreeSet::new();
    for task in fs::read_dir("/proc/self/task").unwrap().flatten() {
        let source = fs::read_to_string(task.path().join("children")).unwrap_or_default();
        for pid in source.split_whitespace() {
            let cmd = fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
            let executable = cmd.split(|b| *b == 0).next().unwrap_or_default();
            if PathBuf::from(String::from_utf8_lossy(executable).as_ref())
                .file_name()
                .is_some_and(|name| name == "rust-analyzer")
            {
                pids.insert(pid.parse().unwrap());
            }
        }
    }
    pids.into_iter().collect()
}
