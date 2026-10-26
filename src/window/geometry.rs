use crate::view::Size;

use super::ViewId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Below,
    Right,
}

/// Validated pane rectangle, including its own status and excluding session echo.
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

    pub(super) fn join(split: Split, first: Self, second: Self) -> Option<Self> {
        match split {
            Split::Below => Some(Self {
                width: first.width.max(second.width),
                height: first.height.checked_add(second.height)?,
            }),
            Split::Right => Some(Self {
                width: first.width.checked_add(1)?.checked_add(second.width)?,
                height: first.height.max(second.height),
            }),
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
        self.divide_to(split, Minimum::leaf(), Minimum::leaf())
    }

    pub(super) fn divide_to(
        self,
        split: Split,
        first: Minimum,
        second: Minimum,
    ) -> Option<Division> {
        match split {
            Split::Below => self.below(first, second),
            Split::Right => self.right(first, second),
        }
    }

    fn below(self, first: Minimum, second: Minimum) -> Option<Division> {
        if first.width.max(second.width) > self.width {
            return None;
        }
        let height = allocation(self.height, first.height, second.height)?;
        let first = Self { height, ..self };
        let second = Self {
            row: self.row.checked_add(height)?,
            height: self.height - height,
            ..self
        };
        Some(Division {
            first,
            second,
            border: None,
        })
    }
    fn right(self, first: Minimum, second: Minimum) -> Option<Division> {
        if first.height.max(second.height) > self.height {
            return None;
        }
        let width = allocation(self.width.checked_sub(1)?, first.width, second.width)?;
        let first = Self { width, ..self };
        let column = self.column.checked_add(width)?;
        let border = Self {
            column,
            width: 1,
            ..self
        };
        let second = Self {
            column: column.checked_add(1)?,
            width: self.width - width - 1,
            ..self
        };
        Some(Division {
            first,
            second,
            border: Some(border),
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
