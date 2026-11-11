use crate::support::{self, DOCUMENT, Result};
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::time::Instant;

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(&root.join(profile), binary, wrapper, profile)?;
    }
    Ok(())
}

fn command(terminal: &mut Terminal, profile: &str, name: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{name}\r").as_bytes())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let config = prepare(root, profile)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.txt")?;
    marker_visibility(&mut terminal, profile)?;
    resize(&mut terminal)?;
    wrapping(&mut terminal, profile)?;
    copied(&mut terminal, profile)?;
    finish(&mut terminal, root, &config)?;
    println!(
        "PASS Linux PTY {profile}: set/show/hide, 40x12/64x12 resize, wrapping, pointer source-only copy/yank/undo, unchanged disk/configuration and restored TTY; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn marked(frame: &str) -> bool {
    row(frame, 1).starts_with("■   1 │alpha") && frame.matches('■').count() == 1
}

fn copied(terminal: &mut Terminal, profile: &str) -> Result<()> {
    copy_region(terminal)?;
    restore_point(terminal, profile)?;
    command(terminal, profile, "yank")?;
    terminal.frame(|frame| row(frame, 1).starts_with("■   1 │alphaalpha beta"))?;
    command(terminal, profile, "undo")?;
    terminal.frame(|frame| marked(frame) && !frame.contains("alphaalpha"))?;
    Ok(())
}

fn prepare(root: &Path, profile: &str) -> Result<String> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        std::fs::create_dir_all(root.join(directory))?;
    }
    let config = support::config(profile);
    std::fs::write(root.join("workspace/terminal.txt"), DOCUMENT)?;
    std::fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    std::fs::write(root.join("pty/size"), "40 12\n")?;
    Ok(config)
}
fn marker_visibility(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}bookmark-set\racceptance\r").as_bytes())?;
    terminal.frame(marked)?;
    command(terminal, profile, "bookmark-markers-mode")?;
    terminal.frame(|frame| !frame.contains('■') && row(frame, 1).starts_with("   1 │alpha"))?;
    command(terminal, profile, "bookmark-markers-mode")?;
    terminal.frame(marked)?;
    Ok(())
}
fn resize(terminal: &mut Terminal) -> Result<()> {
    terminal.resize(12, 64)?;
    terminal.frame(|frame| marked(frame) && row(frame, 2).starts_with("    2 │last"))?;
    terminal.resize(12, 40)?;
    terminal.frame(|frame| marked(frame) && row(frame, 3).starts_with("    2 │last"))?;
    Ok(())
}
fn wrapping(terminal: &mut Terminal, profile: &str) -> Result<()> {
    command(terminal, profile, "visual-line-mode")?;
    terminal.frame(|frame| marked(frame) && row(frame, 2).starts_with("    2 │last"))?;
    command(terminal, profile, "visual-line-mode")?;
    terminal.frame(|frame| marked(frame) && row(frame, 3).starts_with("    2 │last"))?;
    Ok(())
}
fn finish(terminal: &mut Terminal, root: &Path, config: &str) -> Result<()> {
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/terminal.txt"))?,
        DOCUMENT
    );
    assert_eq!(
        std::fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    Ok(())
}
fn copy_region(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1b[<0;8;1M\x1b[<32;13;1M\x1b[<0;13;1m")?;
    terminal.frame(|frame| frame.contains("\x1b[1;13H"))?;
    terminal.send(b"\x1bxkill-ring-save\r")?;
    terminal.frame(|frame| frame.contains("Region copied"))?;
    Ok(())
}
fn restore_point(terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(if profile == "modal" { b"\x1b" } else { b"\x07" })?;
    terminal.frame(marked)?;
    terminal.send(b"\x1b[<0;8;1M\x1b[<0;8;1m")?;
    terminal.frame(|frame| frame.contains("\x1b[1;8H"))?;
    Ok(())
}
