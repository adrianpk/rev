use crate::{
    support::Result,
    terminal::{Terminal, row},
};
use rev::lsp::install::{Registry, State};
use std::{fs, path::Path};

// The shared eight-second/1-MiB PTY boundary retains terminal restoration and
// idle assertions. Both profiles use their actual named-command key sequence.
pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let profile = std::env::var("REV_ACCEPTANCE_PROFILE")?;
    let config = configure(root, &profile)?;
    let mut terminal = start(root, binary, wrapper)?;
    cancel(&mut terminal, &config, &profile)?;
    acquire(&mut terminal, &profile)?;
    ready(root, &config)?;
    inventory(&mut terminal, &profile)?;
    finish(&mut terminal, root, &config, &profile)
}

fn command(terminal: &mut Terminal, name: &str, profile: &str) -> Result<()> {
    let prefix = if profile == "modal" { ":" } else { "\x1bx" };
    terminal.send(format!("{prefix}{name}\r").as_bytes())
}

fn configure(root: &Path, profile: &str) -> Result<std::path::PathBuf> {
    fs::create_dir_all(root.join("workspace"))?;
    fs::create_dir_all(root.join("pty/config/rev"))?;
    let config = root.join("pty/config/rev/config.toml");
    fs::write(
        &config,
        format!(
            "# retain terminal root\n[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[completion]\nautomatic=false\n"
        ),
    )?;
    let file = root.join("workspace/terminal.rs");
    let original = "fn terminal_native() {}\n";
    fs::write(&file, original)?;
    Ok(config)
}

fn cancel(terminal: &mut Terminal, config: &Path, profile: &str) -> Result<()> {
    let initial = fs::read(config)?;
    command(terminal, "lsp-setup", profile)?;
    terminal.frame(|f| f.contains("LSP language:"))?;
    terminal.send(b"\x07")?;
    terminal.frame(|f| f.contains("cancelled"))?;
    assert_eq!(fs::read(config)?, initial);
    Ok(())
}

fn acquire(terminal: &mut Terminal, profile: &str) -> Result<()> {
    command(terminal, "lsp-setup", profile)?;
    terminal.frame(|f| f.contains("LSP language:"))?;
    terminal.send(b"rust\r")?;
    terminal.frame(|f| f.contains("Server name"))?;
    terminal.send(b"install:rust-analyzer\r")?;
    terminal.frame(|f| f.contains("Rust server acceptance-local ready"))?;
    Ok(())
}

fn ready(root: &Path, config: &Path) -> Result<()> {
    let registry = Registry::load(&root.join("pty/data/rev"))?;
    assert_eq!(registry.installations.len(), 1);
    assert_eq!(registry.installations[0].status, State::Ready);
    let saved = rev::config::prepare(config);
    assert!(saved.diagnostic.is_none(), "{:?}", saved.diagnostic);
    assert!(!saved.snapshot.settings().lsp.enabled);
    assert!(
        saved
            .snapshot
            .settings()
            .lsp
            .selected("rust")
            .unwrap()
            .1
            .command
            .contains(&registry.installations[0].id)
    );
    Ok(())
}

fn finish(terminal: &mut Terminal, root: &Path, config: &Path, profile: &str) -> Result<()> {
    terminal.finish()?;
    assert_eq!(
        fs::read_to_string(root.join("workspace/terminal.rs"))?,
        "fn terminal_native() {}\n"
    );
    assert!(fs::read_to_string(config)?.contains("# retain terminal root"));
    println!("MEASURE PTY management bytes={}", terminal.bytes());
    println!(
        "PASS Linux management PTY profile={profile}: cancel/setup/verified install/inventory, disabled analysis, idle silence, source preserved and TTY restored"
    );
    Ok(())
}

fn start(root: &Path, binary: &Path, wrapper: &Path) -> Result<Terminal> {
    let mut terminal = Terminal::start(root, binary, wrapper)?;
    terminal.frame(|f| row(f, 1).contains("fn terminal_native"))?;
    terminal.idle()?;
    Ok(terminal)
}

fn inventory(terminal: &mut Terminal, profile: &str) -> Result<()> {
    command(terminal, "lsp-servers", profile)?;
    terminal.frame(|f| f.contains("acceptance-local") && f.contains("ready"))?;
    terminal.idle()
}
