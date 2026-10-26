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
        match split {
            Split::Below => self.below(),
            Split::Right => self.right(),
        }
    }
    fn below(self) -> Option<Division> {
        if self.height < 4 {
            return None;
        }
        let height = self.height / 2;
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
    fn right(self) -> Option<Division> {
        if self.width < 7 {
            return None;
        }
        let width = (self.width - 1) / 2;
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
