use crate::view::Size;

use super::{Area, Geometry, Split, ViewId};

/// Leaves name live editing views; internal nodes record screen effects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    Leaf(ViewId),
    Split {
        direction: Split,
        first: Box<Layout>,
        second: Box<Layout>,
    },
}

impl Layout {
    pub(super) fn geometry(&self, size: Size) -> Option<Geometry> {
        let mut geometry = Geometry::default();
        self.place(Area::root(size)?, &mut geometry)?;
        Some(geometry)
    }
    fn place(&self, area: Area, geometry: &mut Geometry) -> Option<()> {
        match self {
            Self::Leaf(id) => geometry.panes.push((*id, area)),
            Self::Split {
                direction,
                first,
                second,
            } => {
                let division = area.divide(*direction)?;
                first.place(division.first, geometry)?;
                second.place(division.second, geometry)?;
                if let Some(border) = division.border {
                    geometry.borders.push(border);
                }
            }
        }
        Some(())
    }
    pub(super) fn split(&mut self, selected: ViewId, new: ViewId, direction: Split) -> bool {
        match self {
            Self::Leaf(id) if *id == selected => {
                *self = Self::Split {
                    direction,
                    first: Box::new(Self::Leaf(selected)),
                    second: Box::new(Self::Leaf(new)),
                };
                true
            }
            Self::Split { first, second, .. } => {
                first.split(selected, new, direction) || second.split(selected, new, direction)
            }
            _ => false,
        }
    }
}
