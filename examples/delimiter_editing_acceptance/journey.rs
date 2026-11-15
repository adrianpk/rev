use crate::{
    support::Result,
    terminal::{Terminal, row},
};
use std::{fs, path::Path, time::Instant};

const RECORDED: &str = "(q)[{";
const REPLAYED: &str = "(q)[{(q)[{";

// Each profile owns one controlling PTY. Literal bracketed paste and recorded
// physical keys cross the real input parser; disk bytes are independent oracles.
pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    let config = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\ngutter.visible=false\nsyntax.enabled=false\n"
    );
    prepare(root, &config)?;
    let started = Instant::now();
    let mut t = Terminal::start(root, binary, wrapper, "notes.txt")?;
    workflow(&mut t, root, profile)?;
    t.idle()?;
    t.finish()?;
    verify(root, &config)?;
    println!(
        "MEASURE delimiter PTY profile={profile} elapsed_ms={} output_bytes={} restored=true reaped=true; pairing/skip/literal-paste/macro/undo/redo/save/reopen/focus pass",
        started.elapsed().as_millis(),
        t.bytes()
    );
    Ok(())
}

fn workflow(t: &mut Terminal, root: &Path, profile: &str) -> Result<()> {
    if profile == "modal" {
        step(t, b"i", "INSERT")?;
    }
    record(t)?;
    step(t, b"\x1bOS", REPLAYED)?;
    step(t, b"\x1f", RECORDED)?;
    save(t, root, RECORDED)?;
    step(t, b"\x1b\x1f", REPLAYED)?;
    save(t, root, REPLAYED)?;
    focus(t)?;
    reopen(t, root)
}

fn verify(root: &Path, config: &str) -> Result<()> {
    assert_eq!(
        fs::read_to_string(root.join("workspace/notes.txt"))?,
        REPLAYED
    );
    assert_eq!(
        fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    Ok(())
}

fn prepare(root: &Path, config: &str) -> Result<()> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        fs::create_dir_all(root.join(directory))?;
    }
    fs::write(root.join("pty/config/rev/config.toml"), config)?;
    fs::write(root.join("workspace/notes.txt"), "")?;
    Ok(())
}

fn step(t: &mut Terminal, bytes: &[u8], expected: &str) -> Result<()> {
    t.send(bytes)?;
    t.frame(|frame| frame.contains(expected))?;
    Ok(())
}

fn record(t: &mut Terminal) -> Result<()> {
    step(t, b"\x1bOR", "Def")?;
    step(t, b"(", "()")?;
    step(t, b"q)", "(q)")?;
    step(t, b"\x1b[200~[{\x1b[201~", RECORDED)?;
    step(t, b"\x1bOS", "recorded")
}

fn save(t: &mut Terminal, root: &Path, expected: &str) -> Result<()> {
    t.send(b"\x18\x13")?;
    t.frame(|frame| {
        frame.contains("-- RW")
            && fs::read_to_string(root.join("workspace/notes.txt")).is_ok_and(|s| s == expected)
    })?;
    Ok(())
}

fn focus(t: &mut Terminal) -> Result<()> {
    step(t, b"\x18\x06(", "Find file:")?;
    t.send(b"\x07")?;
    t.frame(|frame| row(frame, 1).trim() == REPLAYED)?;
    Ok(())
}

fn reopen(t: &mut Terminal, root: &Path) -> Result<()> {
    step(t, b"\x18k", "Kill buffer:")?;
    step(t, b"\r", "*scratch*")?;
    t.send(b"\x18\x06notes.txt\r")?;
    t.frame(|frame| frame.contains("notes.txt -- RW") && row(frame, 1).trim() == REPLAYED)?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/notes.txt"))?,
        REPLAYED
    );
    Ok(())
}
