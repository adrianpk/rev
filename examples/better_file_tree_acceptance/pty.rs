use crate::support::{DOCUMENT, Result};
use crate::terminal::{Terminal, row};
use std::path::Path;
use std::time::Instant;

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    for (profile, nerd_fonts) in [("non-modal", false), ("modal", true)] {
        journey(&root.join(profile), binary, wrapper, profile, nerd_fonts)?;
    }
    Ok(())
}

// A single real-input journey per profile complements the native matrix.
// The shared helper owns stream/deadline bounds, idle checks and shutdown.
fn journey(
    root: &Path,
    binary: &Path,
    wrapper: &Path,
    profile: &str,
    nerd_fonts: bool,
) -> Result<()> {
    let config = prepare(root, profile, nerd_fonts)?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.txt")?;
    let labels = focus_tree(&mut terminal, profile, nerd_fonts)?;
    viewport_controls(&mut terminal, &labels)?;
    activate_file(&mut terminal, &labels)?;
    resize_tree(&mut terminal, &labels)?;
    terminal.idle()?;
    terminal.finish()?;
    retained_files(root, &config)?;
    println!(
        "PASS Linux PTY {profile}: root/directory symbols, arrow panning, control expansion, shared deep/long viewport, local/prefixed width controls, icon-cell file activation, 40/31/64-column resize, idle, unchanged disk/configuration and restored TTY; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}

fn prepare(root: &Path, profile: &str, nerd_fonts: bool) -> Result<String> {
    for directory in [
        "workspace/folder/zdeep/nested",
        "pty/home",
        "pty/config/rev",
    ] {
        std::fs::create_dir_all(root.join(directory))?;
    }
    let config = format!(
        "{}[gutter]\nvisible=false\nnerd_fonts={nerd_fonts}\n[bookmarks]\nmarkers=false\n",
        crate::support::config(profile).replace(
            "visible=false",
            "visible=false\nauto_width=false\nwidth=18\nmax_fraction=0.5"
        )
    );
    std::fs::write(root.join("workspace/terminal.txt"), DOCUMENT)?;
    std::fs::write(root.join("workspace/other.txt"), "other document\n")?;
    std::fs::write(root.join("workspace/folder/child.txt"), "child\n")?;
    std::fs::write(
        root.join("workspace/folder/zdeep/nested/a-very-long-nested-name-ending.txt"),
        "nested\n",
    )?;
    std::fs::write(
        root.join("workspace/z-shallow-long-name-ending.txt"),
        "shallow\n",
    )?;
    std::fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    std::fs::write(root.join("pty/size"), "40 12\n")?;
    Ok(config)
}
struct Labels {
    root_label: &'static str,
    parent: &'static str,
    closed: &'static str,
    open: &'static str,
}
fn labels(nerd_fonts: bool) -> Labels {
    let root_label = if nerd_fonts {
        "▾ \u{f1b2} workspace/"
    } else {
        "▾ ◆ workspace/"
    };
    let closed = if nerd_fonts {
        "  ▸ \u{f07b} folder/"
    } else {
        "  ▸ / folder/"
    };
    let open = if nerd_fonts {
        "  ▾ \u{f07c} folder/"
    } else {
        "  ▾ / folder/"
    };
    Labels {
        root_label,
        parent: if nerd_fonts {
            "    \u{f148} .."
        } else {
            "    ↑ .."
        },
        closed,
        open,
    }
}
fn focus_tree(terminal: &mut Terminal, profile: &str, nerd_fonts: bool) -> Result<Labels> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}focus-file-tree\r").as_bytes())?;
    let labels = labels(nerd_fonts);
    let root_label = labels.root_label;
    let closed = labels.closed;
    let open = labels.open;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label)
            && row(frame, 2).starts_with(labels.parent)
            && row(frame, 3).starts_with(closed)
    })?;
    terminal.send(b"\x1b[B\x1b[B\x06")?;
    terminal
        .frame(|frame| row(frame, 3).starts_with(open) && row(frame, 5).contains("child.txt"))?;
    Ok(labels)
}
fn width_controls(terminal: &mut Terminal) -> Result<()> {
    for (keys, width) in [
        (b"<".as_slice(), 14),
        (b">".as_slice(), 18),
        (b"\x18t>".as_slice(), 20),
        (b"\x18t<".as_slice(), 18),
        (b"\x18t=".as_slice(), 18),
    ] {
        terminal.send(keys)?;
        terminal.frame(|frame| row(frame, 1).chars().nth(width) == Some('│'))?;
    }
    Ok(())
}

fn viewport_controls(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    width_controls(terminal)?;
    horizontal_viewport(terminal, labels)
}

// Actual arrows share the capped viewport across shallow and deep rows. The
// control routes alone expand/fold; reset returns every row to its beginning.
fn horizontal_viewport(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    terminal.send(b"\x18t>\x1b[C")?;
    terminal
        .frame(|frame| row(frame, 1).starts_with(" ") && row(frame, 5).contains("child.txt"))?;
    terminal.send(b"\x1b[D")?;
    terminal.frame(|frame| row(frame, 1).starts_with(labels.root_label))?;
    terminal.send(b"\x1b[B\x06\x1b[B\x06")?;
    terminal.frame(|frame| row(frame, 6).contains("a-very-l"))?;
    pan_ends(terminal, labels)?;
    reset_and_fold(terminal, labels)
}

fn pan_ends(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    terminal.send("\x1b[C".repeat(80).as_bytes())?;
    terminal.frame(|frame| {
        row(frame, 6)
            .split('│')
            .next()
            .is_some_and(|label| label.ends_with("-ending.txt"))
            && row(frame, 1).starts_with("                    │")
            && row(frame, 10).contains("ending.txt")
    })?;
    terminal.send("\x1b[D".repeat(80).as_bytes())?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(labels.root_label) && row(frame, 6).contains("a-very-l")
    })?;
    Ok(())
}

fn reset_and_fold(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    terminal.send(b"\x1b[C\x1b[C\x18t=")?;
    terminal.frame(|frame| row(frame, 1).starts_with(labels.root_label))?;
    terminal.send(b"\x02\x02\x02")?;
    terminal.frame(|frame| row(frame, 6).contains("other.txt") && !frame.contains("a-very-l"))?;
    Ok(())
}

fn activate_file(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    let root_label = labels.root_label;
    // The kind field is part of the existing full-row activation target.
    terminal.send(b"\x1b[<0;5;6M\x1b[<0;5;6m")?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label) && row(frame, 1).contains("other document")
    })?;
    Ok(())
}
fn resize_tree(terminal: &mut Terminal, labels: &Labels) -> Result<()> {
    let Labels {
        root_label, open, ..
    } = *labels;
    terminal.resize(12, 31)?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with("other document") && !frame.contains(root_label)
    })?;
    terminal.resize(12, 64)?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label)
            && row(frame, 1).contains("other document")
            && row(frame, 2).starts_with(labels.parent)
            && row(frame, 3).starts_with(open)
    })?;
    Ok(())
}
fn retained_files(root: &Path, config: &str) -> Result<()> {
    for (name, expected) in [
        ("terminal.txt", DOCUMENT),
        ("other.txt", "other document\n"),
        ("folder/child.txt", "child\n"),
        (
            "folder/zdeep/nested/a-very-long-nested-name-ending.txt",
            "nested\n",
        ),
        ("z-shallow-long-name-ending.txt", "shallow\n"),
    ] {
        assert_eq!(
            std::fs::read_to_string(root.join("workspace").join(name))?,
            expected
        );
    }
    assert_eq!(
        std::fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    Ok(())
}
