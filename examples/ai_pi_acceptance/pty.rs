use super::{
    fixture,
    support::*,
    terminal::{Terminal, row},
};
use std::{fs, path::Path, time::Instant};

fn named(t: &mut Terminal, profile: &str, name: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    t.send(format!("{prefix}{name}\r").as_bytes())
}

// Reuse the existing eight-second/1-MiB terminal owner. A real rename wakes
// the editor while another native prompt owns input; stale AI cannot apply.
pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let project = fixture::prepare(root, true)?;
    let profile = std::env::var("REV_ACCEPTANCE_PROFILE")?;
    let started = Instant::now();
    let mut t = Terminal::start(root, binary, wrapper)?;
    t.frame(|frame| frame.contains("needle"))?;
    println!(
        "MEASURE PTY profile={profile} startup_ms={}",
        started.elapsed().as_millis()
    );
    t.idle()?;
    named(&mut t, &profile, "ai-implement")?;
    t.frame(|frame| frame.contains("AI insert"))?;
    t.send(b"ACCEPTANCE-FIRST: insert a comment.\r")?;
    t.frame(|frame| frame.contains("proposal ready"))?;
    let pid = fixture::pid(&project)?;
    named(&mut t, &profile, "switch-to-buffer")?;
    t.frame(|frame| frame.contains("buffer"))?;
    t.send(b"terminal.rs\r")?;
    t.frame(|frame| frame.contains("needle"))?;
    t.send(if profile == "modal" {
        b":find-f"
    } else {
        b"\x1bxfind-f"
    })?;
    t.frame(|frame| frame.contains("find-f"))?;
    fs::write(
        project.join("replacement"),
        "const external_needle: u8 = 9;\n",
    )?;
    let started = Instant::now();
    fs::rename(project.join("replacement"), project.join("terminal.rs"))?;
    let frame = t.frame(|frame| frame.contains("external_needle") && frame.contains("find-f"))?;
    assert!(row(&frame, 24).contains("find-f"));
    println!(
        "MEASURE PTY input_preserving_reload_ms={}",
        started.elapsed().as_millis()
    );
    t.send(b"\x07")?;
    t.frame(|frame| frame.contains("external_needle"))?;
    named(&mut t, &profile, "switch-to-buffer")?;
    t.frame(|frame| frame.contains("buffer"))?;
    t.send(b"*AI Result*\r")?;
    t.frame(|frame| frame.contains("Controlled offline"))?;
    named(&mut t, &profile, "ai-apply-result")?;
    t.frame(|frame| frame.contains("stale"))?;
    named(&mut t, &profile, "ai-discard-result")?;
    t.frame(|frame| frame.contains("external_needle"))?;
    t.idle()?;
    t.finish()?;
    assert!(t.contains("REV_TTY_RESTORED"));
    reaped(&[pid])?;
    assert_eq!(
        fs::read_to_string(project.join("terminal.rs"))?,
        "const external_needle: u8 = 9;\n"
    );
    println!(
        "PASS PTY profile={profile} bounded_bytes={} input ownership, stale result, owned exit and terminal restoration",
        t.bytes()
    );
    Ok(())
}
