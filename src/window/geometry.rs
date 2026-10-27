use crate::view::Size;

use super::ViewId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Below,
    Right,
}

/// Half-open terminal-cell bounds shared by pane and panel projections.
/// Construction rejects coordinate overflow; empty rectangles remain valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    column: u16,
    row: u16,
    width: u16,
    height: u16,
}

/// Minimum content+status extent; the global echo row belongs to the frame.
#[derive(Clone, Copy)]
pub(super) struct Minimum {
    width: u16,
    height: u16,
}

impl Minimum {
    pub(super) fn leaf() -> Self {
        Self {
            width: 3,
            height: 2,
        }
    }

    pub(super) fn join(split: Split, first: Self, second: Self, gap: u16) -> Option<Self> {
        match split {
            Split::Below => Some(Self {
                width: first.width.max(second.width),
                height: first.height.checked_add(gap)?.checked_add(second.height)?,
            }),
            Split::Right => Some(Self {
                width: first.width.checked_add(gap)?.checked_add(second.width)?,
                height: first.height.max(second.height),
            }),
        }
    }

    pub(super) fn size(self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

pub(super) struct Division {
    pub first: Area,
    pub second: Area,
    pub border: Option<Area>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Geometry {
    pub(super) panes: Vec<(ViewId, Area)>,
    pub(super) borders: Vec<Area>,
}

impl Geometry {
    pub fn panes(&self) -> &[(ViewId, Area)] {
        &self.panes
    }
    pub fn borders(&self) -> &[Area] {
        &self.borders
    }
}

impl Area {
    pub fn new(column: u16, row: u16, width: u16, height: u16) -> Option<Self> {
        column.checked_add(width)?;
        row.checked_add(height)?;
        Some(Self {
            column,
            row,
            width,
            height,
        })
    }

    pub fn column(self) -> u16 {
        self.column
    }
    pub fn row(self) -> u16 {
        self.row
    }
    pub fn width(self) -> u16 {
        self.width
    }
    pub fn height(self) -> u16 {
        self.height
    }

    pub(crate) fn with_status(self, reserve: bool) -> (Self, Option<Self>) {
        let height = u16::from(reserve && self.height > 0);
        let content = Self {
            height: self.height - height,
            ..self
        };
        let status = (height > 0).then_some(Self {
            row: self.row + content.height,
            height,
            ..self
        });
        (content, status)
    }
    /// The existing view projection reserves status plus echo. A pane supplies
    /// its status and borrows the global echo reservation for shared calculations.
    pub(crate) fn view_size(self) -> Size {
        Size {
            width: self.width,
            height: self.height + 1,
        }
    }

    pub(super) fn root(size: Size) -> Option<Self> {
        size.usable().then_some(Self {
            column: 0,
            row: 0,
            width: size.width,
            height: size.height.saturating_sub(1),
        })
    }
    pub(super) fn divide(self, split: Split) -> Option<Division> {
        self.divide_to(
            split,
            Minimum::leaf(),
            Minimum::leaf(),
            u16::from(split == Split::Right),
        )
    }

    pub(super) fn divide_to(
        self,
        split: Split,
        first: Minimum,
        second: Minimum,
        gap: u16,
    ) -> Option<Division> {
        match split {
            Split::Below => self.below(first, second, gap),
            Split::Right => self.right(first, second, gap),
        }
    }

    fn below(self, first: Minimum, second: Minimum, gap: u16) -> Option<Division> {
        if first.width.max(second.width) > self.width {
            return None;
        }
        let height = allocation(self.height.checked_sub(gap)?, first.height, second.height)?;
        let first = Self { height, ..self };
        let row = self.row.checked_add(height)?;
        let second = Self {
            row: row.checked_add(gap)?,
            height: self.height - height - gap,
            ..self
        };
        Some(Division {
            first,
            second,
            border: (gap > 0).then_some(Self {
                row,
                height: gap,
                ..self
            }),
        })
    }
    fn right(self, first: Minimum, second: Minimum, gap: u16) -> Option<Division> {
        if first.height.max(second.height) > self.height {
            return None;
        }
        let width = allocation(self.width.checked_sub(gap)?, first.width, second.width)?;
        let first = Self { width, ..self };
        let column = self.column.checked_add(width)?;
        let border = Self {
            column,
            width: gap,
            ..self
        };
        let second = Self {
            column: column.checked_add(gap)?,
            width: self.width - width - gap,
            ..self
        };
        Some(Division {
            first,
            second,
            border: (gap > 0).then_some(border),
        })
    }
}

/// Prefer equal halves, but reserve each child's complete recursive minimum.
/// An undersized report suspends the whole projection without a partial layout.
fn allocation(available: u16, first: u16, second: u16) -> Option<u16> {
    let maximum = available.checked_sub(second)?;
    if first > maximum {
        return None;
    }
    Some((available / 2).clamp(first, maximum))
}
