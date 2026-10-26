use ropey::RopeSlice;

use super::{Byte, Coordinates, Error, Scalar, cursor::Cursor};

impl Coordinates<'_> {
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
