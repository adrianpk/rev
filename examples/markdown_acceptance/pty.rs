use super::support::{self, MARKERS, Result};
use crate::surface;
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for profile in ["non-modal", "modal"] {
        journey(root, binary, wrapper, profile)?;
    }
    Ok(())
}

fn journey(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    let original = prepare(root, profile)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.md")?;
    let frame = terminal.frame(pretty)?;
    actual_cells(&frame);
    let startup = started.elapsed();
    presentation(&mut terminal)?;
    list(&mut terminal, profile)?;
    table(&mut terminal).map_err(|error| format!("Markdown {profile} table: {error}"))?;
    restored(root, &mut terminal, &original)?;
    println!(
        "PASS Linux PTY Markdown {profile}: 80x24 -> 32x12, no_color={}, startup_us={}, output_bytes={}; actual cells, source/pretty, contextual Return, ordered continuation/undo, table align/insert/undo, exact save, idle silence and TTY restored",
        std::env::var_os("NO_COLOR").is_some(),
        startup.as_micros(),
        terminal.bytes()
    );
    Ok(())
}

fn prepare(root: &Path, profile: &str) -> Result<String> {
    let original = support::document(profile == "modal");
    std::fs::write(root.join("workspace/terminal.md"), &original)?;
    std::fs::write(
        root.join("pty/config/rev/config.toml"),
        support::config(profile),
    )?;
    std::fs::write(root.join("pty/size"), "80 24\n")?;
    Ok(original)
}

fn restored(root: &Path, terminal: &mut Terminal, original: &str) -> Result<()> {
    resized(terminal)?;
    command(terminal, "save-buffer")?;
    terminal.frame(|frame| visible(frame, "Saved \"") && visible(frame, "-- RW"))?;
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(
        std::fs::read(root.join("workspace/terminal.md"))?,
        original.as_bytes()
    );
    Ok(())
}

fn resized(terminal: &mut Terminal) -> Result<()> {
    command(terminal, "beginning-of-buffer")?;
    terminal.frame(pretty)?;
    terminal.resize(12, 32)?;
    let frame = terminal.frame(|frame| pretty(frame) && row(frame, 1).width() == 32)?;
    actual_cells(&frame);
    Ok(())
}

fn command(terminal: &mut Terminal, name: &str) -> Result<()> {
    terminal.send(format!("\x1bx{name}\r").as_bytes())
}

fn visible(frame: &str, needle: &str) -> bool {
    (1..=24).any(|line| row(frame, line).contains(needle))
}

// The padded delimiter distinguishes a published alignment from its original
// source. The visible heading symbol also requires the new observation, so a
// subsequent action cannot race a pending structural refresh.
fn aligned(frame: &str) -> bool {
    visible(frame, "| :--- | -----: |") && visible(frame, "▶ Level 6")
}

fn pretty(frame: &str) -> bool {
    MARKERS.iter().enumerate().all(|(line, marker)| {
        row(frame, line + 1)
            .chars()
            .skip(7)
            .collect::<String>()
            .starts_with(&format!("{}{marker} Level {}", " ".repeat(line), line + 1))
    })
}

fn actual_cells(frame: &str) {
    let cells = surface::decode(frame.as_bytes().to_vec());
    for (line, marker) in MARKERS.iter().enumerate() {
        let cell = cells[&(line, 7 + line)];
        assert_eq!(cell.character, *marker);
        if std::env::var_os("NO_COLOR").is_some() {
            assert_eq!(cell.foreground, None);
            assert_eq!(cell.background, None);
        } else {
            assert_eq!(cell.foreground, Some(crate::native::COLORS[line]));
        }
    }
}

fn presentation(terminal: &mut Terminal) -> Result<()> {
    command(terminal, "markdown-pretty-mode")?;
    terminal.frame(|frame| row(frame, 3).contains("### Level 3"))?;
    command(terminal, "markdown-reset-presentation")?;
    terminal.frame(pretty)?;
    Ok(())
}

fn goto(terminal: &mut Terminal, line: usize) -> Result<()> {
    terminal.send(format!("\x1bxgoto-line\r{line}\r").as_bytes())
}

fn list(terminal: &mut Terminal, profile: &str) -> Result<()> {
    goto(terminal, 19)?;
    terminal.frame(|frame| frame.contains("3. Ordered"))?;
    terminal.send(if profile == "modal" { b"i\r" } else { b"\r" })?;
    terminal.frame(|frame| {
        (1..=22).any(|line| row(frame, line).chars().skip(7).collect::<String>().trim() == "4.")
    })?;
    command(terminal, "undo")?;
    terminal.frame(|frame| frame.contains("terminal.md -- RW") && frame.contains("4. Next"))?;
    Ok(())
}

fn table(terminal: &mut Terminal) -> Result<()> {
    align(terminal).map_err(|error| format!("align: {error}"))?;
    command(terminal, "markdown-table-insert-column")?;
    terminal
        .frame(|frame| {
            (1..=22).any(|line| {
                let text = row(frame, line);
                text.contains("Name") && text.matches('|').count() == 4
            })
        })
        .map_err(|error| format!("insert column: {error}"))?;
    command(terminal, "undo")?;
    terminal.frame(aligned)?;
    command(terminal, "undo")?;
    terminal.frame(|frame| frame.contains("terminal.md -- RW"))?;
    Ok(())
}

fn align(terminal: &mut Terminal) -> Result<()> {
    goto(terminal, 26)?;
    terminal.frame(|frame| visible(frame, "界e\u{301}") && visible(frame, "a\\|b"))?;
    command(terminal, "markdown-table-align")?;
    terminal.frame(aligned)?;
    Ok(())
}
