use std::io::{self, Write};

/// Independent undo obligations survive partial writes and failed restoration.
#[derive(Default)]
pub(super) struct Reporting {
    sgr: bool,
    buttons: bool,
}

impl Reporting {
    pub(super) fn set(&mut self, output: &mut impl Write, enabled: bool) -> io::Result<()> {
        if !enabled {
            return self.restore(output);
        }
        let result = self.enable(output);
        match result {
            Ok(()) => Ok(()),
            Err(error) => super::restoration([Err(error), self.restore(output)]),
        }
    }

    fn enable(&mut self, output: &mut impl Write) -> io::Result<()> {
        if !self.sgr {
            self.sgr = true;
            emit(output, b"\x1b[?1006h", "enable SGR mouse coordinates")?;
        }
        if !self.buttons {
            self.buttons = true;
            emit(output, b"\x1b[?1002h", "enable mouse button tracking")?;
        }
        Ok(())
    }

    fn restore(&mut self, output: &mut impl Write) -> io::Result<()> {
        super::restoration([self.disable_buttons(output), self.disable_sgr(output)])
    }

    fn disable_buttons(&mut self, output: &mut impl Write) -> io::Result<()> {
        if self.buttons {
            emit(output, b"\x1b[?1002l", "disable mouse button tracking")?;
            self.buttons = false;
        }
        Ok(())
    }

    fn disable_sgr(&mut self, output: &mut impl Write) -> io::Result<()> {
        if self.sgr {
            emit(output, b"\x1b[?1006l", "disable SGR mouse coordinates")?;
            self.sgr = false;
        }
        Ok(())
    }
}

fn emit(output: &mut impl Write, bytes: &[u8], operation: &str) -> io::Result<()> {
    output
        .write_all(bytes)
        .and_then(|()| output.flush())
        .map_err(|error| super::context(error, operation))
}

#[cfg(test)]
mod tests;
