use std::collections::VecDeque;

use ropey::RopeSlice;

use super::{Limit, MAX_BYTES, SLICE_TOKENS};

pub(super) enum Pause {
    Yield,
    Limit(Limit),
}

/// A finite lookahead window. Each newly inspected scalar is charged once,
/// including token prefixes; oversized literals cannot bypass byte accounting.
pub(super) struct Reader {
    window: VecDeque<char>,
    read: usize,
    eof_observed: bool,
    pub at: usize,
    pub bytes: usize,
    pub previous: Option<char>,
    pub line_start: bool,
    pub slice_reads: usize,
}

impl Default for Reader {
    fn default() -> Self {
        Self {
            window: VecDeque::with_capacity(16),
            read: 0,
            eof_observed: false,
            at: 0,
            bytes: 0,
            previous: None,
            line_start: true,
            slice_reads: 0,
        }
    }
}

impl Reader {
    pub fn new_budget(&mut self) {
        self.bytes = 0;
        self.slice_reads = 0;
    }

    pub fn changed_suffix(&mut self) {
        self.window.clear();
        self.read = self.at;
        self.eof_observed = false;
    }

    pub fn horizon(&self) -> usize {
        self.read.saturating_add(usize::from(self.eof_observed))
    }

    pub fn rewind(&mut self, position: (usize, Option<char>, bool)) {
        self.at = position.0;
        self.previous = position.1;
        self.line_start = position.2;
        self.changed_suffix();
    }

    pub fn peek(&mut self, text: RopeSlice<'_>, offset: usize) -> Result<Option<char>, Pause> {
        assert!(offset < 16, "pair lookahead remains finite");
        while self.window.len() <= offset && self.read < text.len_chars() {
            self.load(text)?;
        }
        let value = self.window.get(offset).copied();
        self.eof_observed |= value.is_none();
        Ok(value)
    }

    fn load(&mut self, text: RopeSlice<'_>) -> Result<(), Pause> {
        if self.slice_reads == SLICE_TOKENS {
            return Err(Pause::Yield);
        }
        let available = (16 - self.window.len())
            .min(SLICE_TOKENS - self.slice_reads)
            .min(text.len_chars() - self.read);
        let before = self.read;
        let mut characters = text.chars_at(self.read);
        for _ in 0..available {
            if !self.fits_next(text) {
                break;
            }
            let c = characters.next().expect("admitted pair scalar");
            self.window.push_back(c);
            self.bytes += c.len_utf8();
            self.read += 1;
            self.slice_reads += 1;
        }
        if self.read == before {
            Err(Pause::Limit(Limit::Bytes))
        } else {
            Ok(())
        }
    }

    fn fits_next(&self, text: RopeSlice<'_>) -> bool {
        let remaining = MAX_BYTES - self.bytes;
        remaining >= 4
            || text.char_to_byte(self.read + 1) - text.char_to_byte(self.read) <= remaining
    }

    pub fn prefix(&mut self, text: RopeSlice<'_>, prefix: &str) -> Result<bool, Pause> {
        for (offset, expected) in prefix.chars().enumerate() {
            if self
                .peek(text, offset)?
                .is_none_or(|c| !c.eq_ignore_ascii_case(&expected))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn consume(&mut self, count: usize) {
        for _ in 0..count {
            let c = self.window.pop_front().expect("inspected pair source");
            self.previous = Some(c);
            self.line_start = c == '\n' || c == '\r';
            self.at += 1;
        }
    }

    pub fn retained(&self) -> usize {
        self.window.capacity() * std::mem::size_of::<char>()
    }
}
