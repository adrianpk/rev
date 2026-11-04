use super::{
    support::*,
    terminal::{Terminal, row},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, binary, wrapper, profile)?;
    }
    super::failure::run(root, binary, wrapper)
}
fn prepare(root: &Path, profile: &str) -> Result<PathBuf> {
    let owner = root.join(format!("host-{profile}"));
    let directory = repository(&owner, "")?;
    let config = settings(&directory, profile)?;
    fs::create_dir_all(owner.join("pty/config/rev"))?;
    fs::copy(config, owner.join("pty/config/rev/config.toml"))?;
    fs::create_dir_all(owner.join("pty/home"))?;
    Ok(owner)
}
fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let owner = prepare(root, profile)?;
    let mut terminal = Terminal::start(&owner, binary, wrapper, "source.org")?;
    initial(&owner, &mut terminal, profile).map_err(|e| format!("initial: {e}"))?;
    lazygit(&owner, &mut terminal).map_err(|e| format!("lazygit: {e}"))?;
    recorder(&owner, &mut terminal, profile).map_err(|e| format!("recorder: {e}"))?;
    closure(&owner, &mut terminal).map_err(|e| format!("closure: {e}"))?;
    terminal.finish()?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/source.org"))?,
        ORIGINAL
    );
    println!(
        "PASS nested host PTY profile={profile}: real shell/lazygit, multiple sessions, relocation, copy/paste, literal-next, mouse, protected source, declined shutdown, actual exit and full restoration; output_bytes={}",
        terminal.bytes()
    );
    Ok(())
}
fn named(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(format!("\x1bx\x1b[200~{name}\x1b[201~\r").as_bytes())
}
fn line(terminal: &mut Terminal, line: &str) -> Result<()> {
    terminal.send(format!("\x1b[200~{line}\x1b[201~\r").as_bytes())
}
fn initial(owner: &Path, terminal: &mut Terminal, profile: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}terminal-open\r").as_bytes())?;
    terminal.frame(|f| plain(f).contains("SHELL_READY>"))?;
    line(
        terminal,
        "printf %s $$ >shell.pid; export KEPT=alive; printf '\\033[2J\\033[HPTY_SHELL_READY\\n'",
    )?;
    terminal.frame(|f| row(f, 24).contains("PTY_SHELL_READY"))?;
    assert!(pid_file(&owner.join("workspace/shell.pid"))? > 0);
    Ok(())
}
fn lazygit(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    line(
        terminal,
        "sh -c 'printf %s $$ >tool.pid; exec lazygit --use-config-file lazygit.yml'",
    )?;
    terminal.frame(|f| plain(f).contains("Files"))?;
    let tool = pid_file(&owner.join("workspace/tool.pid"))?;
    terminal.send(b"?")?;
    terminal.frame(|f| plain(f).contains("Keybindings"))?;
    move_tool(terminal, "terminal-move-to-buffer")?;
    move_tool(terminal, "terminal-move-to-panel")?;
    assert_eq!(pid_file(&owner.join("workspace/tool.pid"))?, tool);
    assert_eq!(
        fs::read_to_string(format!("/proc/{tool}/comm"))?.trim(),
        "lazygit"
    );
    quit_tool(terminal)
}
fn move_tool(terminal: &mut Terminal, name: &str) -> Result<()> {
    named(terminal, name)?;
    terminal.frame(|f| plain(f).contains("Keybindings"))?;
    Ok(())
}
fn quit_tool(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1b")?;
    terminal.frame(|f| plain(f).contains("Files") && !plain(f).contains("Keybindings"))?;
    terminal.send(b"q")?;
    terminal.frame(|f| plain(f).contains("SHELL_READY>"))?;
    Ok(())
}
fn recorder(owner: &Path, terminal: &mut Terminal, profile: &str) -> Result<()> {
    start_recorder(terminal)?;
    terminal.send(b"\x11\x07\x1b[<0;5;3M\x1b[<0;5;3m\x1b[<68;5;3M")?;
    copy(terminal, profile)?;
    terminal.frame(|f| plain(f).contains("INPUT_ACK") && plain(f).contains("SHELL_READY>"))?;
    assert_eq!(
        fs::read(owner.join("workspace/input.bin"))?,
        b"\x07\x1b[<0;5;3M\x1b[<0;5;3m\x1b[200~paste\x1b[201~"
    );
    terminal.idle()?;
    Ok(())
}
fn copy(terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(b"\x03\x14")?;
    terminal.frame(|f| plain(f).contains("[COPY]"))?;
    let keys: &[u8] = if profile == "modal" {
        b"gg0vey"
    } else {
        b"\x1b<\x00\x1bf\x1bw"
    };
    terminal.send(keys)?;
    terminal.frame(|f| plain(f).contains("[COPY]") && plain(f).contains("copied"))?;
    terminal.send(b"\r\x19")?;
    Ok(())
}
fn closure(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    decline_exit(terminal)?;
    close_first(terminal)?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/kept-state"))?,
        "alive"
    );
    switch(terminal, "terminal-2")?;
    line(terminal, "exit 7")?;
    terminal.frame(|f| plain(f).contains("Exited (7)"))?;
    reaped(pid_file(&owner.join("workspace/shell.pid"))?)?;
    Ok(())
}
fn decline_exit(terminal: &mut Terminal) -> Result<()> {
    named(terminal, "save-buffer")?;
    terminal.frame(|f| plain(f).contains("requires a source buffer"))?;
    terminal.send(b"\x18\x03")?;
    terminal.frame(|f| plain(f).contains("terminate 2 terminal sessions"))?;
    terminal.send(b"n")?;
    terminal.frame(|f| plain(f).contains("sessions retained"))?;
    Ok(())
}
fn switch(terminal: &mut Terminal, name: &str) -> Result<()> {
    named(terminal, "switch-to-buffer")?;
    terminal.frame(|f| plain(f).contains("Switch to buffer:"))?;
    terminal.send(format!("{name}\r").as_bytes())?;
    terminal.frame(|f| plain(f).contains(name) && plain(f).contains("SHELL_READY>"))?;
    Ok(())
}
fn close_first(terminal: &mut Terminal) -> Result<()> {
    switch(terminal, "terminal")?;
    line(terminal, "printf '%s' \"$KEPT\" >kept-state")?;
    terminal.frame(|f| plain(f).contains("SHELL_READY>"))?;
    named(terminal, "kill-buffer")?;
    terminal.frame(|f| plain(f).contains("Kill buffer:"))?;
    terminal.send(b"\r")?;
    terminal.frame(|f| plain(f).contains("Terminate"))?;
    terminal.send(b"y")?;
    terminal.frame(|f| plain(f).contains("Buffer closed"))?;
    Ok(())
}

