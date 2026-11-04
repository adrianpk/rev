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
        tool_journey(root, binary, wrapper, profile)?;
        journey(root, binary, wrapper, profile)?;
    }
    super::failure::run(root, binary, wrapper)
}
fn prepare(root: &Path, profile: &str, case: &str) -> Result<PathBuf> {
    let owner = root.join(format!("host-{profile}-{case}"));
    let directory = repository(&owner, "")?;
    let config = settings(&directory, profile)?;
    fs::create_dir_all(owner.join("pty/config/rev"))?;
    fs::write(
        owner.join("pty/config/rev/config.toml"),
        fs::read_to_string(config)?,
    )?;
    fs::create_dir_all(owner.join("pty/home"))?;
    Ok(owner)
}
fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let owner = prepare(root, profile, "input")?;
    let mut terminal = Terminal::start(&owner, binary, wrapper, "source.org")?;
    initial(&owner, &mut terminal, profile).map_err(|e| format!("initial: {e}"))?;
    recorder(&owner, &mut terminal, profile).map_err(|e| format!("recorder: {e}"))?;
    closure(&owner, &mut terminal).map_err(|e| format!("closure: {e}"))?;
    terminal.finish()?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/source.org"))?,
        ORIGINAL
    );
    println!(
        "PASS nested host PTY profile={profile}: real shells, multiple sessions, copy/paste, literal-next, mouse, protected source, declined shutdown, actual exit and full restoration; output_bytes={}",
        terminal.bytes()
    );
    Ok(())
}
fn named(terminal: &mut Terminal, name: &str) -> Result<()> {
    // An explicit Alt-x event avoids the legacy ESC/x transport ambiguity:
    // a separately decoded Escape correctly belongs to the live child.
    terminal.send(b"\x1b[120;3u")?;
    terminal
        .frame(|f| (1..=24).any(|number| row(f, number).starts_with("M-x ")))
        .map_err(|e| format!("{name} command prompt: {e}"))?;
    terminal.send(format!("\x1b[200~{name}\x1b[201~\r").as_bytes())
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
    terminal
        .frame(|f| row(f, 8).contains("PTY_SHELL_READY") && plain(f).contains("SHELL_READY> 2 "))?;
    assert!(pid_file(&owner.join("workspace/shell.pid"))? > 0);
    Ok(())
}
fn lazygit(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    line(
        terminal,
        "sh -c 'printf %s $$ >tool.pid; exec lazygit --use-config-file lazygit.yml'; printf '%s' \"$?\" >tool.exit",
    )?;
    terminal
        .frame(|f| plain(f).contains("Files"))
        .map_err(|e| format!("lazygit files: {e}"))?;
    let tool = pid_file(&owner.join("workspace/tool.pid"))?;
    terminal.send(b"?")?;
    terminal
        .frame(|f| plain(f).contains("Keybindings"))
        .map_err(|e| format!("lazygit help: {e}"))?;
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
    let panel = name == "terminal-move-to-panel";
    terminal
        .frame(|f| {
            plain(f).contains("Keybindings") && row(f, 1).contains("Protected source") == panel
        })
        .map_err(|e| format!("{name}: {e}"))?;
    Ok(())
}
fn quit_tool(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1b")?;
    terminal
        .frame(|f| plain(f).contains("Files") && !plain(f).contains("Keybindings"))
        .map_err(|e| format!("Escape from lazygit help: {e}"))?;
    terminal.send(b"q")?;
    terminal
        // Leaving the alternate screen restores an old prompt before Bash has
        // reaped the tool and prepared readline. Require the new generation.
        .frame(|f| plain(f).contains("SHELL_READY> 3 "))
        .map_err(|e| format!("lazygit quit: {e}"))?;
    Ok(())
}
fn recorder(owner: &Path, terminal: &mut Terminal, profile: &str) -> Result<()> {
    start_recorder(terminal)?;
    terminal.send(b"\x11\x07\x1b[<0;5;3M\x1b[<0;5;3m\x1b[<68;5;3M")?;
    copy(terminal, profile)?;
    terminal.frame(|f| plain(f).contains("INPUT_ACK") && main_prompt(f, 2))?;
    assert_eq!(
        fs::read(owner.join("workspace/input.bin"))?,
        b"\x07\x1b[<0;5;3M\x1b[<0;5;3m\x1b[200~paste\x1b[201~"
    );
    terminal.idle()?;
    Ok(())
}
fn copy(terminal: &mut Terminal, profile: &str) -> Result<()> {
    // Bash's echoed recorder command is older retained text. Select the
    // deterministic visible word only after explicit native history clearing.
    named(terminal, "terminal-clear-scrollback")?;
    terminal.frame(|f| plain(f).contains("scrollback cleared"))?;
    terminal.send(b"\x03\x14")?;
    terminal.frame(|f| plain(f).contains("[COPY]"))?;
    let keys: &[u8] = if profile == "modal" {
        b"gg0y5l"
    } else {
        b"\x1b<\x00\x1bf\x1bw"
    };
    terminal.send(keys)?;
    terminal.frame(|f| plain(f).contains("[COPY]") && plain(f).contains("copied"))?;
    terminal.send(b"\r\x19")?;
    Ok(())
}
fn closure(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    decline_exit(terminal).map_err(|e| format!("decline exit: {e}"))?;
    close_first(owner, terminal).map_err(|e| format!("close first: {e}"))?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/kept-state"))?,
        "alive"
    );
    switch(terminal, "terminal-2")?;
    line(terminal, "exit 7")?;
    terminal.frame(|f| plain(f).contains("Buffer closed") && plain(f).contains("source.org"))?;
    reaped(pid_file(&owner.join("workspace/shell.pid"))?)?;
    Ok(())
}
fn decline_exit(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x18\x13")?;
    terminal.frame(|f| plain(f).contains("requires a source buffer"))?;
    terminal.send(b"\x18\x03")?;
    terminal.frame(|f| plain(f).contains("terminate 2 terminal sessions"))?;
    terminal.send(b"n")?;
    terminal.frame(|f| plain(f).contains("sessions retained"))?;
    Ok(())
}
fn switch(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(b"\x18b")?;
    terminal.frame(|f| plain(f).contains("Switch to buffer:"))?;
    terminal.send(format!("{name}\r").as_bytes())?;
    terminal
        .frame(|f| {
            let text = plain(f);
            text.contains(name)
                && text.contains("SHELL_READY>")
                && !text.contains("Switch to buffer:")
        })
        .map_err(|e| format!("switch {name}: {e}"))?;
    Ok(())
}
fn close_first(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    switch(terminal, "terminal")?;
    shell_state(owner, terminal)?;
    terminal.send(b"\x18k")?;
    terminal
        .frame(|f| plain(f).contains("Kill buffer:"))
        .map_err(|e| format!("kill prompt: {e}"))?;
    terminal.send(b"\r")?;
    terminal
        .frame(|f| plain(f).contains("Terminate"))
        .map_err(|e| format!("terminate prompt: {e}"))?;
    terminal.send(b"y")?;
    terminal
        .frame(|f| plain(f).contains("Buffer closed"))
        .map_err(|e| format!("closed receipt: {e}"))?;
    Ok(())
}

