use crate::{
    repository::Repository,
    support::*,
    terminal::{Terminal, row},
};
use std::{fs, os::unix::fs::symlink, path::Path, time::Instant};
use unicode_width::UnicodeWidthStr;

// Decode real terminal bytes in both profiles. The shared PTY owner keeps its
// existing eight-second/1-MiB limits and verifies TTY modes on application exit.
pub(crate) fn run(binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        let repository = prepare(profile)?;
        let root = &repository.fixture.0;
        let started = Instant::now();
        let mut terminal = Terminal::start(root, binary, wrapper, "source.txt")?;
        println!(
            "MEASURE Git PTY profile={profile} startup_us={}",
            started.elapsed().as_micros()
        );
        startup_metrics(root)?;
        terminal.idle()?;
        git_entry(&mut terminal)?;
        workflows(&repository, &mut terminal)?;
        restore(&repository, &mut terminal, profile)?;
        println!(
            "PASS Git PTY profile={profile}: decoded entry, idle Status, source and TTY restoration; output_bytes={}",
            terminal.bytes()
        );
    }
    Ok(())
}

// Compare ordinary startup with the approved pre-Git binary using only settings
// understood by that candidate. No Git command is sent during this baseline.
pub(crate) fn startup(binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        let repository = prepare(profile)?;
        let root = &repository.fixture.0;
        fs::write(
            root.join("pty/config/rev/config.toml"),
            format!(
                "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"
            ),
        )?;
        let started = Instant::now();
        let mut terminal = Terminal::start(root, binary, wrapper, "source.txt")?;
        println!(
            "MEASURE pre-Git startup profile={profile} startup_us={}",
            started.elapsed().as_micros()
        );
        startup_metrics(root)?;
        terminal.idle()?;
        terminal.finish()?;
    }
    Ok(())
}

fn startup_metrics(root: &Path) -> Result<()> {
    let pid = fs::read_to_string(root.join("pty/editor.pid"))?;
    println!(
        "MEASURE editor first frame: cpu_ticks={} peak_rss={}",
        cpu_ticks(pid.trim())?,
        peak_rss(pid.trim())?
    );
    Ok(())
}

fn workflows(repository: &Repository, terminal: &mut Terminal) -> Result<()> {
    fs::write(repository.root.join("source.txt"), "WORKING_SENTINEL\n")?;
    terminal.send(b"g")?;
    let frame = terminal.frame(|f| f.contains("Unstaged") && f.contains("source.txt"))?;
    let path_row = (1..=24)
        .find(|n| row(&frame, *n).contains("source.txt"))
        .ok_or("missing semantic path row")?;
    terminal.send(format!("\x1b[<0;5;{path_row}M\x1b[<0;5;{path_row}m\t").as_bytes())?;
    terminal
        .frame(|f| f.contains("comparison retained as read-only") && f.contains("@@ -1 +1 @@"))?;
    stage_commit(repository, terminal)
}

fn stage_commit(repository: &Repository, terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"s")?;
    terminal.frame(|f| f.contains("Index updated and verified"))?;
    assert_eq!(
        repository.git(["show", ":source.txt"]),
        b"WORKING_SENTINEL\n"
    );
    draft(terminal)?;
    terminal.send(b"\x03\x03")?;
    terminal.frame(|f| f.contains("Commit created and verified"))?;
    assert_eq!(
        repository.git(["show", "HEAD:source.txt"]),
        b"WORKING_SENTINEL\n"
    );
    assert_eq!(
        repository.git(["log", "-1", "--format=%B"]),
        b"cc q PTY_ONLY\n"
    );
    terminal.resize(18, 72)?;
    terminal.frame(|f| f.contains("Git:") && row(f, 1).width() == 72)?;
    Ok(())
}

fn terminal_context(terminal: &mut Terminal, profile: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}terminal-open\r").as_bytes())?;
    terminal
        .frame(|f| plain_frame(f).contains("TERM_READY"))
        .map_err(|e| format!("terminal ready: {e}"))?;
    terminal_return(terminal)?;
    terminal_exit(terminal)
}

fn terminal_return(terminal: &mut Terminal) -> Result<()> {
    host_named(terminal, "git-status")?;
    terminal
        .frame(|f| f.contains("Git:") && f.contains("Working tree clean"))
        .map_err(|e| format!("Git entry: {e}"))?;
    terminal.send(b"q")?;
    terminal
        .frame(|f| plain_frame(f).contains("TERM_READY"))
        .map_err(|e| format!("terminal ready: {e}"))?;
    terminal.send(b"ping\r")?;
    terminal.frame(|f| plain_frame(f).contains("GOT:ping"))?;
    Ok(())
}

fn terminal_exit(terminal: &mut Terminal) -> Result<()> {
    // The normal guarded-exit coordinator needs explicit terminal consent.
    host_named(terminal, "save-buffers-kill-terminal")?;
    terminal.frame(|f| f.contains("terminal sessions"))?;
    terminal.send(b"n")?;
    terminal.frame(|f| f.contains("canceled"))?;
    terminal.send(b"exit\r")?;
    terminal.frame(|f| {
        plain_frame(f).contains("Buffer closed") && plain_frame(f).contains("source.txt")
    })?;
    Ok(())
}

fn prepare(profile: &str) -> Result<Repository> {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source.txt"), "SOURCE_SENTINEL\n")?;
    repository.commit();
    let root = &repository.fixture.0;
    symlink(&repository.root, root.join("workspace"))?;
    fs::create_dir_all(root.join("pty/config/rev"))?;
    fs::create_dir_all(root.join("pty/home"))?;
    let shell = repository.fixture.write(
        "terminal.sh",
        "printf 'TERM_READY\\n'\nwhile IFS= read -r line; do [ \"$line\" = exit ] && exit 0; printf 'GOT:%s\\n' \"$line\"; done\n",
    );
    repository.snapshot(&format!("[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[completion]\nautomatic=false\n[terminal]\nprogram='/bin/sh'\nargs=[{}]\n", serde_json::to_string(&shell)?));
    fs::copy(
        root.join("config.toml"),
        root.join("pty/config/rev/config.toml"),
    )?;
    Ok(repository)
}

fn host_named(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(b"\x1b[120;3u")?;
    terminal
        .frame(|f| f.contains("M-x "))
        .map_err(|e| format!("host prompt for {name}: {e}"))?;
    terminal.send(format!("\x1b[200~{name}\x1b[201~\r").as_bytes())
}

fn plain_frame(frame: &str) -> String {
    (1..=24)
        .map(|n| row(frame, n))
        .collect::<Vec<_>>()
        .join("\n")
}

fn git_entry(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x18g")?;
    terminal
        .frame(|f| f.contains("Git:") && f.contains("Working tree clean"))
        .map_err(|e| format!("Git entry: {e}"))?;
    terminal.idle()?;
    Ok(())
}

fn restore(repository: &Repository, terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(b"q")?;
    terminal.frame(|f| f.contains("source.txt") && !f.contains("Git:"))?;
    assert!(terminal.contains("source.txt"));
    terminal_context(terminal, profile).map_err(|e| format!("terminal context: {e}"))?;
    terminal.finish()?;
    assert_eq!(
        fs::read(repository.root.join("source.txt"))?,
        b"WORKING_SENTINEL\n"
    );
    Ok(())
}

fn draft(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"ccc q PTY_ONLY")?;
    terminal.frame(|f| f.contains("cc q PTY_ONLY"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| f.contains("Git:"))?;
    terminal.send(b"c")?;
    terminal.frame(|f| f.contains("cc q PTY_ONLY"))?;
    Ok(())
}
