use crate::support::{self, Result};
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::time::Instant;

const DOCUMENT: &str = "** Child\nabcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ\ntail\n";

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, binary, wrapper, profile)?;
    }
    Ok(())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let config = prepare(root, profile)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.org")?;
    terminal.frame(enabled)?;
    let startup = started.elapsed();
    controls(&mut terminal, profile)?;
    pointer_edit(&mut terminal, profile)?;
    resize(&mut terminal)?;
    restored(root, &mut terminal, &config)?;
    println!(
        "PASS Linux PTY {profile}: 40x12/64x12/40x12, startup_us={} output_bytes={}; toggle/enable/disable, wrap, virtual-margin pointer, source edit/undo, idle silence, unchanged disk/configuration and TTY restoration",
        startup.as_micros(),
        terminal.bytes()
    );
    Ok(())
}

fn restored(root: &Path, terminal: &mut Terminal, config: &str) -> Result<()> {
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/terminal.org"))?,
        DOCUMENT
    );
    assert_eq!(
        std::fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    Ok(())
}

fn prepare(root: &Path, profile: &str) -> Result<String> {
    let config = format!(
        "gutter.visible=false\nbookmarks.markers=false\n{}",
        support::config(profile).replace("indent=false", "indent=true")
    );
    std::fs::write(root.join("workspace/terminal.org"), DOCUMENT)?;
    std::fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    std::fs::write(root.join("pty/size"), "40 12\n")?;
    Ok(config)
}

fn command(terminal: &mut Terminal, profile: &str, name: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{name}\r").as_bytes())
}

fn controls(terminal: &mut Terminal, profile: &str) -> Result<()> {
    for name in ["org-indent-disable", "org-indent-mode", "org-indent-enable"] {
        command(terminal, profile, name)?;
        terminal.frame(if name == "org-indent-disable" {
            disabled
        } else {
            enabled
        })?;
    }
    command(terminal, profile, "visual-line-mode")?;
    terminal.frame(|f| row(f, 3).trim_end() == "    tail")?;
    command(terminal, profile, "visual-line-mode")?;
    terminal.frame(enabled)?;
    Ok(())
}

fn pointer_edit(terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(b"\x1b[<0;1;3M\x1b[<0;1;3m")?;
    let frame = terminal.frame(|f| cursor(f) == Some((3, 5)))?;
    assert!(enabled(&frame));
    terminal.send(if profile == "modal" { b"iX\x1b" } else { b"X" })?;
    terminal.frame(|f| row(f, 3).trim_end() == "    XKLMNOPQRSTUVWXYZ")?;
    terminal.send(if profile == "modal" { b"u" } else { b"\x18u" })?;
    terminal.frame(enabled)?;
    Ok(())
}

fn resize(terminal: &mut Terminal) -> Result<()> {
    terminal.resize(12, 64)?;
    terminal.frame(|f| {
        row(f, 2).trim_end() == "    abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            && row(f, 3).trim_end() == "    tail"
    })?;
    terminal.resize(12, 40)?;
    terminal.frame(enabled)?;
    Ok(())
}

fn enabled(frame: &str) -> bool {
    row(frame, 1).trim_end() == "  ○ Child"
        && row(frame, 2).trim_end() == "    abcdefghijklmnopqrstuvwxyzABCDEFGHIJ"
        && row(frame, 3).trim_end() == "    KLMNOPQRSTUVWXYZ"
}

fn disabled(frame: &str) -> bool {
    row(frame, 1).trim_end() == "○ Child"
        && row(frame, 2).trim_end() == "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN"
        && row(frame, 3).trim_end() == "OPQRSTUVWXYZ"
}

fn cursor(frame: &str) -> Option<(u16, u16)> {
    frame
        .split("\x1b[")
        .skip(1)
        .filter_map(|part| {
            let end = part.find(|ch: char| ('@'..='~').contains(&ch))?;
            if part.as_bytes()[end] != b'H' {
                return None;
            }
            let (row, column) = part[..end].split_once(';')?;
            Some((row.parse().ok()?, column.parse().ok()?))
        })
        .last()
}
