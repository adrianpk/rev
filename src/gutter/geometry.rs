use std::fmt;

use crate::window::Area;

use super::{Format, Settings, Side};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LineCount,
    Overflow,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LineCount => "gutter requires at least one logical line",
            Self::Overflow => "gutter geometry overflow",
        })
    }
}

impl std::error::Error for Error {}

/// Checked terminal-cell partition within panel content. Status/border/prompt
/// bounds remain outside it. No row labels or document coordinates are stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Geometry {
    digits: u8,
    numbers: Option<Area>,
    text: Area,
}

impl Geometry {
    pub const SEPARATOR_WIDTH: u16 = 1;
    /// Match the editing panel's existing protected three-column minimum.
    pub const MIN_TEXT_COLUMNS: u16 = 3;

    pub fn new(content: Area, lines: usize, settings: Settings) -> Result<Self, Error> {
        let digits = digits(lines, settings.format.minimum_digits())?;
        let width = field_width(digits, settings.format).ok_or(Error::Overflow)?;
        let mut geometry = Self {
            digits,
            numbers: None,
            text: content,
        };
        if settings.visible
            && content.height() > 0
            && let Some((numbers, text)) = partition(content, width, settings.side)
        {
            geometry.numbers = Some(numbers);
            geometry.text = text;
        }
        Ok(geometry)
    }

    pub fn digits(self) -> u8 {
        self.digits
    }
    /// None means hidden or insufficient content geometry. Requested policy is
    /// retained by the view, so a later projection can restore the full field.
    pub fn numbers(self) -> Option<Area> {
        self.numbers
    }
    pub fn text(self) -> Area {
        self.text
    }
}

fn digits(lines: usize, minimum: u8) -> Result<u8, Error> {
    let exponent = lines.checked_ilog10().ok_or(Error::LineCount)?;
    let width = u8::try_from(exponent.checked_add(1).ok_or(Error::Overflow)?)
        .map_err(|_| Error::Overflow)?;
    if width > Format::MAX_DIGITS {
        return Err(Error::Overflow);
    }
    Ok(width.max(minimum))
}

fn field_width(digits: u8, format: Format) -> Option<u16> {
    u16::from(digits)
        .checked_add(u16::from(format.left_padding()))?
        .checked_add(u16::from(format.right_padding()))?
        .checked_add(Geometry::SEPARATOR_WIDTH)
}

fn partition(content: Area, width: u16, side: Side) -> Option<(Area, Area)> {
    let remaining = content.width().checked_sub(width)?;
    if remaining < Geometry::MIN_TEXT_COLUMNS {
        return None;
    }
    let (number_column, text_column) = match side {
        Side::Left => (content.column(), content.column().checked_add(width)?),
        Side::Right => (content.column().checked_add(remaining)?, content.column()),
    };
    Some((
        Area::new(number_column, content.row(), width, content.height())?,
        Area::new(text_column, content.row(), remaining, content.height())?,
    ))
}
