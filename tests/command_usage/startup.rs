use crate::fixture::Fixture;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::command::Command;
use rev::config::Roots;
use rev::display::Band;
use rev::session::Session;
use rev::view::Size;
use std::path::Path;
use std::process::Command as Process;

#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../../examples/completion_acceptance/terminal.rs"]
mod terminal;

// Actual executable-style startup resolves isolated XDG roots in child processes.
// Pure constructors remain empty and passive even when valid user state exists.
#[test]
fn startup_owner() {
    if let Ok(profile) = std::env::var("REV_COMMAND_USAGE_CHILD") {
        startup_child(&profile);
        return;
    }
    for profile in ["non-modal", "modal"] {
        let fixture = Fixture::new();
        let config = fixture.write(
            "config/rev/config.toml",
            &format!("editor.interaction_profile='{profile}'\ntree.visible=false"),
        );
        let output = Process::new(std::env::current_exe().unwrap())
            .args(["--exact", "startup::startup_owner", "--nocapture"])
            .env("REV_COMMAND_USAGE_CHILD", profile)
            .env("HOME", fixture.0.join("home"))
            .env(
                "XDG_CONFIG_HOME",
                config.parent().unwrap().parent().unwrap(),
            )
            .env("XDG_DATA_HOME", fixture.0.join("data"))
            .env("XDG_STATE_HOME", fixture.0.join("state"))
            .env("XDG_CACHE_HOME", fixture.0.join("cache"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{profile}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn startup_child(profile: &str) {
    let roots = Roots::from_env().unwrap();
    let path = roots.state.join("command-history.toml");
    let size = Size {
        width: 80,
        height: 24,
    };
    let mut first = Session::startup(None, size).unwrap();
    assert!(!roots.state.exists());
    first.execute(Command::ExecuteExtended);
    first.handle(Event::Paste("undo".into()));
    first.handle(Event::Key(KeyEvent::new(
        KeyCode::Enter,
        KeyModifiers::NONE,
    )));
    first.flush_command_usage().unwrap();
    assert_eq!(super::store::disk(&path), ["undo"]);
    let mut restarted = Session::startup(None, size).unwrap();
    restarted.execute(Command::ExecuteExtended);
    let frame = restarted.frame().unwrap();
    assert_eq!(
        frame
            .rows()
            .iter()
            .find(|row| row.band() == Band::Completion)
            .unwrap()
            .text()
            .split_whitespace()
            .next(),
        Some("undo"),
        "{profile}"
    );
    let mut isolated = Session::empty(size).unwrap();
    isolated.execute(Command::ExecuteExtended);
    let frame = isolated.frame().unwrap();
    assert_eq!(
        frame
            .rows()
            .iter()
            .find(|row| row.band() == Band::Completion)
            .unwrap()
            .text()
            .split_whitespace()
            .next(),
        Some("abort-recursive-edit")
    );
    // Independent configuration and history failures share one non-modal echo;
    // neither owner can erase the other's operation or source diagnostic.
    std::fs::write(roots.config_file(), "editor.tab_width='invalid'").unwrap();
    std::fs::write(&path, "private-invalid-content").unwrap();
    let failed = Session::startup(None, size).unwrap();
    assert!(failed.echo().starts_with("Configuration: "));
    assert!(failed.echo().contains("tab_width") && failed.echo().contains("Command usage:"));
    assert!(!failed.echo().contains("private-invalid-content"));
}

// One shared native terminal journey verifies real startup, named dispatch,
// ordinary shutdown publication and terminal restoration without a new driver.
#[test]
fn native_shutdown() {
    let fixture = Fixture::new();
    for path in [
        "pty/home",
        "pty/config/rev",
        "pty/data",
        "pty/cache",
        "workspace",
    ] {
        std::fs::create_dir_all(fixture.0.join(path)).unwrap();
    }
    fixture.write(
        "pty/config/rev/config.toml",
        "tree.visible=false\ncompletion.automatic=false",
    );
    fixture.write("workspace/terminal.rs", "fn example() {}\n");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut terminal = terminal::Terminal::start(
        &fixture.0,
        Path::new(env!("CARGO_BIN_EXE_rev")),
        &root.join("scripts/check-completion.sh"),
        "terminal.rs",
    )
    .unwrap();
    let state = fixture.0.join("pty/state/rev/command-history.toml");
    assert!(!state.exists());
    terminal.send(b"\x1bxundo\r").unwrap();
    terminal.finish().unwrap();
    assert!(terminal.contains("REV_TTY_RESTORED"));
    assert_eq!(super::store::disk(&state), ["undo"]);
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("workspace/terminal.rs")).unwrap(),
        "fn example() {}\n"
    );
}
