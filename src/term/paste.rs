use std::io;

use alacritty_terminal::{
    term::TermMode,
    vte::ansi::{ClearMode, Handler},
};

use super::{Session, SessionId, State};

/// Receipt for one queued paste. It grants no authority after any bytes leave
/// the queue, another input is accepted, or a protocol reply is appended.
#[derive(Clone, Copy)]
pub struct Paste {
    pub(super) id: SessionId,
    pub(super) serial: u64,
}

pub(super) struct Pending {
    serial: u64,
    start: usize,
}

impl Session {
    pub(super) fn paste(&mut self, text: &str, previous: Option<u64>) -> io::Result<u64> {
        if self.state != State::Running {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Terminal is not running",
            ));
        }
        let start = self.paste_start(previous)?;
        let bracketed = self.engine.term.mode().contains(TermMode::BRACKETED_PASTE);
        let available = self.limits.max_pending_input_bytes.saturating_sub(start);
        let bytes = encode(text, bracketed, available)?;
        let serial = self
            .input_serial
            .checked_add(1)
            .ok_or_else(|| io::Error::other("Terminal input identity exhausted"))?;
        self.input.truncate(start);
        self.input.extend(bytes);
        self.paste = Some(Pending { serial, start });
        self.input_serial = serial;
        self.rearm();
        Ok(serial)
    }
    fn paste_start(&self, previous: Option<u64>) -> io::Result<usize> {
        match previous {
            None => Ok(self.input.len()),
            Some(serial) => self
                .paste
                .as_ref()
                .filter(|paste| paste.serial == serial)
                .map(|paste| paste.start)
                .ok_or_else(|| {
                    io::Error::other(
                        "Previous paste is no longer wholly queued; cannot replace child input",
                    )
                }),
        }
    }
    pub(super) fn clear_scrollback(&mut self) {
        self.engine.term.clear_screen(ClearMode::Saved);
    }
}

// Bound before allocation. Explicit text may contain line breaks and tabs, but
// no escape/control that could end paste framing or claim command authority.
fn encode(text: &str, bracketed: bool, available: usize) -> io::Result<Vec<u8>> {
    let framing = if bracketed { 12 } else { 0 };
    if text.len().saturating_add(framing) > available {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "Terminal paste exceeds the input queue limit; nothing accepted",
        ));
    }
    if text.is_empty() || text.chars().any(rejected) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Terminal paste requires nonempty text without escape/control characters",
        ));
    }
    let mut bytes = Vec::with_capacity(text.len() + framing);
    if bracketed {
        bytes.extend_from_slice(b"\x1b[200~");
    }
    bytes.extend_from_slice(text.as_bytes());
    if bracketed {
        bytes.extend_from_slice(b"\x1b[201~");
    }
    Ok(bytes)
}
fn rejected(character: char) -> bool {
    character.is_control() && !matches!(character, '\n' | '\r' | '\t')
}