fn shell_state(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    line(terminal, "printf '%s' \"$KEPT\" >kept-state")?;
    terminal
        .frame(|f| {
            plain(f).contains("SHELL_READY>")
                && fs::read_to_string(owner.join("workspace/kept-state"))
                    .is_ok_and(|text| text == "alive")
        })
        .map_err(|e| format!("shell state acknowledgement: {e}"))?;
    Ok(())
}

fn start_recorder(terminal: &mut Terminal) -> Result<()> {
    named(terminal, "terminal-new-buffer")?;
    terminal.frame(|f| plain(f).contains("terminal-2") && main_prompt(f, 1))?;
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

fn tool_journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let owner = prepare(root, profile, "tool")?;
    let mut terminal = Terminal::start(&owner, binary, wrapper, "source.org")?;
    initial(&owner, &mut terminal, profile).map_err(|e| format!("tool {profile} initial: {e}"))?;
    lazygit(&owner, &mut terminal).map_err(|e| {
        format!(
            "tool {profile}: {e}; tool_exit={:?}",
            fs::read_to_string(owner.join("workspace/tool.exit"))
        )
    })?;
    finish_tool(&owner, &mut terminal).map_err(|e| format!("tool {profile} finish: {e}"))?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/source.org"))?,
        ORIGINAL
    );
    println!(
        "PASS nested tool PTY profile={profile}: actual lazygit help, panel/buffer movement, same PID/cwd/shell state, truthful exit and complete host restoration; output_bytes={}",
        terminal.bytes()
    );
    Ok(())
}
fn finish_tool(owner: &Path, terminal: &mut Terminal) -> Result<()> {
    reaped(pid_file(&owner.join("workspace/tool.pid"))?)?;
    assert_eq!(fs::read_to_string(owner.join("workspace/tool.exit"))?, "0");
    line(terminal, "printf '%s' \"$KEPT\" >kept-state; exit 7")?;
    terminal.frame(|f| plain(f).contains("Buffer closed") && plain(f).contains("source.org"))?;
    assert_eq!(
        fs::read_to_string(owner.join("workspace/kept-state"))?,
        "alive"
    );
    terminal.finish()
}

// The main terminal and lower panel can display different shell generations.
// A ready prompt in the inactive panel cannot acknowledge the main child.
fn main_prompt(frame: &str, generation: usize) -> bool {
    let prompt = format!("SHELL_READY> {generation} ");
    (1..=5).any(|number| row(frame, number).contains(&prompt))
}
