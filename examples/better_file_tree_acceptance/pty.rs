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
    for directory in ["workspace/folder", "pty/home", "pty/config/rev"] {
        std::fs::create_dir_all(root.join(directory))?;
    }
    let config = format!(
        "{}[gutter]\nvisible=false\nnerd_fonts={nerd_fonts}\n[bookmarks]\nmarkers=false\n",
        crate::support::config(profile)
            .replace("visible=false", "visible=false\nwidth=18\nmax_fraction=0.5")
    );
    std::fs::write(root.join("workspace/terminal.txt"), DOCUMENT)?;
    std::fs::write(root.join("workspace/other.txt"), "other document\n")?;
    std::fs::write(root.join("workspace/folder/child.txt"), "child\n")?;
    std::fs::write(root.join("pty/config/rev/config.toml"), &config)?;
    std::fs::write(root.join("pty/size"), "40 12\n")?;
    let started = Instant::now();
    let mut terminal = Terminal::start(root, binary, wrapper, "terminal.txt")?;
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}focus-file-tree\r").as_bytes())?;
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
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label) && row(frame, 2).starts_with(closed)
    })?;
    terminal.send(b"\x1b[B\x1b[C")?;
    terminal
        .frame(|frame| row(frame, 2).starts_with(open) && row(frame, 3).contains("child.txt"))?;
    // The kind field is part of the existing full-row activation target.
    terminal.send(b"\x1b[<0;5;4M\x1b[<0;5;4m")?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label) && row(frame, 1).contains("other document")
    })?;
    terminal.resize(12, 31)?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with("other document") && !frame.contains(root_label)
    })?;
    terminal.resize(12, 64)?;
    terminal.frame(|frame| {
        row(frame, 1).starts_with(root_label)
            && row(frame, 1).contains("other document")
            && row(frame, 2).starts_with(open)
    })?;
    terminal.idle()?;
    terminal.finish()?;
    for (name, expected) in [
        ("terminal.txt", DOCUMENT),
        ("other.txt", "other document\n"),
        ("folder/child.txt", "child\n"),
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
    println!(
        "PASS Linux PTY {profile}: root/directory symbols, arrow expansion, icon-cell file activation, 40/31/64-column resize, idle, unchanged disk/configuration and restored TTY; elapsed_ms={} output_bytes={}",
        started.elapsed().as_millis(),
        terminal.bytes()
    );
    Ok(())
}
