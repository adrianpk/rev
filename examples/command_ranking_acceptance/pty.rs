use crate::support::Result;
use crate::terminal::{Terminal, row};
use rev::command::usage::Store;
use std::path::Path;
use std::time::{Duration, Instant};

const DOCUMENT: &str = "retained source\nsecond line\n";

fn setup(root: &Path, profile: &str, names: &[&str]) -> Result<String> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        std::fs::create_dir_all(root.join(directory))?;
    }
    let config = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"
    );
    std::fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    for name in names {
        std::fs::write(root.join("workspace").join(name), DOCUMENT)?;
    }
    Ok(config)
}
fn prompt(frame: &str, expected: &str) -> bool {
    (1..=24).any(|number| row(frame, number).starts_with(expected))
}
fn first_candidate(frame: &str, expected: &str) -> bool {
    (1..24)
        .find(|number| row(frame, *number).starts_with("M-x "))
        .is_some_and(|number| row(frame, number + 1).trim() == expected)
}
fn state(root: &Path) -> std::path::PathBuf {
    root.join("pty/state/rev/command-history.toml")
}
fn unchanged(root: &Path, config: &str, names: &[&str]) -> Result<()> {
    for name in names {
        assert_eq!(
            std::fs::read_to_string(root.join("workspace").join(name))?,
            DOCUMENT
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    // Only the complete record and stable lock may survive publication.
    let mut entries = std::fs::read_dir(state(root).parent().ok_or("missing state parent")?)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    assert_eq!(
        entries,
        ["command-history.toml", "command-history.toml.lock"]
    );
    Ok(())
}

// Two serial starts form one input/render/restart journey per profile. The
// shared helper caps each owned editor at eight seconds and 1 MiB; the script
// additionally caps the entire journey at 30 seconds, excluding the one build.
pub(super) fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    let config = setup(root, profile, &["first.txt", "restart.txt"])?;
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    let started = Instant::now();
    let mut first = Terminal::start(root, binary, wrapper, "first.txt")?;
    assert!(!state(root).exists(), "startup must not create history");
    first.send(format!("{prefix}goto\t").as_bytes())?;
    first.frame(|frame| prompt(frame, "M-x goto-line "))?;
    assert!(
        !state(root).exists(),
        "Tab must not publish a named invocation"
    );
    first.send(b"\r")?;
    first.frame(|frame| row(frame, 24).starts_with("Goto line:"))?;
    first.send(b"\x07")?;
    first.frame(|frame| !frame.contains("Goto line:"))?;
    first.idle()?;
    first.finish()?;
    assert_eq!(Store::open(&state(root))?.1, ["goto-line"]);
    let first_bytes = first.bytes();
    drop(first);

    let mut restarted = Terminal::start(root, binary, wrapper, "restart.txt")?;
    restarted.send(prefix.as_bytes())?;
    restarted.frame(|frame| first_candidate(frame, "goto-line"))?;
    // Recall is still session-local after restart, even though ranking persists.
    restarted.send(b"\x1bp")?;
    restarted.frame(|frame| {
        (1..=24).any(|number| row(frame, number).contains("Command history is empty"))
    })?;
    restarted.send(b"\x07")?;
    restarted.frame(|frame| !frame.contains("M-x "))?;
    restarted.send(prefix.as_bytes())?;
    restarted.frame(|frame| first_candidate(frame, "goto-line"))?;
    restarted.resize(12, 40)?;
    restarted.frame(|frame| first_candidate(frame, "goto-line"))?;
    restarted.resize(24, 80)?;
    restarted.frame(|frame| first_candidate(frame, "goto-line"))?;
    restarted.send(b"\t\r")?;
    restarted.frame(|frame| row(frame, 24).starts_with("Goto line:"))?;
    restarted.send(b"\x07")?;
    restarted.frame(|frame| !frame.contains("Goto line:"))?;
    restarted.finish()?;
    assert_eq!(Store::open(&state(root))?.1, ["goto-line", "goto-line"]);
    unchanged(root, &config, &["first.txt", "restart.txt"])?;
    assert!(started.elapsed() < Duration::from_secs(30));
    println!(
        "PASS Linux PTY {profile}: Tab without invocation, later argument cancellation, cross-file restart ranking, separate recall, resize, exactly-once dispatch, unchanged files/configuration, restored TTYs; elapsed_ms={} output_bytes={first_bytes}+{}",
        started.elapsed().as_millis(),
        restarted.bytes()
    );
    Ok(())
}

// Both real executables start before either records usage, so the second owns a
// stale startup snapshot. Its later batch must rebase onto the first's complete
// published record. At most two fixture editors are live, with no provider/service.
pub(super) fn paired(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let config = setup(root, "non-modal", &["left.txt", "right.txt"])?;
    let started = Instant::now();
    let mut left = Terminal::start(root, binary, wrapper, "left.txt")?;
    let mut right = Terminal::start(root, binary, wrapper, "right.txt")?;
    assert!(!state(root).exists());
    left.send(b"\x1bxundo\r")?;
    left.finish()?;
    assert_eq!(Store::open(&state(root))?.1, ["undo"]);
    right.send(b"\x1bxgoto-line\r")?;
    right.frame(|frame| row(frame, 24).starts_with("Goto line:"))?;
    right.send(b"\x07")?;
    right.frame(|frame| !frame.contains("Goto line:"))?;
    right.finish()?;
    assert_eq!(Store::open(&state(root))?.1, ["undo", "goto-line"]);
    unchanged(root, &config, &["left.txt", "right.txt"])?;
    assert!(started.elapsed() < Duration::from_secs(30));
    println!(
        "PASS paired Linux publishers: stale-start rebase, both events once, complete state, no temporary residue, unchanged source/configuration and restored TTYs; elapsed_ms={} output_bytes={}+{}",
        started.elapsed().as_millis(),
        left.bytes(),
        right.bytes()
    );
    Ok(())
}
