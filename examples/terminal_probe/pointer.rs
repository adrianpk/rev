use std::io::{self, Read, Write};

use rev::terminal::Terminal;
use rev::{
    command::Command,
    input::{Input, Outcome},
};

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
        "mouse-input" => input(terminal),
        "mouse-restore" => super::restore_probe(terminal),
        _ => {
            terminal.restore()?;
            terminal.restore()
        }
    }
}

fn input(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    terminal.writer().write_all(b"probe-input-ready\r\n")?;
    terminal.writer().flush()?;
    let mut input = Input::default();
    for _ in 0..32 {
        let outcome = input.handle(crossterm::event::read()?);
        record(terminal, &outcome)?;
        if outcome == Outcome::Command(Command::Quit) {
            return terminal.restore();
        }
    }
    Err(io::Error::other("pointer probe exceeded 32 native events"))
}

fn record(terminal: &mut Terminal<Fault>, outcome: &Outcome) -> io::Result<()> {
    let output = terminal.writer();
    match outcome {
        Outcome::Pointer(event) => writeln!(
            output,
            "pointer {:?} {} {} {}\r",
            event.kind,
            event.position.column,
            event.position.row,
            event.modifiers.bits()
        ),
        Outcome::Ignored => writeln!(output, "ignored\r"),
        Outcome::Prefix(prefix) => writeln!(output, "prefix {}\r", prefix.label()),
        Outcome::Command(command) => writeln!(output, "command {}\r", command.name()),
        Outcome::Editing(_) => writeln!(output, "keyboard\r"),
        _ => Err(io::Error::other("unexpected pointer probe outcome")),
    }?;
    output.flush()
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
