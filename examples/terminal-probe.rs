#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use rev::terminal::Terminal;

struct Fault {
    output: io::Stdout,
    pattern: Option<&'static [u8]>,
}

impl Write for Fault {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if let Some(pattern) = self.pattern
            && bytes.windows(pattern.len()).any(|part| part == pattern)
        {
            self.pattern = None;
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
        "screen-stop" => Some(b"\x1b[?1049l"),
        _ => None,
    }
}

fn probe(mode: &str) -> io::Result<()> {
    let output = Fault {
        output: io::stdout(),
        pattern: pattern(mode),
    };
    let mut terminal = Terminal::acquire(output)?;
    if ["raw", "screen", "paste"].contains(&mode) {
        ready(&mut terminal)?;
    }
    if mode == "raw" {
        return Err(io::Error::other("injected raw-stage failure"));
    }
    terminal.enter()?;
    ready(&mut terminal)?;
    finish(&mut terminal, mode)
}

fn finish(terminal: &mut Terminal<Fault>, mode: &str) -> io::Result<()> {
    match mode {
        "io" => {
            terminal.writer().write_all(b"probe-io")?;
            terminal.writer().flush()
        }
        "unwind" => panic!("injected test unwind"),
        "paste-stop" | "cursor-stop" | "attributes-stop" | "screen-stop" => terminal.restore(),
        _ => Err(io::Error::other("injected session failure")),
    }
}

fn main() -> ExitCode {
    let mode = std::env::args().nth(1).unwrap_or_default();
    match probe(&mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("probe: {error}");
            ExitCode::FAILURE
        }
    }
}
