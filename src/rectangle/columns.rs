use crate::coordinates::{Cell, Coordinates, Error, Scalar};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Columns {
    Display,
    Grapheme,
}

impl Columns {
    pub(super) fn cell(self, coordinates: &Coordinates<'_>, point: Scalar) -> Result<Cell, Error> {
        match self {
            Self::Display => coordinates.to_cell(point),
            Self::Grapheme => coordinates.grapheme_cell(point),
        }
    }

    pub(super) fn position(
        self,
        coordinates: &Coordinates<'_>,
        cell: Cell,
    ) -> Result<Scalar, Error> {
        match self {
            Self::Display => coordinates.from_cell(cell),
            Self::Grapheme => coordinates.from_grapheme(cell),
        }
    }
}
