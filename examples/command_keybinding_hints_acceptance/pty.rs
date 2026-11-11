use crate::support::Result;
use crate::terminal::{Terminal, row};
use rev::command::usage::Store;
use std::path::Path;
use std::time::{Duration, Instant};

const DOCUMENT: &str = "retained source\nsecond line\n";

fn candidate(frame: &str, name: &str, hint: Option<&str>) -> bool {
    let hint = hint.map(|hint| format!("[{hint}]"));
    let expected: Vec<_> = std::iter::once(name)
        .chain(hint.as_deref().into_iter().flat_map(str::split_whitespace))
        .collect();
    // The editable query can scroll its label at narrow widths. Match the
    // complete candidate fields without using that label as row geometry.
    (1..=24).any(|number| {
        row(frame, number)
            .split_whitespace()
            .eq(expected.iter().copied())
    })
}
fn prompt(frame: &str, expected: &str) -> bool {
    (1..=24).any(|number| row(frame, number).starts_with(expected))
}

// One real editor per profile, under the shared eight-second/1-MiB owner and
// the script's 30-second outer deadline. Build time is outside both bounds.
pub(super) fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    let config = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\nminibuffer.candidate_limit=3\n[keys.editor]\n'M-j'='goto-line'\n[keys.vim_normal]\n'M-j'='goto-line'"
    );
    let config_path = root.join("pty/config/rev/config.toml");
    prepare(root, &config_path, &config)?;
    let history = root.join("pty/state/rev/command-history.toml");
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "notes.txt")?;
    hinted_selection(&mut terminal, prefix, &history)?;
    named_argument(&mut terminal)?;
    bound_argument(&mut terminal)?;
    hint_query(&mut terminal, prefix)?;
    unbound_query(&mut terminal, prefix)?;
    finish_journey(&mut terminal, root, &config_path, &config, &history)?;
    assert!(started.elapsed() < Duration::from_secs(30));
    println!(
        "PASS Linux PTY {profile}: configured effective hint, NO_COLOR selection, complete narrow omission/resize restoration, name-only Tab and filtering, unbound name, named/bound argument dispatch, exactly-once usage, unchanged source/configuration and restored TTY; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn prepare(root: &Path, config_path: &Path, config: &str) -> Result<()> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        std::fs::create_dir_all(root.join(directory))?;
    }
    std::fs::write(config_path, config)?;
    std::fs::write(root.join("workspace/notes.txt"), DOCUMENT)?;
    Ok(())
}
fn hinted_selection(terminal: &mut Terminal, prefix: &str, history: &Path) -> Result<()> {
    terminal.send(format!("{prefix}goto\t").as_bytes())?;
    terminal.frame(|frame| {
        prompt(frame, "M-x goto-line ") && candidate(frame, "goto-line", Some("M-j"))
    })?;
    assert!(
        terminal.contains("\x1b[7m"),
        "NO_COLOR selected row must use reverse video"
    );
    assert!(
        !history.exists(),
        "hint rendering and Tab must not add usage"
    );
    terminal.resize(12, 12)?;
    terminal.frame(|frame| candidate(frame, "goto-line", None))?;
    terminal.resize(24, 80)?;
    terminal.frame(|frame| candidate(frame, "goto-line", Some("M-j")))?;
    Ok(())
}
fn named_argument(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\r")?;
    terminal.frame(|frame| prompt(frame, "Goto line:"))?;
    terminal.send(b"2\r")?;
    terminal.frame(|frame| frame.contains("L2 C1") && !frame.contains("Goto line:"))?;
    Ok(())
}
fn bound_argument(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bj")?;
    terminal.frame(|frame| prompt(frame, "Goto line:"))?;
    terminal.send(b"1\r")?;
    terminal.frame(|frame| frame.contains("L1 C1") && !frame.contains("Goto line:"))?;
    Ok(())
}
fn hint_query(terminal: &mut Terminal, prefix: &str) -> Result<()> {
    terminal.send(format!("{prefix}M-j").as_bytes())?;
    terminal.frame(|frame| prompt(frame, "M-x M-j") && prompt(frame, "No matching commands"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|frame| !frame.contains("M-x "))?;
    Ok(())
}
fn unbound_query(terminal: &mut Terminal, prefix: &str) -> Result<()> {
    terminal.send(format!("{prefix}mouse-mode\t").as_bytes())?;
    terminal
        .frame(|frame| prompt(frame, "M-x mouse-mode ") && candidate(frame, "mouse-mode", None))?;
    terminal.send(b"\x07")?;
    terminal.frame(|frame| !frame.contains("M-x "))?;
    Ok(())
}
fn finish_journey(
    terminal: &mut Terminal,
    root: &Path,
    config_path: &Path,
    config: &str,
    history: &Path,
) -> Result<()> {
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(Store::open(history)?.1, ["goto-line"]);
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/notes.txt"))?,
        DOCUMENT
    );
    assert_eq!(std::fs::read_to_string(config_path)?, config);
    Ok(())
}
