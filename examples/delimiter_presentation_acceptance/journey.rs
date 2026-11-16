use crate::{fixture, held, observe, observe::Observer, support::Result, terminal::Terminal};
use std::{path::Path, time::Instant};

// One owned controlling PTY per profile covers all styles in one journey.
pub fn run(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let config = fixture::prepare(root, profile)?;
    let started = Instant::now();
    let mut t = Terminal::start(root, binary, wrapper, "notes.txt")?;
    let mut observer = Observer::default();
    controls(&mut t, &mut observer, profile)?;
    save_reopen(&mut t, &mut observer, root)?;
    held::run(&mut t, &mut observer, profile)?;
    t.finish()?;
    fixture::verify(root, &config)?;
    assert!(observer.frames > 0);
    println!(
        "MEASURE presentation PTY profile={profile} elapsed_ms={} output_bytes={} completed_frames={} restored=true reaped=true; default depth/match/pulse/off, bounded prompt/cancel, effective hints, source/point/viewport, save/reopen and held stop pass",
        started.elapsed().as_millis(),
        t.bytes(),
        observer.frames
    );
    Ok(())
}

pub fn named(t: &mut Terminal, command: &str) -> Result<()> {
    t.send(format!("\x1bx{command}\r").as_bytes())
}

pub fn wait(
    t: &mut Terminal,
    observer: &mut Observer,
    predicate: impl Fn(&str) -> bool,
) -> Result<String> {
    t.updates(predicate, |frame| observer.record(frame))
}

pub fn choose(t: &mut Terminal, observer: &mut Observer, style: &str) -> Result<()> {
    named(t, "pair-highlight-style")?;
    wait(t, observer, |f| f.contains("Pair highlight style ("))?;
    t.send(format!("{style}\r").as_bytes())?;
    wait(t, observer, |f| {
        f.contains(&format!("Pair highlight style: {style}"))
    })?;
    Ok(())
}

fn controls(t: &mut Terminal, o: &mut Observer, profile: &str) -> Result<()> {
    activate(t, o, profile)?;
    prompts(t, o)?;
    choose(t, o, "match")?;
    form(t, o, "match", true)?;
    pulse(t, o)?;
    restore(t, o)
}

fn activate(t: &mut Terminal, o: &mut Observer, profile: &str) -> Result<()> {
    if profile == "modal" {
        t.send(b"i")?;
        wait(t, o, |f| f.contains("INSERT"))?;
    }
    t.send(b"\x06\x06\x06\x06\x06")?;
    form(t, o, "depth", true)
}

fn pulse(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    choose(t, o, "pulse")?;
    t.send(b"\x06\x02")?;
    form(t, o, "pulse", true)?;
    form(t, o, "pulse", false)
}

fn restore(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    choose(t, o, "off")?;
    form(t, o, "off", false)?;
    named(t, "pair-highlight-mode")?;
    wait(t, o, |f| f.contains("Pair highlight style: pulse"))?;
    choose(t, o, "depth")?;
    form(t, o, "depth", true)
}

fn form(t: &mut Terminal, o: &mut Observer, style: &str, active: bool) -> Result<()> {
    wait(t, o, |f| {
        observe::position(f, 1, 6)
            && observe::caret(f) == Some((1, 6))
            && observe::form(f, style, active)
    })?;
    Ok(())
}

fn prompts(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    hints(t, o)?;
    named(t, "pair-highlight-style")?;
    wait(t, o, |f| f.contains("Pair highlight style ("))?;
    t.send(format!("\x1b[200~{}\x1b[201~", "x".repeat(65)).as_bytes())?;
    wait(t, o, |f| f.contains("Prompt input limit exceeded"))?;
    t.send(b"\x07")?;
    form(t, o, "depth", true)
}

fn hints(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    t.send(b"\x1bxpair-highlight-mode")?;
    wait(t, o, |f| {
        f.contains("M-x pair-highlight-mode") && f.contains("[C-c h]")
    })?;
    t.send(b"\x07")?;
    form(t, o, "depth", true)
}

fn save_reopen(t: &mut Terminal, o: &mut Observer, root: &Path) -> Result<()> {
    save(t, o, root)?;
    reopen(t, o)
}

fn save(t: &mut Terminal, o: &mut Observer, root: &Path) -> Result<()> {
    t.send(b"\x05!\x18\x13")?;
    wait(t, o, |f| {
        f.contains("notes.txt -- RW")
            && std::fs::read_to_string(root.join("workspace/notes.txt"))
                .is_ok_and(|s| s == fixture::SAVED)
    })?;
    Ok(())
}

fn reopen(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    t.send(b"\x18k")?;
    wait(t, o, |f| f.contains("Kill buffer:"))?;
    t.send(b"\r")?;
    wait(t, o, |f| {
        !f.contains("Kill buffer:") && f.contains("*scratch*")
    })?;
    t.send(b"\x18\x06notes.txt\r")?;
    wait(t, o, |f| {
        f.contains("notes.txt -- RW") && observe::form(f, "depth", false)
    })?;
    Ok(())
}
