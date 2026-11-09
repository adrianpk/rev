use super::lifecycle::{Editor, lines, stopped, text};
use rev::{
    command::{Command, PluginCommand},
    plugins::lifecycle::State,
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

#[path = "faults/pty.rs"]
mod pty;
#[path = "faults/resources.rs"]
mod resources;

// Malformed/oversized traffic, EOF/crash and foreign handshake agreement fail
// before handlers publish. A failure leaves all ordinary source text intact.
#[test]
fn peer_failures() {
    for mode in [
        "crash",
        "eof-after-init",
        "noise",
        "truncated",
        "oversize",
        "foreign-api",
        "foreign-generation",
        "undeclared",
    ] {
        let mut editor = Editor::new("non-modal", mode, "", false);
        let source = text(&editor.session);
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().state == State::Failed && e.status().pending == 0);
        assert!(!editor.status().diagnostic.is_empty(), "{mode}");
        assert!(
            editor.status().cleanup.contains("group reaped"),
            "{mode}: {:?}",
            editor.status()
        );
        assert_eq!(text(&editor.session), source, "{mode}");
        if let Some(pid) = editor.status().pid {
            assert!(
                !Path::new(&format!("/proc/{pid}")).exists(),
                "{mode}: leader not reaped"
            );
        }
    }
}

// Owned descendant settlement includes TERM-resistant members with open pipes.
// An independent editor's package must remain alive through the first shutdown.
#[test]
fn group_shutdown() {
    let mut first = Editor::new("non-modal", "descendant", "", true);
    let mut second = Editor::new("non-modal", "echo", "", true);
    first.wait(|e| e.status().state == State::Active && e.path("descendant").exists());
    second.wait(|e| e.status().state == State::Active);
    let leader = first.status().pid.unwrap();
    let descendant: u32 = fs::read_to_string(first.path("descendant"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let other = second.status().pid.unwrap();
    assert_ne!(leader, other);
    first.manage(PluginCommand::Deactivate);
    first.wait(|e| e.status().state == State::Inactive);
    assert!(!Path::new(&format!("/proc/{leader}")).exists());
    assert!(stopped(descendant));
    assert!(
        first
            .status()
            .cleanup
            .contains("escaped descendant settlement uncertain")
    );
    assert!(!stopped(other));
    assert_eq!(text(&first.session), "source 🦀\n");
    first.session.shutdown_plugins().unwrap();
    assert!(!stopped(other));
}

// The real SDK sends a package-to-host request and receives a correlated typed
// rejection. A noisy diagnostic stream stays bounded and cannot inject escapes.
#[test]
fn rpc_and_stderr() {
    for mode in ["rpc", "stderr"] {
        let mut editor = Editor::new("non-modal", mode, "", true);
        editor.wait(|e| e.status().state == State::Active);
        if mode == "rpc" {
            editor.wait(|e| e.path("errors").exists());
            let errors = lines(&editor.path("errors"));
            assert_eq!(errors[0]["error"]["data"]["kind"], "unsupported_method");
            assert_eq!(errors[0]["error"]["code"], -32601);
            assert!(
                errors[0]["error"]["data"]["request"]
                    .as_str()
                    .unwrap()
                    .starts_with(&format!("{}:", editor.status().generation.unwrap()))
            );
        } else {
            editor.wait(|e| e.status().stderr.contains("END"));
            let status = editor.status();
            assert_eq!(status.stderr_bytes, 128 * 1024);
            assert!(!status.stderr.contains('\x1b'));
            assert!(status.stderr.contains("END"));
        }
        editor.wait(|e| e.session.remaining_input(Instant::now()).is_none());
        editor.manage(PluginCommand::Deactivate);
        editor.wait(|e| e.status().state == State::Inactive);
    }
}

// Exercise the actual startup deadline without shortening the production
// contract. A silent peer cannot keep activation or request permits forever.
#[test]
fn startup_deadline() {
    let mut editor = Editor::new("non-modal", "timeout", "", true);
    editor.wait(|e| e.path("offers").exists());
    editor.wait(|e| e.status().state == State::Failed);
    assert!(editor.status().diagnostic.contains("deadline exceeded"));
    assert_eq!(editor.status().pending, 0);
    assert_eq!(text(&editor.session), "source 🦀\n");
}

// Isolate exact descriptor evidence from unrelated test workers. Repeated
// failed execs release their pipes, control eventfd and pending registrations.
#[test]
fn spawn_descriptors() {
    if std::env::var_os("REV_PLUGIN_FD_FIXTURE").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "faults::spawn_descriptors", "--test-threads=1"])
            .env("REV_PLUGIN_FD_FIXTURE", "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    let mut editor = Editor::new("non-modal", "missing", "", false);
    let baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    for _ in 0..8 {
        editor.call("field.notes.collect");
        editor.wait(|e| e.status().state == State::Failed && e.status().pending == 0);
    }
    editor.session.shutdown_plugins().unwrap();
    let mut live = Editor::new("non-modal", "echo", "", false);
    let live_baseline = fs::read_dir("/proc/self/fd").unwrap().count();
    for mode in ["echo", "noise", "echo", "truncated", "echo"] {
        fs::write(live.path("mode"), mode).unwrap();
        live.manage(PluginCommand::Restart);
        live.wait(|e| matches!(e.status().state, State::Active | State::Failed));
        if mode == "echo" {
            let inherited: Vec<u32> =
                serde_json::from_slice(&fs::read(live.path("inherited-fds")).unwrap()).unwrap();
            assert_eq!(
                inherited,
                vec![0, 1, 2],
                "private host descriptors crossed exec"
            );
            live.manage(PluginCommand::Deactivate);
            live.wait(|e| e.status().state == State::Inactive);
        }
    }
    live.session.shutdown_plugins().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while fs::read_dir("/proc/self/fd").unwrap().count() != live_baseline
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        fs::read_dir("/proc/self/fd").unwrap().count(),
        live_baseline
    );
    drop(live);
    editor.session.shutdown_plugins().unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while fs::read_dir("/proc/self/fd").unwrap().count() != baseline && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(fs::read_dir("/proc/self/fd").unwrap().count(), baseline);
    assert_eq!(text(&editor.session), "source 🦀\n");
}

