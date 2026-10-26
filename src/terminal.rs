use std::io::{self, BufWriter, Write};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::style::{Attribute, SetAttribute};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{Command, ExecutableCommand};

/// Owns terminal restoration, including partially completed acquisition.
pub struct Terminal<W: Write> {
    output: BufWriter<W>,
    raw: bool,
    screen: bool,
    cursor: bool,
    attributes: bool,
    paste: bool,
}

impl<W: Write> Terminal<W> {
    /// Acquires raw input. Dropping this owner restores it even before entry.
    pub fn acquire(output: W) -> io::Result<Self> {
        enable_raw_mode().map_err(|error| context(error, "enable raw mode"))?;
        Ok(Self {
            output: BufWriter::new(output),
            raw: true,
            screen: false,
            cursor: false,
            attributes: false,
            paste: false,
        })
    }

    /// Enters the alternate screen and hides the hardware cursor.
    pub fn enter(&mut self) -> io::Result<()> {
        // A failed write may already have changed the terminal; retain its undo.
        self.screen = true;
        self.attributes = true;
        self.command(EnterAlternateScreen, "enter alternate screen")?;
        self.cursor = true;
        self.command(Hide, "hide cursor")?;
        self.paste = true;
        self.command(EnableBracketedPaste, "enable bracketed paste")
    }

    /// Borrows buffered output; the caller flushes at render boundaries.
    pub fn writer(&mut self) -> &mut BufWriter<W> {
        &mut self.output
    }

    /// Attempts every acquired restoration, even if an earlier one fails.
    /// Failed obligations remain owned so drop can retry them.
    pub fn restore(&mut self) -> io::Result<()> {
        let paste = self.disable_paste();
        let cursor = self.show();
        let attributes = self.reset_attributes();
        let screen = self.leave();
        let raw = self.release();
        paste.and(cursor).and(attributes).and(screen).and(raw)
    }

    fn command(&mut self, command: impl Command, operation: &str) -> io::Result<()> {
        self.output
            .execute(command)
            .map(|_| ())
            .map_err(|error| context(error, operation))
    }

    fn show(&mut self) -> io::Result<()> {
        if self.cursor {
            self.command(Show, "show cursor")?;
            self.cursor = false;
        }
        Ok(())
    }

    fn reset_attributes(&mut self) -> io::Result<()> {
        if self.attributes {
            self.command(SetAttribute(Attribute::Reset), "reset display attributes")?;
            self.attributes = false;
        }
        Ok(())
    }

    fn leave(&mut self) -> io::Result<()> {
        if self.screen {
            self.command(LeaveAlternateScreen, "leave alternate screen")?;
            self.screen = false;
        }
        Ok(())
    }

    fn release(&mut self) -> io::Result<()> {
        if self.raw {
            disable_raw_mode().map_err(|error| context(error, "disable raw mode"))?;
            self.raw = false;
        }
        Ok(())
    }

    fn disable_paste(&mut self) -> io::Result<()> {
        if self.paste {
            self.command(DisableBracketedPaste, "disable bracketed paste")?;
            self.paste = false;
        }
        Ok(())
    }
}

impl<W: Write> Drop for Terminal<W> {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            eprintln!("rev: terminal restoration: {error}");
        }
    }
}

fn context(error: io::Error, operation: &str) -> io::Error {
    io::Error::new(error.kind(), format!("{operation}: {error}"))
}
