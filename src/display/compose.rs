use crate::buffer::Buffers;
use crate::coordinates::{Error as CoordinateError, Glyphs};
use crate::window::{Area, Geometry, Windows};

use super::{Band, Error, Frame, Position, Row, Size, clip};

impl Frame {
    /// Compose validated pane projections and dividers into one cell-bounded
    /// terminal frame, keeping one global echo and only the selected cursor.
    pub fn compose(
        buffers: &Buffers,
        windows: &Windows,
        size: Size,
        echo: &str,
    ) -> Result<Self, Error> {
        let selected = buffers.get(windows.selected().point.stamp.buffer)?;
        if windows.iter().len() == 1 {
            return Self::project(selected, windows.selected(), size, echo);
        }
        let mut frame = Self {
            size,
            stamp: selected.stamp(),
            rows: Vec::new(),
            cursor: None,
        };
        if size.width == 0 || size.height == 0 {
            return Ok(frame);
        }
        frame.blank_rows();
        frame.echo(echo)?;
        if let Some(geometry) = windows.geometry(size) {
            frame.panes(buffers, windows, &geometry)?;
            frame.dividers(geometry.borders())?;
        }
        Ok(frame)
    }

    fn panes(
        &mut self,
        buffers: &Buffers,
        windows: &Windows,
        geometry: &Geometry,
    ) -> Result<(), Error> {
        for (id, area) in geometry.panes() {
            let view = windows.get(*id)?;
            let buffer = buffers.get(view.point.stamp.buffer)?;
            let selected = *id == windows.selected_id();
            let mut pane = Self::project_view(buffer, view, area.view_size(), "", selected)?;
            if !selected {
                let status = &mut pane.rows[usize::from(area.height() - 1)];
                status.text.replace_range(..1, " ");
            }
            self.paint(*area, &pane, selected)?;
        }
        Ok(())
    }

    fn paint(&mut self, area: Area, pane: &Frame, selected: bool) -> Result<(), Error> {
        for row in pane.rows.iter().take(usize::from(area.height())) {
            self.blit(area, row)?;
        }
        if selected && let Some(cursor) = pane.cursor {
            self.cursor = Some(Position {
                column: area
                    .column()
                    .checked_add(cursor.column)
                    .ok_or(CoordinateError::Overflow)?,
                row: area
                    .row()
                    .checked_add(cursor.row)
                    .ok_or(CoordinateError::Overflow)?,
            });
        }
        Ok(())
    }

    fn dividers(&mut self, borders: &[Area]) -> Result<(), Error> {
        for area in borders {
            for row in 0..area.height() {
                self.blit(
                    *area,
                    &Row {
                        row,
                        text: "|".into(),
                        syntax: Vec::new(),
                        band: Band::Divider,
                        search: None,
                        selection: None,
                    },
                )?;
            }
        }
        Ok(())
    }

    fn blit(&mut self, area: Area, source: &Row) -> Result<(), Error> {
        let row = area
            .row()
            .checked_add(source.row)
            .ok_or(CoordinateError::Overflow)?;
        let target = self
            .rows
            .get_mut(usize::from(row))
            .ok_or(CoordinateError::Overflow)?;
        let end = area
            .column()
            .checked_add(area.width())
            .ok_or(CoordinateError::Overflow)?;
        let left = clip::line(Glyphs::for_text(&target.text)?, 0, area.column())?;
        let right = clip::line(
            Glyphs::for_text(&target.text)?,
            usize::from(end),
            self.size.width - end,
        )?;
        target.text = format!("{left}{}{right}", source.text);
        target
            .syntax
            .extend(source.syntax.iter().map(|paint| super::Paint {
                columns: area.column() + paint.columns.start..area.column() + paint.columns.end,
                style: paint.style,
            }));
        if let Some(selection) = &source.selection {
            target.selection = Some(area.column() + selection.start..area.column() + selection.end);
        }
        target.band = if area.width() == self.size.width {
            source.band
        } else {
            Band::Mixed
        };
        Ok(())
    }
}
