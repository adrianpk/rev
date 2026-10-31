use crate::buffer::Buffer;
use crate::coordinates::Scalar;

pub const CONTEXT_CHARS: usize = 64;
#[cfg(test)]
pub const SEARCH_CHARS: usize = 262144;
pub const SEARCH_BATCH: usize = 16384;

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
    #[cfg(test)]
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
        if position < before
            || position > buffer.len_chars()
            || after > buffer.len_chars() - position
            || self.at_start && position != before
            || self.at_end && position + after != buffer.len_chars()
        {
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
}

/// A streaming KMP scan. Each pass consumes at most SEARCH_BATCH source scalars;
/// context length does not multiply that work for each candidate position.
pub(crate) struct Search {
    context: Context,
    needle: Vec<char>,
    failure: Vec<usize>,
    matched: usize,
    cursor: usize,
    end: usize,
    before: usize,
    found: Option<usize>,
    scanned: usize,
}

pub(crate) enum Progress {
    Pending,
    Found(Scalar),
}

impl Search {
    pub(crate) fn new(context: Context, len: usize, limit: usize) -> Result<Self, String> {
        if limit == 0 {
            return Err("Bookmark context changed; relocation is disabled".into());
        }
        let needle: Vec<_> = context
            .before
            .chars()
            .chain(context.after.chars())
            .collect();
        if needle.is_empty() {
            return Err("Bookmark has no matching source evidence".into());
        }
        let mut failure = vec![0; needle.len()];
        let mut matched = 0;
        for index in 1..needle.len() {
            while matched > 0 && needle[index] != needle[matched] {
                matched = failure[matched - 1];
            }
            if needle[index] == needle[matched] {
                matched += 1;
            }
            failure[index] = matched;
        }
        let center = context.position.min(len);
        let start = center.saturating_sub(limit / 2);
        let end = start.saturating_add(limit).min(len);
        let start = end.saturating_sub(limit);
        let before = context.before.chars().count();
        Ok(Self {
            context,
            needle,
            failure,
            matched: 0,
            cursor: start,
            end,
            before,
            found: None,
            scanned: 0,
        })
    }

    pub(crate) fn step(&mut self, buffer: &Buffer) -> Result<Progress, String> {
        let end = self.cursor.saturating_add(SEARCH_BATCH).min(self.end);
        let span = buffer.span(self.cursor..end).map_err(|e| e.to_string())?;
        let source = buffer.slice(&span).map_err(|e| e.to_string())?;
        for character in source.chars() {
            self.cursor += 1;
            self.scanned += 1;
            while self.matched > 0 && character != self.needle[self.matched] {
                self.matched = self.failure[self.matched - 1];
            }
            if character == self.needle[self.matched] {
                self.matched += 1;
            }
            if self.matched == self.needle.len() {
                let position = self.cursor - self.needle.len() + self.before;
                if (!self.context.at_start || position == self.before)
                    && (!self.context.at_end || self.cursor == buffer.len_chars())
                {
                    if self.found.is_some() {
                        return Err("Bookmark context is ambiguous".into());
                    }
                    self.found = Some(position);
                }
                self.matched = self.failure[self.matched - 1];
            }
        }
        if self.cursor < self.end {
            return Ok(Progress::Pending);
        }
        self.found
            .map(|position| Progress::Found(Scalar(position)))
            .ok_or_else(|| "Bookmark context is stale or outside the search window".into())
    }

    #[cfg(test)]
    pub(crate) fn scanned(&self) -> usize {
        self.scanned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffers;

    // Streaming comparisons consume at most the per-pass and total scalar
    // limits, even across a long repeated prefix; a found result waits for the
    // completed search so a later duplicate cannot be silently ignored.
    #[test]
    fn bounded_search() {
        let original = format!("{}unique-target{}", "a".repeat(100000), "b".repeat(100000));
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create(&original).unwrap();
        let context = Context::capture(buffers.get(id).unwrap(), Scalar(100007)).unwrap();
        let candidate = format!("inserted\n{original}");
        let id = buffers.create(&candidate).unwrap();
        let buffer = buffers.get(id).unwrap();
        assert!(!context.matches(buffer, context.position).unwrap());
        let mut search = Search::new(context.clone(), buffer.len_chars(), SEARCH_CHARS).unwrap();
        let mut passes = 0;
        loop {
            let before = search.scanned();
            let result = search.step(buffer).unwrap();
            assert!(search.scanned() - before <= SEARCH_BATCH);
            assert!(search.scanned() <= SEARCH_CHARS);
            passes += 1;
            match result {
                Progress::Pending => assert!(passes < 20),
                Progress::Found(point) => {
                    assert_eq!(point.0, 100016);
                    break;
                }
            }
        }
        assert!(passes > 1);
        assert!(Search::new(context.clone(), buffer.len_chars(), 0).is_err());
        let id = buffers
            .create(&format!("{}{original}", "X".repeat(1000)))
            .unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut search = Search::new(context, buffer.len_chars(), 256).unwrap();
        assert!(search.step(buffer).is_err());
        assert_eq!(search.scanned(), 256);
    }
}
