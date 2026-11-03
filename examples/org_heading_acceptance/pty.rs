use super::support::{self, MARKERS, Result};
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        for (width, height) in [(40, 12), (80, 24)] {
            journey(root, binary, wrapper, (profile, width, height))?;
        }
    }
    Ok(())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, case: (&str, u16, u16)) -> Result<()> {
    let (profile, width, height) = case;
    let original = prepare(root, case)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.org")?;
    let first = terminal.frame(pretty)?;
    assert_eq!(row(&first, 1).width(), usize::from(width));
    let startup = started.elapsed();
    controls(&mut terminal, profile)?;
    copy(&mut terminal, profile)?;
    let resized = if width == 40 { (80, 24) } else { (40, 12) };
    resize(root, &mut terminal, resized)?;
    resized_frame(&mut terminal, resized.0)?;
    restored(root, &mut terminal, &original)?;
    println!(
        "PASS Linux PTY {profile}: {width}x{height} -> {}x{}, startup_us={} output_bytes={}; reference levels 1-8, source/pretty/reset, canonical star copy, source cursor, selector cancellation, idle silence, disk unchanged and TTY restored",
        resized.0,
        resized.1,
        startup.as_micros(),
        terminal.bytes()
    );
    Ok(())
}

fn resized_frame(terminal: &mut Terminal, width: u16) -> Result<()> {
    let frame =
        terminal.frame(|frame| pretty(frame) && row(frame, 1).width() == usize::from(width))?;
    assert!(frame.contains("terminal.org -- RW"));
    Ok(())
}

fn prepare(root: &Path, case: (&str, u16, u16)) -> Result<String> {
    let (profile, width, height) = case;
    let original = support::document();
    std::fs::write(root.join("workspace/terminal.org"), &original)?;
    std::fs::write(
        root.join("pty/config/rev/config.toml"),
        support::config(profile),
    )?;
    std::fs::write(root.join("pty/size"), format!("{width} {height}\n"))?;
    Ok(original)
}

fn restored(root: &Path, terminal: &mut Terminal, original: &str) -> Result<()> {
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/terminal.org"))?,
        original
    );
    Ok(())
}

fn send(terminal: &mut Terminal, input: &[u8], predicate: impl Fn(&str) -> bool) -> Result<String> {
    terminal.send(input)?;
    terminal.frame(predicate)
}

// The wrapper records only its exec'd editor in this unique fixture root.
fn resize(root: &Path, terminal: &mut Terminal, size: (u16, u16)) -> Result<()> {
    let pid = std::fs::read_to_string(root.join("pty/editor.pid"))?;
    let tty = format!("/proc/{}/fd/0", pid.trim());
    terminal.send(b"")?;
    assert!(
        Command::new("stty")
            .args([
                "-F",
                &tty,
                "cols",
                &size.0.to_string(),
                "rows",
                &size.1.to_string()
            ])
            .status()?
            .success()
    );
    assert!(
        Command::new("kill")
            .args(["-WINCH", pid.trim()])
            .status()?
            .success()
    );
    Ok(())
}

fn command(terminal: &mut Terminal, profile: &str, name: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{name}\r").as_bytes())
}

fn controls(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let frame = send(terminal, b"\x1b[B\x1b[B\x1b[C", pretty)?;
    let point = cursor(&frame).ok_or("missing pretty cursor")?;
    assert_eq!(point, (3, 9));
    toggle(terminal, profile, point, false)?;
    toggle(terminal, profile, point, true)?;
    command(terminal, profile, "org-reset-heading-presentation")?;
    terminal.frame(pretty)?;
    cancel(terminal, profile, point)
}

fn toggle(terminal: &mut Terminal, profile: &str, point: (u16, u16), enabled: bool) -> Result<()> {
    command(terminal, profile, "org-pretty-headings-mode")?;
    let predicate = if enabled { pretty } else { source };
    let feedback = if enabled {
        "Org headings: pretty"
    } else {
        "Org headings: source"
    };
    let frame = terminal.frame(|frame| predicate(frame) && frame.contains(feedback))?;
    assert_eq!(cursor(&frame), Some(point));
    Ok(())
}

fn cancel(terminal: &mut Terminal, profile: &str, point: (u16, u16)) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    send(
        terminal,
        format!("{prefix}org-pretty-headings-mode").as_bytes(),
        |frame| frame.contains("M-x "),
    )?;
    let frame = send(terminal, b"\x07", pretty)?;
    assert_eq!(cursor(&frame), Some(point));
    Ok(())
}

fn copy(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let input: &[u8] = if profile == "modal" {
        b"3G0vlly$p"
    } else {
        b"\x1bxgoto-line\r3\r\x01\x00\x06\x06\x06\x1bw\x05\x19"
    };
    send(terminal, input, |frame| {
        row(frame, 3).contains("✸ Heading 3***")
    })?;
    let undo: &[u8] = if profile == "modal" { b"u" } else { b"\x18u" };
    send(terminal, undo, |frame| {
        pretty(frame) && frame.contains("terminal.org -- RW")
    })?;
    Ok(())
}

fn pretty(frame: &str) -> bool {
    MARKERS.iter().enumerate().all(|(line, marker)| {
        row(frame, line + 1).contains(&format!(
            "{}{marker} Heading {}",
            " ".repeat(line),
            line + 1
        ))
    })
}

fn source(frame: &str) -> bool {
    (1..=8).all(|line| row(frame, line).contains(&format!("{} Heading {line}", "*".repeat(line))))
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
