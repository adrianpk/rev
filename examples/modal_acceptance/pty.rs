use crate::support::Result;
use crate::terminal::{Terminal, row};
use std::path::Path;

const ORIGINAL: &str = "one two\nthree four\nfn cfg_value() {}\nfn main() { cfg }\n";

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let mut terminal = start(root, binary, wrapper)?;
    operators(&mut terminal)?;
    repeat(&mut terminal)?;
    jumps(&mut terminal)?;
    commands(&mut terminal)?;
    completion(&mut terminal)?;
    burst(&mut terminal)?;
    restored(root, &mut terminal)?;
    println!(
        "PASS Vim Linux PTY: decoded operators/Visual/jumps/repeat, colon/M-x arguments, Insert completion, Replace, 128-event burst, cursor shapes/reset, disk unchanged and TTY restored"
    );
    Ok(())
}

fn start(root: &Path, binary: &Path, wrapper: &Path) -> Result<Terminal> {
    std::fs::write(
        root.join("pty/config/rev/config.toml"),
        "[editor]\ninteraction_profile = 'vim'\n[tree]\nvisible = false\n[completion]\nautomatic = false\n",
    )?;
    std::fs::write(root.join("workspace/terminal.rs"), ORIGINAL)?;
    let mut terminal = Terminal::start(root, binary, wrapper)?;
    terminal.frame(|frame| frame.contains("NORMAL"))?;
    assert!(terminal.contains("\x1b[2 q"), "Normal block cursor");
    Ok(terminal)
}

fn restored(root: &Path, terminal: &mut Terminal) -> Result<()> {
    terminal.idle()?;
    terminal.finish()?;
    assert!(
        terminal.contains("\x1b[0 q"),
        "modal cursor shape reset on exit"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("workspace/terminal.rs"))?,
        ORIGINAL
    );
    Ok(())
}

fn send_frame(
    terminal: &mut Terminal,
    input: &[u8],
    predicate: impl Fn(&str) -> bool,
) -> Result<String> {
    terminal.send(input)?;
    terminal
        .frame(predicate)
        .map_err(|error| format!("PTY input {input:?}: {error}").into())
}

fn operators(terminal: &mut Terminal) -> Result<()> {
    send_frame(terminal, b"dw", |frame| {
        let first = row(frame, 1);
        first.contains("two") && !first.contains("one")
    })?;
    send_frame(terminal, b"uvld", |frame| {
        text_state(frame, 1, "e two", "NORMAL")
    })?;
    Ok(())
}

fn repeat(terminal: &mut Terminal) -> Result<()> {
    send_frame(terminal, b"u0i", |frame| frame.contains("INSERT"))?;
    assert!(terminal.contains("\x1b[6 q"), "Insert bar cursor");
    send_frame(terminal, b"X\x1b", |frame| {
        text_state(frame, 1, "Xone two", "NORMAL")
    })?;
    send_frame(terminal, b"j0.", |frame| {
        row(frame, 2).contains("Xthree four")
    })?;
    send_frame(terminal, b"uu2G", |frame| clean_at(frame, 2))?;
    Ok(())
}

fn jumps(terminal: &mut Terminal) -> Result<()> {
    send_frame(terminal, b"\x0f", |frame| normal_at(frame, 1))?;
    send_frame(terminal, b"\t", |frame| normal_at(frame, 2))?;
    Ok(())
}

fn commands(terminal: &mut Terminal) -> Result<()> {
    send_frame(terminal, b":goto-li\t\r1\r", |frame| normal_at(frame, 1))?;
    send_frame(terminal, b"\x1bxgoto-line\r2\r", |frame| {
        normal_at(frame, 2)
    })?;
    send_frame(terminal, b"v:", |frame| frame.contains("M-x "))?;
    send_frame(terminal, b"\x07", |frame| {
        frame.contains("VISUAL") && !frame.contains("M-x ")
    })?;
    send_frame(terminal, b"\x1b", |frame| frame.contains("NORMAL"))?;
    Ok(())
}

fn completion(terminal: &mut Terminal) -> Result<()> {
    send_frame(
        terminal,
        b":goto-line\r4\rfc3li\x1bxcompletion-at-point\r",
        popup,
    )?;
    send_frame(terminal, b"\t", |frame| completed(frame, "cfg_value }"))?;
    // A separate decoded Escape precedes undo; packed Escape-u is Meta-u.
    send_frame(terminal, b"\x1b", |frame| {
        text_state(frame, 4, "cfg_value }", "NORMAL")
    })?;
    send_frame(terminal, b"u", |frame| clean_source(frame, "{ cfg }"))?;
    send_frame(terminal, b"R", |frame| frame.contains("REPLACE"))?;
    assert!(terminal.contains("\x1b[4 q"), "Replace underline cursor");
    send_frame(terminal, b"Z\x1b", |frame| {
        text_state(frame, 4, "Z", "NORMAL")
    })?;
    send_frame(terminal, b"u", |frame| clean_source(frame, "{ cfg }"))?;
    Ok(())
}

fn burst(terminal: &mut Terminal) -> Result<()> {
    let before = send_frame(terminal, b"gg0", |frame| normal_at(frame, 1))?;
    let point = cursor(&before).ok_or("missing source cursor")?;
    let mut input = b"lh".repeat(64);
    input.extend(b":goto-line\r2\r");
    send_frame(terminal, &input, |frame| normal_at(frame, 2))?;
    send_frame(terminal, b"\x0f", |frame| cursor(frame) == Some(point))?;
    assert!(terminal.contains("terminal.rs -- RW"));
    Ok(())
}

fn text_state(frame: &str, line: usize, text: &str, state: &str) -> bool {
    row(frame, line).contains(text) && frame.contains(state)
}
fn normal_at(frame: &str, line: u16) -> bool {
    frame.contains("NORMAL") && cursor(frame).is_some_and(|p| p.0 == line)
}
fn clean_at(frame: &str, line: u16) -> bool {
    normal_at(frame, line) && frame.contains("terminal.rs -- RW")
}
fn clean_source(frame: &str, text: &str) -> bool {
    text_state(frame, 4, text, "NORMAL") && frame.contains("terminal.rs -- RW")
}
fn popup(frame: &str) -> bool {
    frame.contains("INSERT") && frame.contains('┌') && frame.contains("function")
}
fn completed(frame: &str, text: &str) -> bool {
    !frame.contains('┌') && text_state(frame, 4, text, "INSERT")
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
