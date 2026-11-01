use crate::coordinates::{Coordinates, Error, Scalar};
use std::ops::Range;

impl Coordinates<'_> {
    pub(super) fn object_pair(
        &self,
        point: Scalar,
        policy: (char, bool),
        count: usize,
    ) -> Result<Option<Range<usize>>, Error> {
        let (open, around) = policy;
        let close = match open {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            _ => return Ok(None),
        };
        let mut at = if self.text.get_char(point.0) == Some(open) && !self.escaped(point.0) {
            Some(point.0)
        } else {
            self.enclosing_open(point.0, (open, close))
        };
        for _ in 1..count {
            let Some(before) = at else { return Ok(None) };
            at = self.enclosing_open(before, (open, close));
        }
        let Some(start) = at else { return Ok(None) };
        let Some(end) = self.closing_pair(start, (open, close)) else {
            return Ok(None);
        };
        self.delimited_range(start..end, around).map(Some)
    }

    fn enclosing_open(&self, before: usize, pair: (char, char)) -> Option<usize> {
        let mut depth = 0usize;
        let source = self.text.get_slice(..before)?;
        for (offset, c) in source.chars_at(source.len_chars()).reversed().enumerate() {
            let at = before - offset - 1;
            if !self.object_delimiter(at, c, pair) {
                continue;
            }
            if c == pair.0 {
                if depth == 0 {
                    return Some(at);
                }
                depth -= 1;
            } else if c == pair.1 {
                depth += 1;
            }
        }
        None
    }

    fn closing_pair(&self, start: usize, pair: (char, char)) -> Option<usize> {
        let mut depth = 1usize;
        for (offset, c) in self.text.slice(start + 1..).chars().enumerate() {
            let at = start + 1 + offset;
            if !self.object_delimiter(at, c, pair) {
                continue;
            }
            if c == pair.0 {
                depth += 1;
            }
            if c == pair.1 {
                depth -= 1;
            }
            if depth == 0 {
                return Some(at);
            }
        }
        None
    }

    pub(super) fn object_quote(
        &self,
        point: Scalar,
        policy: (char, bool),
        mut count: usize,
    ) -> Result<Option<Range<usize>>, Error> {
        let (quote, around) = policy;
        let start = self.line_start(point)?.0;
        let end = self.line_end(point)?.0;
        let mut opened = None;
        let mut selected = None;
        for (offset, c) in self.text.slice(start..end).chars().enumerate() {
            let at = start + offset;
            if c != quote || self.escaped(at) {
                continue;
            }
            match opened.take() {
                None => opened = Some(at),
                Some(begin) => {
                    if at < point.0 {
                        continue;
                    }
                    let first = *selected.get_or_insert(begin);
                    count -= 1;
                    if count == 0 {
                        return self.delimited_range(first..at, around).map(Some);
                    }
                }
            }
        }
        Ok(None)
    }

    fn object_delimiter(&self, at: usize, c: char, pair: (char, char)) -> bool {
        (c == pair.0 || c == pair.1) && !self.escaped(at)
    }

    fn escaped(&self, at: usize) -> bool {
        self.text
            .slice(..at)
            .chars_at(at)
            .reversed()
            .take_while(|c| *c == '\\')
            .count()
            % 2
            == 1
    }

    fn delimited_range(&self, range: Range<usize>, around: bool) -> Result<Range<usize>, Error> {
        let start = self.delimiter_start(Scalar(range.start))?;
        let end = self.delimiter_start(Scalar(range.end))?;
        if around {
            Ok(start.0..self.next(end)?.0)
        } else {
            Ok(self.next(start)?.0..end.0)
        }
    }

    fn delimiter_start(&self, point: Scalar) -> Result<Scalar, Error> {
        if self.is_boundary(point)? {
            Ok(point)
        } else {
            self.previous(point)
        }
    }
}
