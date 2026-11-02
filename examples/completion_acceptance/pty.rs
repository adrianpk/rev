use super::support::Result;
pub(crate) use crate::terminal::{Terminal, row};
use std::path::Path;

pub(super) fn run(root: &Path, binary: &Path, wrapper: &Path) -> Result<()> {
    let original = "fn configuration() {}\nfn config_value() {}\ncon\n";
    let path = root.join("workspace/terminal.rs");
    std::fs::write(&path, original)?;
    let mut terminal = Terminal::start(root, binary, wrapper)?;
    manual_cancel(&mut terminal)?;
    manual_accept(&mut terminal)?;
    automatic(&mut terminal)?;
    terminal.idle()?;
    terminal.finish()?;
    assert_eq!(std::fs::read_to_string(path)?, original);
    println!(
        "PASS Linux PTY completion: {} output bytes; manual selection/cancel/accept, automatic debounce, literal one-step undo, source unchanged and TTY restored",
        terminal.bytes()
    );
    Ok(())
}

fn manual_cancel(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bxgoto-line\r3\r\x05\x1bxcompletion-at-point\r")?;
    terminal.frame(popup)?;
    terminal.send(b"\x07")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}
fn manual_accept(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"\x1bxcompletion-at-point\r")?;
    terminal.frame(popup)?;
    terminal.send(b"\x0e\r")?;
    terminal.frame(|frame| source_is(frame, "configuration", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}
fn automatic(terminal: &mut Terminal) -> Result<()> {
    terminal.send(b"f")?;
    terminal.frame(popup)?;
    terminal.send(b"\t")?;
    terminal.frame(|frame| source_is(frame, "config_value", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "conf", true))?;
    terminal.send(b"\x18u")?;
    terminal.frame(|frame| source_is(frame, "con", false))?;
    Ok(())
}

fn popup(frame: &str) -> bool {
    frame.contains('┌') && frame.contains("config_value") && frame.contains("configuration")
}
fn source_is(frame: &str, source: &str, modified: bool) -> bool {
    !frame.contains('┌')
        && row(frame, 3).contains(source)
        && frame.contains(if modified {
            "terminal.rs ** RW"
        } else {
            "terminal.rs -- RW"
        })
}
