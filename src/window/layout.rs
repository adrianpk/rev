use crate::view::Size;

use super::geometry::Minimum;
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
                let division = area.divide_to(*direction, first.minimum()?, second.minimum()?)?;
                first.place(division.first, geometry)?;
                second.place(division.second, geometry)?;
                if let Some(border) = division.border {
                    geometry.borders.push(border);
                }
            }
        }
        Some(())
    }
    fn minimum(&self) -> Option<Minimum> {
        match self {
            Self::Leaf(_) => Some(Minimum::leaf()),
            Self::Split {
                direction,
                first,
                second,
            } => Minimum::join(*direction, first.minimum()?, second.minimum()?),
        }
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
    pub(super) fn leaves(&self, output: &mut Vec<ViewId>) {
        match self {
            Self::Leaf(id) => output.push(*id),
            Self::Split { first, second, .. } => {
                first.leaves(output);
                second.leaves(output);
            }
        }
    }

    fn first_leaf(&self) -> ViewId {
        match self {
            Self::Leaf(id) => *id,
            Self::Split { first, .. } => first.first_leaf(),
        }
    }

    fn is_leaf(&self, id: ViewId) -> bool {
        matches!(self, Self::Leaf(current) if *current == id)
    }

    /// Reconnect the sibling subtree and select its first editing leaf. This
    /// operates on a private candidate, leaving the live layout untouched.
    pub(super) fn remove(&mut self, id: ViewId) -> Option<ViewId> {
        let Self::Split { first, second, .. } = self else {
            return None;
        };
        if first.is_leaf(id) {
            let replacement = second.first_leaf();
            *self = *second.clone();
            return Some(replacement);
        }
        if second.is_leaf(id) {
            let replacement = first.first_leaf();
            *self = *first.clone();
            return Some(replacement);
        }
        first.remove(id).or_else(|| second.remove(id))
    }
}
