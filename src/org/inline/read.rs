use std::ops::Range;
use std::time::{Duration, Instant};

use crate::org::{Cancellation, Coverage};
use ropey::Rope;

pub(super) struct Stop(pub Coverage);

pub(super) struct Read<'a> {
    text: &'a Rope,
    cancellation: &'a Cancellation,
    yielded: Instant,
    pub start: usize,
    pub end: usize,
    pub depth: usize,
    pub max_depth: usize,
    pub failed_until: [usize; 8],
}

impl<'a> Read<'a> {
    pub fn new(
        text: &'a Rope,
        range: Range<usize>,
        cancellation: &'a Cancellation,
        max_depth: usize,
    ) -> Self {
        Self {
            text,
            cancellation,
            yielded: Instant::now(),
            start: range.start,
            end: range.end,
            depth: 0,
            max_depth,
            failed_until: [0; 8],
        }
    }

    pub fn tick(&mut self) -> Result<(), Stop> {
        if self.cancellation.is_cancelled() {
            return Err(Stop(Coverage::Cancelled));
        }
        if self.yielded.elapsed() >= Duration::from_millis(10) {
            std::thread::yield_now();
            self.yielded = Instant::now();
        }
        Ok(())
    }

    pub fn byte(&mut self, position: usize) -> Result<Option<u8>, Stop> {
        self.tick()?;
        Ok((position < self.end).then(|| self.text.byte(position)))
    }

    pub fn starts(&mut self, position: usize, pattern: &[u8]) -> Result<bool, Stop> {
        for (offset, byte) in pattern.iter().enumerate() {
            if self.byte(position + offset)? != Some(*byte) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn character(&mut self, position: usize) -> Result<Option<char>, Stop> {
        self.tick()?;
        if position >= self.end {
            return Ok(None);
        }
        Ok(self.text.get_char(self.text.byte_to_char(position)))
    }

    pub fn previous(&mut self, position: usize) -> Result<Option<char>, Stop> {
        self.tick()?;
        if position <= self.start {
            return Ok(None);
        }
        Ok(self.text.get_char(self.text.byte_to_char(position) - 1))
    }

    pub fn equals_ascii_case(&mut self, range: Range<usize>, word: &str) -> Result<bool, Stop> {
        self.tick()?;
        if range.len() != word.len() {
            return Ok(false);
        }
        for (offset, expected) in word.bytes().enumerate() {
            if self
                .byte(range.start + offset)?
                .is_none_or(|byte| !byte.eq_ignore_ascii_case(&expected))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn find(&mut self, mut position: usize, byte: u8) -> Result<Option<usize>, Stop> {
        while let Some(current) = self.byte(position)? {
            if current == byte {
                return Ok(Some(position));
            }
            position += 1;
        }
        Ok(None)
    }
}
