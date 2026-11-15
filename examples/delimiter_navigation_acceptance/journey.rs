use crate::{
    support::Result,
    terminal::{Terminal, row},
};
use std::{fs, path::Path, time::Instant};

const SOURCE: &str = "(one [two]) tail\n";
const COPIED: &str = "(one [two]) tail\n(one [two])";

// One controlling PTY per profile uses actual Meta/control bytes. Saved source
// effects, rather than cursor labels alone, independently prove each endpoint.
pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path, profile: &str) -> Result<()> {
    assert!(matches!(profile, "non-modal" | "modal"));
    let config = configuration(profile);
    prepare(root, &config)?;
    let started = Instant::now();
    let mut t = Terminal::start(root, binary, wrapper, "notes.txt")?;
    workflow(&mut t, root, profile)?;
    t.idle()?;
    t.finish()?;
    assert_eq!(
        fs::read_to_string(root.join("pty/config/rev/config.toml"))?,
        config
    );
    println!(
        "MEASURE navigation PTY profile={profile} elapsed_ms={} output_bytes={} restored=true reaped=true; six C-M chords/named mate/counts/mark/readonly/focus/hints/undo/save/reopen pass",
        started.elapsed().as_millis(),
        t.bytes()
    );
    Ok(())
}

fn workflow(t: &mut Terminal, root: &Path, profile: &str) -> Result<()> {
    activate(t, profile)?;
    endpoints(t, root, profile)?;
    hints(t, profile)?;
    focus(t, root)?;
    mark(t, root)?;
    readonly(t)?;
    reopen(t, root)
}

fn activate(t: &mut Terminal, profile: &str) -> Result<()> {
    if profile == "modal" {
        t.send(b"i")?;
        t.frame(|f| f.contains("INSERT"))?;
    }
    Ok(())
}

fn configuration(profile: &str) -> String {
    let keymap = if profile == "modal" {
        "[keys.vim_insert]\n'C-M-f'='forward-char'\n'M-j'='forward-sexp'\n"
    } else {
        ""
    };
    format!(
        "editor.interaction_profile='{profile}'\npairs.enabled=false\npairs.quotes=false\ntree.visible=false\ncompletion.automatic=false\ngutter.visible=false\nsyntax.enabled=false\nshell.program='/bin/sh'\nshell.timeout_seconds=3\n{keymap}"
    )
}

fn prepare(root: &Path, config: &str) -> Result<()> {
    for directory in ["workspace", "pty/home", "pty/config/rev"] {
        fs::create_dir_all(root.join(directory))?;
    }
    fs::write(root.join("pty/config/rev/config.toml"), config)?;
    fs::write(root.join("workspace/notes.txt"), SOURCE)?;
    Ok(())
}

fn endpoints(t: &mut Terminal, root: &Path, profile: &str) -> Result<()> {
    let sexp = if profile == "modal" {
        b"\x1bj".as_slice()
    } else {
        b"\x1b\x06"
    };
    let forward = [b"\x1b\x04".as_slice(), sexp, b"f"].concat();
    let backward = [b"\x1b\x04".as_slice(), sexp, b"\x1b\x02b"].concat();
    let counted = [b"\x1b\x04\x152".as_slice(), sexp, b"c"].concat();
    for (input, expected) in [
        (b"\x1b\x0en".as_slice(), "(one [two])n tail\n"),
        (b"\x1b\x0e\x1b\x10p", "p(one [two]) tail\n"),
        (b"\x1b\x04\x1b\x04d", "(one [dtwo]) tail\n"),
        (b"\x1b\x04\x1b\x04\x1b\x15u", "(one u[two]) tail\n"),
        (&forward, "(onef [two]) tail\n"),
        (&backward, "(bone [two]) tail\n"),
        (&counted, "(one [two]c) tail\n"),
        (b"\x1bxmatching-delimiter\rm", "(one [two]m) tail\n"),
    ] {
        effect(t, root, input, expected)?;
        effect(t, root, b"\x1f\x1b<", SOURCE)?;
    }
    if profile == "modal" {
        effect(t, root, b"\x1b\x04\x1b\x06O", "(oOne [two]) tail\n")?;
        effect(t, root, b"\x1f\x1b<", SOURCE)?;
    }
    Ok(())
}

