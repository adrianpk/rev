use crate::buffer::Buffer;
use crate::command::Scroll;
use crate::coordinates::{Cell, Coordinates};

use super::{Error, Size, View};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Alignment {
    Center,
    Top,
    Bottom,
}

impl View {
    /// Move through projected rows, retaining canonical point until it leaves
    /// the viewport. Never clamp to a hidden source row at the near edge.
    pub(crate) fn scroll_viewport(
        &mut self,
        buffer: &Buffer,
        backward: bool,
        distance: usize,
        size: Size,
    ) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        if !size.usable() {
            return Ok(());
        }
        self.reveal_fold(buffer, point.row)?;
        let last = self.visible_lines(buffer) - 1;
        let top = destination(self.projected_top(buffer), distance, backward).min(last);
        self.top = self
            .source_at(buffer, top)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let bottom = top
            .saturating_add(usize::from(size.text_rows()) - 1)
            .min(last);
        let projected = self
            .project_row(buffer, point.row)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let row = projected.clamp(top, bottom);
        if row != projected {
            let row = self
                .source_at(buffer, row)
                .ok_or(crate::coordinates::Error::Overflow)?;
            self.point.position = Coordinates::new(buffer)?.from_cell(Cell {
                row,
                column: point.column,
            })?;
        }
        self.reveal(buffer, size)
    }

    pub(crate) fn reset_navigation(&mut self) {
        self.column = None;
        self.alignment = Alignment::Center;
    }

    pub(crate) fn scroll(
        &mut self,
        buffer: &Buffer,
        scroll: Scroll,
        distance: usize,
    ) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        self.reveal_fold(buffer, point.row)?;
        self.sync_policy(buffer);
        if !buffer.soft_wrap() {
            self.sync_wrap_width(None);
        }
        self.alignment = Alignment::Center;
        if distance == 0 {
            return Ok(());
        }
        let column = *self.column.get_or_insert(point.column);
        let origin = self
            .project_row(buffer, point.row)
            .ok_or(crate::coordinates::Error::Overflow)?;
        let row =
            destination(origin, distance, scroll.backward()).min(self.visible_lines(buffer) - 1);
        let source = self
            .source_at(buffer, row)
            .ok_or(crate::coordinates::Error::Overflow)?;
        self.point.position = Coordinates::new(buffer)?.from_cell(Cell {
            row: source,
            column,
        })?;
        if scroll.page() {
            let top = destination(
                self.projected_top(buffer),
                origin.abs_diff(row),
                scroll.backward(),
            )
            .min(self.visible_lines(buffer) - 1);
            self.top = self
                .source_at(buffer, top)
                .ok_or(crate::coordinates::Error::Overflow)?;
        }
        Ok(())
    }

    pub(crate) fn recenter(&mut self, buffer: &Buffer, size: Size) -> Result<(), Error> {
        let point = self.cell(buffer)?;
        self.reveal_fold(buffer, point.row)?;
        self.sync_policy(buffer);
        self.align(point, size, buffer)
    }

    fn align(&mut self, point: Cell, size: Size, buffer: &Buffer) -> Result<(), Error> {
        if !size.usable() {
            return Ok(());
        }
        let rows = usize::from(size.text_rows());
        let width = self.text_width(buffer, size)?;
        self.sync_wrap_width((buffer.soft_wrap() && width > 0).then_some(width));
        let (offset, next) = match self.alignment {
            Alignment::Center => (rows / 2, Alignment::Top),
            Alignment::Top => (0, Alignment::Bottom),
            Alignment::Bottom => (rows - 1, Alignment::Center),
        };
        if buffer.soft_wrap() && width > 0 {
            self.align_wrapped(buffer, width, offset)?;
            self.alignment = next;
            return Ok(());
        }
        let row = self
            .project_row(buffer, point.row)
            .ok_or(crate::coordinates::Error::Overflow)?;
        self.top = self
            .source_at(buffer, row.saturating_sub(offset))
            .ok_or(crate::coordinates::Error::Overflow)?;
        self.alignment = next;
        Ok(())
    }
}

fn destination(origin: usize, distance: usize, backward: bool) -> usize {
    if backward {
        origin.saturating_sub(distance)
    } else {
        origin.saturating_add(distance)
    }
}
