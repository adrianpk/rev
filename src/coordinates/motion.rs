use ropey::RopeSlice;

use super::{Byte, Coordinates, Error, Scalar, cursor::Cursor};

impl Coordinates<'_> {
    /// The native point owner has already admitted this grapheme boundary.
    /// Starting its suffix avoids arbitrary lookbehind on a long RI run.
    pub(crate) fn next_observed(&self, point: Scalar) -> Result<Scalar, Error> {
        let suffix = self.text.get_slice(point.0..).ok_or(Error::Scalar(point))?;
        let end = Cursor::new(suffix, 0)?.next()?.unwrap_or(0);
        Ok(Scalar(point.0 + suffix.byte_to_char(end)))
    }

    /// Resolve a predecessor from checked forward observations. Cold native
    /// queries yield with the last complete boundary retained for the next step.
    pub(crate) fn previous_observed(&self, point: Scalar) -> Result<Scalar, Error> {
        if point.0 == 0 {
            return Ok(point);
        }
        let row = self.to_line(point)?.row;
        let (_, start) = self.body(row)?;
        if point == start {
            let (line, start) = self.body(row - 1)?;
            return Ok(Scalar(start.0 + line.len_chars()));
        }
        let (mut last, mut column) = self
            .checkpoint(Scalar(point.0 - 1), row)
            .unwrap_or((start, 0));
        let mut glyphs = self.glyphs_at(row, last, column)?;
        let work = self.work();
        let mut count = 0;
        loop {
            if !Self::charge(&work) {
                self.pause(last, super::Cell { row, column });
                return Err(Error::Pending);
            }
            let Some(glyph) = glyphs.next() else {
                return Ok(last);
            };
            let glyph = glyph?;
            if glyph.source.end >= point {
                self.remember(
                    glyph.source.start,
                    super::Cell {
                        row,
                        column: glyph.columns.start,
                    },
                );
                return Ok(glyph.source.start);
            }
            last = glyph.source.end;
            column = glyph.columns.end;
            count += 1;
            if count % super::checkpoints::INTERVAL == 0 {
                self.remember(last, super::Cell { row, column });
            }
        }
    }

    pub fn end(&self) -> Scalar {
        Scalar(self.text.len_chars())
    }

    pub fn line_start(&self, point: Scalar) -> Result<Scalar, Error> {
        let row = self.to_line(point)?.row;
        Ok(self.body(row)?.1)
    }

    /// Stops before the whole LF/CRLF separator, retaining lone visible controls.
    pub fn line_end(&self, point: Scalar) -> Result<Scalar, Error> {
        let row = self.to_line(point)?.row;
        let (line, start) = self.body(row)?;
        Ok(Scalar(start.0 + line.len_chars()))
    }

    /// Words are runs of graphemes containing Unicode letters/numbers or `_`.
    /// Skip separators, then consume one run, without flattening rope text.
    pub fn forward_word(&self, point: Scalar) -> Result<Scalar, Error> {
        self.word(point, false)
    }

    /// Uses the same grapheme classification in reverse and clamps at BOF.
    pub fn backward_word(&self, point: Scalar) -> Result<Scalar, Error> {
        self.word(point, true)
    }

    fn word(&self, point: Scalar, backward: bool) -> Result<Scalar, Error> {
        if !self.is_boundary(point)? {
            return Err(Error::Cluster(point));
        }
        let byte = self.to_byte(point)?.0;
        self.from_byte(Byte(scan(self.text, byte, backward)?))
    }
}

fn scan(text: RopeSlice<'_>, mut position: usize, backward: bool) -> Result<usize, Error> {
    let mut cursor = Cursor::new(text, position)?;
    let mut word = false;
    while let Some(next) = step(&mut cursor, backward)? {
        let range = position.min(next)..position.max(next);
        let cluster = text.get_byte_slice(range).ok_or(Error::Segmentation)?;
        let current = cluster.chars().any(word_character);
        if word && !current {
            break;
        }
        word |= current;
        position = next;
    }
    Ok(position)
}

fn step(cursor: &mut Cursor<'_>, backward: bool) -> Result<Option<usize>, Error> {
    if backward {
        cursor.previous()
    } else {
        cursor.next()
    }
}

fn word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}