fn start_recorder(terminal: &mut Terminal) -> Result<()> {
    named(terminal, "terminal-new-buffer")?;
    terminal.frame(|f| plain(f).contains("terminal-2") && plain(f).contains("SHELL_READY>"))?;
    let payload = b"\x07\x1b[<0;5;3M\x1b[<0;5;3m\x1b[200~paste\x1b[201~";
    line(
        terminal,
        &format!(
            "printf %s $$ >peer.pid; stty raw -echo; printf '\\033[?1002h\\033[?1006h\\033[?2004h\\033[2J\\033[Hpaste beta\\r\\nINPUT_READY'; dd bs=1 count={} of=input.bin 2>/dev/null; printf '\\033[?1002l\\033[?1006l\\033[?2004l\\r\\nINPUT_ACK'; stty sane",
            payload.len()
        ),
    )?;
    terminal.frame(|f| row(f, 1).contains("paste beta") && row(f, 2).contains("INPUT_READY"))?;
    Ok(())
}

// Native terminal cells carry explicit column/style controls between glyphs.
// Inspect their rendered text while row-specific checks keep native row maps.
fn plain(frame: &str) -> String {
    frame
        .split("\x1b[")
        .skip(1)
        .filter_map(|part| {
            let end = part.find(|c: char| ('@'..='~').contains(&c))?;
            Some(&part[end + 1..])
        })
        .collect()
}
