use std::io::{self, Read, Write};

use rev::terminal::Terminal;

use super::Fault;

pub(super) fn patterns(mode: &str) -> Vec<&'static [u8]> {
    match mode {
        "mouse-format" => vec![b"\x1b[?1006h"],
        "mouse-buttons" => vec![b"\x1b[?1002h"],
        "mouse-format-stop" => vec![b"\x1b[?1006l"],
        "mouse-buttons-stop" => vec![b"\x1b[?1002l"],
        "mouse-restore" => vec![b"\x1b[?1002l", b"\x1b[?1006l"],
        "mouse-rollback" => vec![b"\x1b[?1002h", b"\x1b[?1002l", b"\x1b[?1006l"],
        "mouse-io" => vec![b"probe-io"],
        _ => Vec::new(),
    }
}

pub(super) fn probe(mode: &str) -> io::Result<()> {
    let output = Fault {
        output: io::stdout(),
        patterns: patterns(mode),
    };
    let mut terminal = Terminal::acquire(output)?;
    if mode == "mouse-before-entry" {
        super::ready(&mut terminal)?;
        return terminal.set_mouse_reporting(true);
    }
    terminal.enter()?;
    terminal.set_mouse_reporting(true)?;
    terminal.set_mouse_reporting(true)?;
    super::ready(&mut terminal)?;
    finish(&mut terminal, mode)
}

fn finish(terminal: &mut Terminal<Fault>, mode: &str) -> io::Result<()> {
    match mode {
        "mouse-unwind" => panic!("injected pointer unwind"),
        "mouse-io" => {
            terminal.writer().write_all(b"probe-io")?;
            terminal.writer().flush()
        }
        "mouse-toggle" => toggle(terminal),
        "mouse-restore" => super::restore_probe(terminal),
        _ => {
            terminal.restore()?;
            terminal.restore()
        }
    }
}

fn toggle(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    terminal.set_mouse_reporting(false)?;
    terminal.set_mouse_reporting(false)?;
    checkpoint(terminal, b"probe-mouse-off\r\n")?;
    terminal.set_mouse_reporting(true)?;
    terminal.set_mouse_reporting(true)?;
    checkpoint(terminal, b"probe-mouse-on\r\n")?;
    terminal.restore()?;
    terminal.restore()
}

fn checkpoint(terminal: &mut Terminal<Fault>, text: &[u8]) -> io::Result<()> {
    terminal.writer().write_all(text)?;
    terminal.writer().flush()?;
    io::stdin().read_exact(&mut [0])
}