fn effect(t: &mut Terminal, root: &Path, input: &[u8], expected: &str) -> Result<()> {
    let mut input = input.to_vec();
    input.extend_from_slice(b"\x18\x13");
    t.send(&input)?;
    t.frame(|f| {
        f.contains("-- RW")
            && fs::read_to_string(root.join("workspace/notes.txt")).is_ok_and(|s| s == expected)
    })?;
    Ok(())
}

fn hints(t: &mut Terminal, profile: &str) -> Result<()> {
    let forward = if profile == "modal" {
        "[M-j]"
    } else {
        "[C-M-f]"
    };
    for (name, hint) in [("forward-sexp", forward), ("backward-sexp", "[C-M-b]")] {
        t.send(format!("\x1bx{name}").as_bytes())?;
        t.frame(|f| f.contains(name) && f.contains(hint))?;
        t.send(b"\x07")?;
        t.frame(|f| !f.contains("M-x ") && f.contains("L1 C1"))?;
    }
    Ok(())
}

fn focus(t: &mut Terminal, root: &Path) -> Result<()> {
    t.send(b"\x18\x06")?;
    t.frame(|f| f.contains("Find file:"))?;
    t.send(b"\x1b\x0e\x07")?;
    t.frame(|f| !f.contains("Find file:") && f.contains("L1 C1"))?;
    effect(t, root, b"F", "F(one [two]) tail\n")?;
    effect(t, root, b"\x1f\x1b<", SOURCE)
}

fn mark(t: &mut Terminal, root: &Path) -> Result<()> {
    // The copied region proves its original anchor survived forward-pair.
    // M-w deactivates selection before the one ordinary yank edit.
    effect(t, root, b"\x00\x1b\x0e\x1bw\x1b>\x19", COPIED)
}

fn named(t: &mut Terminal, command: &str) -> Result<()> {
    t.send(format!("\x1bx{command}\r").as_bytes())
}

fn readonly(t: &mut Terminal) -> Result<()> {
    // Native retained shell stdout is a canonical read-only source buffer.
    // No new CLI flags, filesystem-permission assumptions or editor hooks.
    readonly_output(t)?;
    readonly_navigation(t)?;
    return_buffer(t)
}

fn readonly_output(t: &mut Terminal) -> Result<()> {
    named(t, "shell-command")?;
    t.frame(|f| f.contains("Shell command:"))?;
    t.send(b"printf '(one [two]) tail\\n'\r")?;
    t.frame(|f| f.contains("shell-command-output opens"))?;
    named(t, "shell-command-output")?;
    t.frame(|f| f.contains("RO") && row(f, 1).contains("(one [two]) tail"))?;
    Ok(())
}

fn readonly_navigation(t: &mut Terminal) -> Result<()> {
    named(t, "forward-sexp")?;
    t.frame(|f| f.contains("RO") && f.contains("L1 C12"))?;
    named(t, "matching-delimiter")?;
    t.frame(|f| f.contains("RO") && f.contains("L1 C1"))?;
    named(t, "newline")?;
    t.frame(|f| f.contains("read-only") && row(f, 1).contains("(one [two]) tail"))?;
    Ok(())
}

fn return_buffer(t: &mut Terminal) -> Result<()> {
    named(t, "switch-to-buffer")?;
    t.frame(|f| f.contains("Switch to buffer:"))?;
    t.send(b"notes.txt\r")?;
    t.frame(|f| f.contains("notes.txt -- RW") && row(f, 2).contains("(one [two])"))?;
    Ok(())
}

fn reopen(t: &mut Terminal, root: &Path) -> Result<()> {
    t.send(b"\x18k")?;
    t.frame(|f| f.contains("Kill buffer:"))?;
    t.send(b"\r")?;
    t.frame(|f| !f.contains("Kill buffer:") && !f.contains("notes.txt -- RW"))?;
    t.send(b"\x18\x06notes.txt\r")?;
    t.frame(|f| f.contains("notes.txt -- RW") && row(f, 2).contains("(one [two])"))?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/notes.txt"))?,
        COPIED
    );
    Ok(())
}
