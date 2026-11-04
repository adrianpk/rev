use super::support::*;
use rev::command::TerminalCommand;
use std::{
    cell::Cell,
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
    rc::Rc,
};

struct FailOnce {
    failed: Rc<Cell<bool>>,
    output: io::Stdout,
}
impl Write for FailOnce {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failed.replace(false) {
            return Err(io::Error::other("owned fixture write failure"));
        }
        self.output.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
// Inject one host write failure inside a real controlling PTY. All subsequent
// writes are real: this proves restoration/cleanup orchestration after that
// boundary fault, not recovery from an unavailable physical terminal.
pub(crate) fn host(root: &Path) -> Result<()> {
    let mut editor = Editor::open(&root.join("host-error"), "non-modal")?;
    editor
        .session
        .execute(rev::command::Command::Terminal(TerminalCommand::Open));
    editor.wait(|s| screen(s).contains("SHELL_READY>"))?;
    let id = editor.selected();
    let pid = editor
        .session
        .terminals()
        .get(id)
        .unwrap()
        .process_id()
        .unwrap();
    let failed = Rc::new(Cell::new(false));
    let writer = FailOnce {
        failed: failed.clone(),
        output: io::stdout(),
    };
    let mut terminal = rev::terminal::Terminal::acquire(writer)?;
    terminal.enter()?;
    failed.set(true);
    assert!(editor.session.run_terminal(&mut terminal).is_err());
    assert_eq!(editor.session.terminals().ids().count(), 0);
    assert!(!fs::exists(format!("/proc/{pid}"))?);
    assert_eq!(source(&editor.session), ORIGINAL);
    println!("HOST_ERROR_CLEANED");
    Ok(())
}
pub(crate) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let output = Command::new("script")
        .args([
            "-q",
            "-e",
            "-f",
            "-E",
            "never",
            "-o",
            "1048576",
            "/dev/null",
            "--",
            "bash",
        ])
        .arg(wrapper)
        .arg("--error-child")
        .arg(
            binary
                .parent()
                .unwrap()
                .join("examples/integrated-terminal-acceptance"),
        )
        .arg(root)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let text = String::from_utf8(output.stdout)?;
    for marker in [
        "HOST_ERROR_CLEANED",
        "REV_TTY_RESTORED",
        "\x1b[?1049l",
        "\x1b[?2004l",
        "\x1b[?25h",
    ] {
        assert!(text.contains(marker), "missing restoration: {marker:?}");
    }
    println!(
        "PASS host error: one injected output fault, real PTY attribute/mode restoration, owned child reaped and source preserved"
    );
    Ok(())
}