// Backpressure from a live peer that never reads and a valid-request flood must
// leave native input and an independent editor live. All signals remain owned.
#[test]
fn pipe_backpressure() {
    for mode in ["stall-input", "flood"] {
        let mut other = Editor::new("non-modal", "echo", "", true);
        other.wait(|e| e.status().state == State::Active);
        let other_pid = other.status().pid.unwrap();
        let mut editor = Editor::new("non-modal", mode, "", true);
        editor.wait(|e| e.status().state == State::Active);
        let pid = editor.status().pid.unwrap();
        let began = Instant::now();
        if mode == "stall-input" {
            editor.wait(|e| e.path("stalled").exists());
            for _ in 0..16 {
                editor.call("field.notes.collect");
            }
            assert_eq!(editor.status().pending, 16);
            editor.call("field.notes.collect");
            assert!(editor.session.echo().contains("16 per package"));
        }
        editor.session.insert('!');
        assert_eq!(text(&editor.session), "!source 🦀\n");
        assert!(
            began.elapsed() < Duration::from_secs(1),
            "{mode}: native input stalled"
        );
        if mode == "flood" {
            editor.wait(|e| {
                e.status().state == State::Failed
                    && e.status().pending == 0
                    && e.status().cleanup.contains("group reaped")
            });
            assert!(
                editor
                    .status()
                    .diagnostic
                    .contains("outbound queue limit exceeded"),
                "{:?}",
                editor.status()
            );
        } else {
            editor.session.execute(Command::KeyboardQuit);
            editor.wait(|e| e.status().state == State::Inactive && e.status().pending == 0);
        }
        assert_eq!(editor.status().pending, 0);
        assert_eq!(text(&editor.session), "!source 🦀\n");
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        assert!(!stopped(other_pid));
        other.call("field.notes.collect");
        other.wait(|e| e.status().pending == 0 && e.path("calls").exists());
        assert_eq!(other.status().pid, Some(other_pid));
    }
}
