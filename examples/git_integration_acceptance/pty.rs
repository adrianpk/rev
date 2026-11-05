use crate::{repository::Repository, support::*, terminal::Terminal};
use std::{fs, os::unix::fs::symlink, path::Path, time::Instant};

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
        terminal.send(b"\x18g")?;
        terminal.frame(|f| f.contains("Git:") && f.contains("Working tree clean"))?;
        terminal.idle()?;
        terminal.send(b"q")?;
        terminal.frame(|f| f.contains("SOURCE_SENTINEL") && !f.contains("Git:"))?;
        assert!(terminal.contains("source.txt"));
        terminal.finish()?;
        assert_eq!(
            fs::read(repository.root.join("source.txt"))?,
            b"SOURCE_SENTINEL\n"
        );
        println!(
            "PASS Git PTY profile={profile}: decoded entry, idle Status, source and TTY restoration; output_bytes={}",
            terminal.bytes()
        );
    }
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
    repository.snapshot(&format!("[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[completion]\nautomatic=false\n"));
    fs::copy(
        root.join("config.toml"),
        root.join("pty/config/rev/config.toml"),
    )?;
    Ok(repository)
}
