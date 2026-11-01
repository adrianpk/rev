use crate::buffer::Buffer;
use crate::coordinates::Scalar;

pub const CONTEXT_CHARS: usize = 64;

/// Bounded source evidence, independent of transient buffer revision identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    pub position: usize,
    pub before: String,
    pub after: String,
    pub at_start: bool,
    pub at_end: bool,
}

impl Context {
    pub(crate) fn capture(buffer: &Buffer, position: Scalar) -> Result<Self, crate::buffer::Error> {
        Self::capture_with(buffer, position, CONTEXT_CHARS)
    }
    pub(crate) fn capture_with(
        buffer: &Buffer,
        position: Scalar,
        limit: usize,
    ) -> Result<Self, crate::buffer::Error> {
        let start = position.0.saturating_sub(limit);
        let end = position.0.saturating_add(limit).min(buffer.len_chars());
        let before = buffer
            .span(start..position.0)
            .and_then(|span| buffer.slice(&span).map(|s| s.to_string()))?;
        let after = buffer
            .span(position.0..end)
            .and_then(|span| buffer.slice(&span).map(|s| s.to_string()))?;
        Ok(Self {
            position: position.0,
            before,
            after,
            at_start: start == 0,
            at_end: end == buffer.len_chars(),
        })
    }

    pub(crate) fn matches(&self, buffer: &Buffer, position: usize) -> Result<bool, String> {
        let before = self.before.chars().count();
        let after = self.after.chars().count();
        if !self.contains(position, buffer.len_chars()) {
            return Ok(false);
        }
        let prefix = buffer
            .span(position - before..position)
            .map_err(|e| e.to_string())?;
        let suffix = buffer
            .span(position..position + after)
            .map_err(|e| e.to_string())?;
        Ok(
            buffer.slice(&prefix).map_err(|e| e.to_string())? == self.before.as_str()
                && buffer.slice(&suffix).map_err(|e| e.to_string())? == self.after.as_str(),
        )
    }
    fn contains(&self, position: usize, len: usize) -> bool {
        let before = self.before.chars().count();
        let after = self.after.chars().count();
        !(position < before
            || position > len
            || after > len - position
            || self.at_start && position != before
            || self.at_end && position + after != len)
    }
}
