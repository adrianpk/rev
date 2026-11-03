use super::{
    support::*,
    terminal::{Terminal, row},
};
use std::{fs, path::Path, process::Command};

use unicode_width::UnicodeWidthStr;

const ORIGINAL: &str = "z界\na line\n";

pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, binary, wrapper, profile)?;
    }
    Ok(())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    prepare(root, profile)?;
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.txt")?;
    filter(&mut terminal, profile)?;
    insert(&mut terminal, profile)?;
    cancel(root, &mut terminal, profile)?;
    resize(root, &mut terminal)?;
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/terminal.txt"))?,
        ORIGINAL
    );
    println!(
        "PASS shell PTY profile={profile}: canonical Unicode selection, sort/tr, one-step undo, inner cancellation, descendant cleanup, resize, post-job idle and terminal restoration; bounded_bytes={}",
        terminal.bytes()
    );
    Ok(())
}

fn prepare(root: &Path, profile: &str) -> Result<()> {
    for name in ["leader", "descendant"] {
        let path = root.join("workspace").join(name);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    fs::write(root.join("workspace/terminal.txt"), ORIGINAL)?;
    fs::write(
        root.join("pty/config/rev/config.toml"),
        format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\nshell.program='/bin/sh'\nshell.timeout_seconds=3\n"
        ),
    )?;
    Ok(())
}

fn send(terminal: &mut Terminal, input: &[u8], predicate: impl Fn(&str) -> bool) -> Result<String> {
    terminal.send(input)?;
    terminal.frame(predicate)
}

fn named(terminal: &mut Terminal, profile: &str, name: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{name}\r").as_bytes())
}

fn filter(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let selection: &[u8] = if profile == "modal" {
        b"ggVG"
    } else {
        b"\x1b<\x00\x1b>"
    };
    send(terminal, selection, |frame| frame.contains("terminal.txt"))?;
    if profile == "modal" {
        named(terminal, profile, "shell-filter-region")?;
    } else {
        terminal.send(b"\x15\x1b|")?;
    }
    terminal.frame(|frame| frame.contains("Shell command:"))?;
    send(terminal, b"sort\r", |frame| {
        row(frame, 1).contains("a line") && frame.contains("one undo step")
    })?;
    undo(terminal, profile)?;
    Ok(())
}

fn insert(terminal: &mut Terminal, profile: &str) -> Result<()> {
    if profile == "modal" {
        named(terminal, profile, "shell-command-insert")?;
    } else {
        terminal.send(b"\x15\x1b!")?;
    }
    terminal.frame(|frame| frame.contains("Shell command:"))?;
    send(terminal, "printf 'spark✨\\n'\r".as_bytes(), |frame| {
        frame.contains("spark✨") && frame.contains("one undo step")
    })?;
    undo(terminal, profile)?;
    display(terminal, profile)
}

fn display(terminal: &mut Terminal, profile: &str) -> Result<()> {
    named(terminal, profile, "shell-command")?;
    terminal.frame(|frame| frame.contains("Shell command:"))?;
    send(terminal, b"printf 'hello\\n' | tr a-z A-Z\r", |frame| {
        frame.contains("shell-command-output opens")
    })?;
    named(terminal, profile, "shell-command-output")?;
    terminal.frame(|frame| row(frame, 1).contains("HELLO") && frame.contains("RO"))?;
    named(terminal, profile, "switch-to-buffer")?;
    terminal.frame(|frame| frame.contains("Switch to buffer:"))?;
    send(terminal, b"terminal.txt\r", original)?;
    Ok(())
}

fn undo(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let input: &[u8] = if profile == "modal" { b"u" } else { b"\x18u" };
    send(terminal, input, original)?;
    Ok(())
}

fn original(frame: &str) -> bool {
    row(frame, 1).contains("z界")
        && row(frame, 2).contains("a line")
        && frame.contains("terminal.txt -- RW")
}

fn cancel(root: &Path, terminal: &mut Terminal, profile: &str) -> Result<()> {
    let owned = waiting(root, terminal, profile)?;
    named(terminal, profile, "goto-line")?;
    terminal.frame(|frame| frame.contains("Goto line:"))?;
    send(terminal, b"\x07", |frame| {
        !frame.contains("Goto line:") && frame.contains("<Shell running>")
    })?;
    send(terminal, b"\x07", |frame| {
        !frame.contains("<Shell") && frame.contains("cancellation requested")
    })?;
    reaped(&owned.leader)?;
    reaped(&owned.descendant)?;
    assert!(terminal.contains("terminal.txt -- RW"));
    Ok(())
}

struct Owned {
    leader: String,
    descendant: String,
}

fn waiting(root: &Path, terminal: &mut Terminal, profile: &str) -> Result<Owned> {
    named(terminal, profile, "shell-command")?;
    terminal.frame(|frame| frame.contains("Shell command:"))?;
    send(
        terminal,
        b"printf '%s' $$ > leader; sleep 3 & printf '%s' $! > descendant; wait\r",
        |frame| frame.contains("<Shell running>"),
    )?;
    let leader = ready(&root.join("workspace/leader"))?;
    let descendant = ready(&root.join("workspace/descendant"))?;

    Ok(Owned { leader, descendant })
}

fn resize(root: &Path, terminal: &mut Terminal) -> Result<()> {
    let pid = ready(&root.join("pty/editor.pid"))?;
    let tty = format!("/proc/{}/fd/0", pid.trim().parse::<u32>()?);
    terminal.send(b"")?;
    assert!(
        Command::new("stty")
            .args(["-F", &tty, "cols", "72", "rows", "18"])
            .status()?
            .success()
    );
    assert!(
        Command::new("kill")
            .args(["-WINCH", pid.trim()])
            .status()?
            .success()
    );
    let frame = terminal.frame(|frame| row(frame, 1).width() == 72)?;
    assert!(original(&frame));
    Ok(())
}
