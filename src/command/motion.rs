use crate::coordinates::{Coordinates, Error, Scalar};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    ForwardChar,
    BackwardChar,
    LineStart,
    LineEnd,
    BufferStart,
    BufferEnd,
    ForwardWord,
    BackwardWord,
}

impl Motion {
    pub fn name(self) -> &'static str {
        match self {
            Self::ForwardChar => "forward-char",
            Self::BackwardChar => "backward-char",
            Self::LineStart => "move-beginning-of-line",
            Self::LineEnd => "move-end-of-line",
            Self::BufferStart => "beginning-of-buffer",
            Self::BufferEnd => "end-of-buffer",
            Self::ForwardWord => "forward-word",
            Self::BackwardWord => "backward-word",
        }
    }

    pub(crate) fn target(
        self,
        coordinates: &Coordinates<'_>,
        point: Scalar,
    ) -> Result<Scalar, Error> {
        match self {
            Self::ForwardChar => coordinates.next(point),
            Self::BackwardChar => coordinates.previous(point),
            Self::LineStart => coordinates.line_start(point),
            Self::LineEnd => coordinates.line_end(point),
            Self::BufferStart => Ok(Scalar(0)),
            Self::BufferEnd => Ok(coordinates.end()),
            Self::ForwardWord => coordinates.forward_word(point),
            Self::BackwardWord => coordinates.backward_word(point),
        }
    }
}
