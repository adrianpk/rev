use crate::{
    repository::Repository,
    support::Result,
    terminal::{Terminal, row},
};
use std::{fs, path::Path, time::Instant};

const SOURCE: &str = "  alpha\nsecond\n";
const EDITED: &str = "  Xalpha\nsecond\n";

// Physical Meta/control bytes enter the real terminal parser. Source bytes,
// undo/redo and mode/prefix effects establish behavior beyond process exit.
pub(super) fn editing(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    let config = format!(
        "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"
    );
    prepare(root, &config)?;
    fs::write(root.join("workspace/notes.txt"), SOURCE)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "notes.txt")?;
    editing_steps(&mut terminal, profile)?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/notes.txt"))?,
        EDITED
    );
    assert_eq!(
        fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    println!(
        "PASS Linux PTY {profile}: Meta first-text, exact insertion/undo/redo/save, cancelled project/Org prefixes, Text/Org transitions, effective hint and pane chord; configuration retained, TTY restored; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn editing_steps(terminal: &mut Terminal, profile: &str) -> Result<()> {
    first_edit(terminal, profile)?;
    history(terminal)?;
    prefixes(terminal)?;
    modes(terminal)?;
    finish_editing(terminal)
}

fn first_edit(terminal: &mut Terminal, profile: &str) -> Result<()> {
    terminal.send(b"\x1bm")?;
    terminal.frame(|f| f.contains("L1 C3"))?;
    terminal.send(if profile == "modal" { b"iX" } else { b"X" })?;
    terminal.frame(|f| row(f, 1).contains("  Xalpha"))?;
    Ok(())
}

fn finish_editing(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x18+")?;
    terminal.frame(|f| row(f, 1).contains("  Xalpha"))?;
    terminal.idle()?;
    terminal.finish()
}

fn history(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1f")?;
    terminal.frame(|f| row(f, 1).contains("  alpha") && !row(f, 1).contains('X'))?;
    terminal.send(b"\x1b\x1f")?;
    terminal.frame(|f| row(f, 1).contains("  Xalpha"))?;
    terminal.send(b"\x18\x13")?;
    terminal.frame(|f| f.contains("Saved") && f.contains("-- RW"))?;
    Ok(())
}

fn prefixes(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x18p")?;
    terminal.frame(|f| f.contains("C-x p"))?;
    terminal.send(b"\x07Z")?;
    terminal.frame(|f| row(f, 1).contains("  XZalpha"))?;
    terminal.send(b"\x1f")?;
    terminal.frame(|f| row(f, 1).contains("  Xalpha") && !row(f, 1).contains('Z'))?;
    terminal.send(b"\x18\x13")?;
    terminal.frame(|f| f.contains("-- RW"))?;
    Ok(())
}

fn modes(terminal: &mut Terminal) -> Result<()> {
    org_prefix(terminal)?;
    named(terminal, "text-mode")?;
    terminal.frame(|f| row(f, 23).split_whitespace().any(|part| part == "Text"))?;
    discovery(terminal)
}

fn org_prefix(terminal: &mut Terminal) -> Result<()> {
    named(terminal, "org-mode")?;
    terminal.frame(|f| row(f, 23).split_whitespace().any(|part| part == "Org"))?;
    terminal.send(b"\x03\x16")?;
    terminal.frame(|f| f.contains("C-c C-v"))?;
    terminal.send(b"\x07\x1bm")?;
    terminal.frame(|f| f.contains("L1 C3") && row(f, 1).contains("  Xalpha"))?;
    Ok(())
}

fn discovery(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bxbalance-windows")?;
    terminal.frame(|f| f.contains("balance-windows") && f.contains("[C-x +]"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| !f.contains("M-x "))?;
    Ok(())
}

fn named(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(format!("\x1bx{name}\r").as_bytes())
}

fn prepare(root: &Path, config: &str) -> Result<()> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        fs::create_dir_all(root.join(directory))?;
    }
    fs::write(root.join("pty/config/rev/config.toml"), config)?;
    Ok(())
}

// The third representative PTY uses only a tiny local repository. History
// paging/review/cancellation must retain exact HEAD, index, disk and source.
pub(super) fn git(binary: &Path, wrapper: &Path) -> Result<()> {
    let repository = git_fixture()?;
    let root = &repository.fixture.0;
    let config = fs::read_to_string(root.join("config.toml"))?;
    let head = repository.git(["rev-parse", "HEAD"]);
    let index = repository.git(["ls-files", "--stage", "-z"]);
    let source = fs::read(repository.root.join("notes.txt"))?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "notes.txt")?;
    git_history(&mut terminal, std::str::from_utf8(&head)?.trim())?;
    git_return(&mut terminal)?;
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(repository.git(["ls-files", "--stage", "-z"]), index);
    assert_eq!(fs::read(repository.root.join("notes.txt"))?, source);
    assert_eq!(
        fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    println!(
        "PASS Linux PTY local Git: + pinned ancestry paging, A exact revision review, cancelled c prefix, source return; HEAD/index/disk/configuration unchanged, no transport, TTY restored; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn git_fixture() -> Result<Repository> {
    let repository = Repository::new("sha256");
    for n in 0..3 {
        fs::write(repository.root.join("notes.txt"), format!("  git {n}\n"))?;
        repository.commit();
    }
    let root = &repository.fixture.0;
    std::os::unix::fs::symlink(&repository.root, root.join("workspace"))?;
    repository
        .snapshot("history_page_size=2\n[tree]\nvisible=false\n[completion]\nautomatic=false\n");
    let config = fs::read_to_string(root.join("config.toml"))?;
    prepare(root, &config)?;
    Ok(repository)
}

fn git_return(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"q")?;
    terminal.frame(|f| f.contains("Git:") && !f.contains("Git History"))?;
    terminal.send(b"q\x1bm")?;
    terminal.frame(|f| f.contains("L1 C3") && row(f, 1).contains("  git 2"))?;
    terminal.idle()?;
    terminal.finish()
}

fn git_history(terminal: &mut Terminal, head: &str) -> Result<()> {
    git_entry(terminal)?;
    terminal.send(b"+")?;
    terminal.frame(|f| f.contains("3 revisions / 2 pages") && f.contains("complete ancestry"))?;
    git_review(terminal, head)?;
    git_prefix(terminal)
}

fn git_entry(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x18g")?;
    terminal.frame(|f| f.contains("Git:") && f.contains("Working tree clean"))?;
    terminal.send(b"h")?;
    terminal.frame(|f| f.contains("Git History") && f.contains("more ancestry: +"))?;
    Ok(())
}

fn git_review(terminal: &mut Terminal, head: &str) -> Result<()> {
    terminal.send(b"A")?;
    let target = format!("1: {head}");
    terminal.frame(|f| f.contains("Ordered revision review") && f.contains(&target))?;
    terminal.send(b"\r")?;
    terminal.frame(|f| f.contains("Ordered revision review"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| f.contains("Git History"))?;
    Ok(())
}

fn git_prefix(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"c")?;
    terminal.frame(|f| (1..=24).any(|n| row(f, n).trim() == "c"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| f.contains("Git History"))?;
    Ok(())
}
