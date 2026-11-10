use std::io::{self, Write};

/// Independent undo obligations survive partial writes and failed restoration.
#[derive(Default)]
pub(super) struct Reporting {
    active: bool,
    sgr: bool,
    buttons: bool,
    motion: bool,
}

impl Reporting {
    pub(super) fn set(&mut self, output: &mut impl Write, enabled: bool) -> io::Result<()> {
        if !enabled {
            return self.restore(output);
        }
        if self.active {
            return Ok(());
        }
        // Failed undo flags describe uncertain effects, not confirmed modes.
        self.restore(output)?;
        let result = self.enable(output);
        match result {
            Ok(()) => {
                self.active = true;
                Ok(())
            }
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
        self.active = false;
        super::restoration([
            self.disable_motion(output),
            self.disable_buttons(output),
            self.disable_sgr(output),
        ])
    }

    pub(super) fn motion(&mut self, output: &mut impl Write, enabled: bool) -> io::Result<()> {
        if enabled {
            if !self.active {
                return Err(io::Error::other("motion requires owned mouse reporting"));
            }
            if !self.motion {
                self.motion = true;
                emit(output, b"\x1b[?1003h", "enable mouse motion tracking")?;
            }
        } else if self.motion {
            self.disable_motion(output)?;
            if self.active {
                emit(output, b"\x1b[?1002h", "restore mouse button tracking")?;
            }
        }
        Ok(())
    }

    fn disable_motion(&mut self, output: &mut impl Write) -> io::Result<()> {
        if self.motion {
            emit(output, b"\x1b[?1003l", "disable mouse motion tracking")?;
            self.motion = false;
        }
        Ok(())
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
        .map_err(|error| super::context(&error, operation))
}

#[cfg(test)]
mod tests;
