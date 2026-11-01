use super::{Coordinates, Error, Scalar, cursor::Cursor};
use crate::command::navigation::Word;

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
            let Some(glyph) =
                self.charged_glyph(&mut glyphs, &work, (last, super::Cell { row, column }))?
            else {
                return Ok(last);
            };
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
            self.remember_periodic(count, last, super::Cell { row, column });
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
        self.words(
            point,
            if backward {
                Word::PreviousStart
            } else {
                Word::NextEnd
            },
            1,
        )
    }
}
