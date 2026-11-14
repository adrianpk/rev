use super::fixture::{self, Case, Colors, Format};
use crate::support::Result;
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path, format: Format) -> Result<()> {
    let colors = if std::env::var_os("NO_COLOR").is_some() {
        Colors::Monochrome
    } else if std::env::var("REV_CONTINUITY_COLORS").as_deref() == Ok("disabled") {
        Colors::Disabled
    } else {
        Colors::Enabled
    };
    for profile in ["non-modal", "modal"] {
        journey(
            root,
            binary,
            wrapper,
            Case {
                format,
                profile,
                indent: true,
                pretty: true,
                colors,
            },
        )?;
    }
    Ok(())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, case: Case) -> Result<()> {
    let started = Instant::now();
    let source = prepare(root, case)?;
    let settings = case.settings(root)?;
    let name = format!("continuity.{}", case.format.name());
    let mut terminal = Terminal::start(root, binary, wrapper, &name)?;
    for two_rows in [false, true] {
        burst(&mut terminal, root, case, &settings, two_rows)?;
    }
    finish(&mut terminal, case, &settings)?;
    assert_eq!(
        std::fs::read(root.join("workspace").join(name))?,
        source.as_bytes()
    );
    println!(
        "PASS continuity PTY {case:?}: 120x30 -> 32x12, body/two-row edits, all emitted updates, source/pretty/reset, exact undo/save, idle silence and TTY restoration; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn prepare(root: &Path, case: Case) -> Result<String> {
    let source = case.format.document(0, 3);
    std::fs::write(
        root.join(format!("workspace/continuity.{}", case.format.name())),
        &source,
    )?;
    std::fs::write(root.join("pty/config/rev/config.toml"), case.config())?;
    std::fs::write(root.join("pty/size"), "120 30\n")?;
    Ok(source)
}

fn burst(
    terminal: &mut Terminal,
    root: &Path,
    case: Case,
    settings: &rev::config::Snapshot,
    two_rows: bool,
) -> Result<()> {
    let first = terminal.frame(ready)?;
    let baseline = fixture::sample(&first, case, settings);
    let input = input(case.profile, two_rows);
    let started = Instant::now();
    terminal.send(input.as_bytes())?;
    let enqueue = started.elapsed();
    assert!(enqueue <= Duration::from_millis(180));
    let enqueued_workers = workers(root)?;
    let mut states = Vec::new();
    terminal.updates(
        |frame| edited(frame, two_rows),
        |frame| {
            let samples = fixture::sample(frame, case, settings);
            assert_eq!(samples, baseline);
            states.push((
                workers(root).unwrap(),
                samples,
                row(frame, 3),
                row(frame, 4),
            ));
        },
    )?;
    terminal.settle_updates(|frame| {
        let samples = fixture::sample(frame, case, settings);
        assert_eq!(samples, baseline);
        states.push((
            workers(root).unwrap(),
            samples,
            row(frame, 3),
            row(frame, 4),
        ));
    })?;
    assert!(!states.is_empty());
    let workers = workers(root)?;
    assert_eq!(workers, 0, "exact work must finish before source controls");
    println!(
        "MEASURE PTY burst {case:?}: two_rows={two_rows} keys=5 enqueue_us={} observed_updates={} enqueued_workers={enqueued_workers} settled_workers={workers} states={states:?}; CPU-pending during input is not inferred",
        enqueue.as_micros(),
        states.len()
    );
    undo(terminal, two_rows)
}

fn finish(terminal: &mut Terminal, case: Case, settings: &rev::config::Snapshot) -> Result<()> {
    controls(terminal, case)?;
    terminal.resize(12, 32)?;
    let narrow = terminal.frame(|frame| ready(frame) && row(frame, 1).chars().count() == 32)?;
    fixture::sample(&narrow, case, settings);
    command(terminal, "save-buffer")?;
    terminal.frame(|frame| frame.contains("Saved") && frame.contains("-- RW"))?;
    terminal.idle()?;
    terminal.finish()
}

fn ready(frame: &str) -> bool {
    row(frame, 1).contains("◉ Root")
        && row(frame, 3).contains("○ Child")
        && row(frame, 5).contains("✸ Deep")
}

fn edited(frame: &str, two_rows: bool) -> bool {
    if two_rows {
        row(frame, 3).contains("Childzzz") && row(frame, 4).contains("Child body.zz")
    } else {
        row(frame, 4).contains("Child body.zzzzz")
    }
}

fn command(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(format!("\x1bx{name}\r").as_bytes())
}

fn input(profile: &str, two_rows: bool) -> String {
    let row = if two_rows { 3 } else { 4 };
    let enter = if profile == "modal" { "A" } else { "\x05" };
    let end = if profile == "modal" { "\x1b[F" } else { "\x05" };
    let mut input = format!("\x1bxgoto-line\r{row}\r{enter}");
    for key in 0..5 {
        if two_rows && key > 0 {
            input.push_str(if key % 2 == 0 { "\x1b[A" } else { "\x1b[B" });
            input.push_str(end);
        }
        input.push('z');
    }
    if profile == "modal" {
        input.push('\x1b');
    }
    input
}

fn undo(terminal: &mut Terminal, two_rows: bool) -> Result<()> {
    let groups = if two_rows { 5 } else { 1 };
    terminal.send("\x1bxundo\r".repeat(groups).as_bytes())?;
    terminal.frame(|frame| {
        ready(frame) && !row(frame, 3).contains("Childz") && !row(frame, 4).contains("body.z")
    })?;
    Ok(())
}

fn controls(terminal: &mut Terminal, case: Case) -> Result<()> {
    let (toggle, reset) = if case.format == Format::Org {
        ("org-pretty-headings-mode", "org-reset-heading-presentation")
    } else {
        ("markdown-pretty-mode", "markdown-reset-presentation")
    };
    command(terminal, toggle)?;
    terminal
        .frame(|frame| row(frame, 3).contains(&format!("{0}{0} Child", case.format.marker())))?;
    command(terminal, reset)?;
    terminal.frame(ready)?;
    Ok(())
}

fn workers(root: &Path) -> Result<usize> {
    let pid = std::fs::read_to_string(root.join("pty/editor.pid"))?;
    let tasks = std::fs::read_dir(format!("/proc/{}/task", pid.trim()))?;
    let mut count = 0;
    for task in tasks {
        match std::fs::read_to_string(task?.path().join("comm")) {
            Ok(name) => count += usize::from(name.trim() == "org-observation"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(count)
}
