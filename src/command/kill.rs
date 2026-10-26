use std::ops::Range;

use crate::coordinates::{Coordinates, Error, Scalar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillCommand {
    Line,
    Word,
    BackwardWord,
}

impl KillCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Line => "kill-line",
            Self::Word => "kill-word",
            Self::BackwardWord => "backward-kill-word",
        }
    }

    pub(crate) fn range(
        self,
        coordinates: &Coordinates<'_>,
        point: Scalar,
    ) -> Result<Range<usize>, Error> {
        match self {
            Self::Line => {
                let end = coordinates.line_end(point)?;
                let end = if point == end {
                    coordinates.next(point)?
                } else {
                    end
                };
                Ok(point.0..end.0)
            }
            Self::Word => Ok(point.0..coordinates.forward_word(point)?.0),
            Self::BackwardWord => Ok(coordinates.backward_word(point)?.0..point.0),
        }
    }
}
