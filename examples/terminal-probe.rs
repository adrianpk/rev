#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use crossterm::{
    QueueableCommand,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use rev::terminal::Terminal;

#[path = "terminal_probe/pointer.rs"]
mod pointer;
#[path = "terminal_probe/preview.rs"]
mod preview;
#[path = "terminal_probe/timer.rs"]
mod timer;

struct Fault {
    output: io::Stdout,
    patterns: Vec<&'static [u8]>,
}

impl Write for Fault {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Some(index) = self
            .patterns
            .iter()
            .position(|pattern| bytes.windows(pattern.len()).any(|part| part == *pattern))
        {
            self.patterns.remove(index);
            return Err(io::Error::other("injected output failure"));
        }
        self.output.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

fn ready(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    terminal.writer().write_all(b"probe-ready\r\n")?;
    terminal.writer().flush()?;
    io::stdin().read_exact(&mut [0])
}

fn pattern(mode: &str) -> Option<&'static [u8]> {
    match mode {
        "screen" => Some(b"\x1b[?25l"),
        "io" => Some(b"probe-io"),
        "paste" => Some(b"\x1b[?2004h"),
        "paste-stop" => Some(b"\x1b[?2004l"),
        "cursor-stop" => Some(b"\x1b[?25h"),
        "attributes-stop" => Some(b"\x1b[0m"),
        "color-set" => Some(b"\x1b]12;"),
        "color-stop" => Some(b"\x1b]112"),
        "screen-stop" => Some(b"\x1b[?1049l"),
        _ => preview::pattern(mode).or_else(|| timer::pattern(mode)),
    }
}

fn patterns(mode: &str) -> Vec<&'static [u8]> {
    if mode == "restore" {
        return vec![
            b"\x1b[?2004l",
            b"\x1b[?25h",
            b"\x1b[0m",
            b"\x1b]112",
            b"\x1b[?1049l",
        ];
    }
    pattern(mode).into_iter().collect()
}

fn probe(mode: &str) -> io::Result<()> {
    let output = Fault {
        output: io::stdout(),
        patterns: patterns(mode),
    };
    let mut terminal = Terminal::acquire(output)?;
    if ["raw", "screen", "paste"].contains(&mode) {
        ready(&mut terminal)?;
    }
    if mode == "raw" {
        return Err(io::Error::other("injected raw-stage failure"));
    }
    terminal.enter()?;
    if mode == "color-set" {
        ready(&mut terminal)?;
    }
    palette(&mut terminal)?;
    ready(&mut terminal)?;
    finish(&mut terminal, mode)
}

fn palette(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    terminal.set_cursor_color(rev::config::Rgb(1, 35, 69))?;
    terminal
        .writer()
        .queue(SetForegroundColor(Color::Rgb { r: 1, g: 35, b: 69 }))?
        .queue(SetBackgroundColor(Color::Rgb {
            r: 67,
            g: 89,
            b: 101,
        }))?;
    Ok(())
}

fn finish(terminal: &mut Terminal<Fault>, mode: &str) -> io::Result<()> {
    match mode {
        "org-timer-io" => timer::failure(terminal),
        "org-preview-io" | "org-preview-unwind" => preview::failure(terminal, mode),
        "io" => {
            terminal.writer().write_all(b"probe-io")?;
            terminal.writer().flush()
        }
        "unwind" => panic!("injected test unwind"),
        "restore" => restore_probe(terminal),
        mode if restores(mode) => terminal.restore(),
        _ => Err(io::Error::other("injected session failure")),
    }
}

fn restores(mode: &str) -> bool {
    [
        "paste-stop",
        "cursor-stop",
        "attributes-stop",
        "screen-stop",
        "color-stop",
        "normal",
    ]
    .contains(&mode)
}

fn restore_probe(terminal: &mut Terminal<Fault>) -> io::Result<()> {
    let result = terminal.restore();
    // A second PTY acknowledgment lets the harness inspect raw-mode release
    // before Drop retries the still-owned output restoration obligations.
    eprintln!("probe-cleanup-pending");
    io::stdin().read_exact(&mut [0])?;
    result
}

fn main() -> ExitCode {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let result = if mode.starts_with("mouse-") {
        pointer::probe(&mode)
    } else {
        probe(&mode)
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("probe: {error}");
            ExitCode::FAILURE
        }
    }
}
