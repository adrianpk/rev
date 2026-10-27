use crate::buffer::Buffers;
use crate::coordinates::{Error as CoordinateError, Glyphs};
use crate::panel::{Composition, Panel};
use crate::window::{Area, Windows};

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
        let outer = Area::new(0, 0, size.width, size.height).ok_or(CoordinateError::Overflow)?;
        if let Some(composition) = Composition::new(windows, outer, 0) {
            frame.panes(buffers, windows, &composition)?;
            frame.dividers(&composition)?;
        }
        Ok(frame)
    }

    fn panes(
        &mut self,
        buffers: &Buffers,
        windows: &Windows,
        composition: &Composition,
    ) -> Result<(), Error> {
        for (id, panel) in composition.panels() {
            let area = panel.outer();
            let view = windows.get(*id)?;
            let buffer = buffers.get(view.point.stamp.buffer)?;
            let selected = panel.active();
            let mut pane = Self::project_view(buffer, view, area.view_size(), "", selected)?;
            if !selected {
                let status = &mut pane.rows[usize::from(area.height() - 1)];
                status.text.replace_range(..1, " ");
            }
            self.paint(*panel, &pane)?;
        }
        Ok(())
    }

    fn paint(&mut self, panel: Panel, pane: &Frame) -> Result<(), Error> {
        let area = panel.outer();
        for row in pane.rows.iter().take(usize::from(area.height())) {
            self.blit(area, row)?;
        }
        if panel.active() {
            self.cursor = pane.cursor.and_then(|cursor| panel.position(cursor));
        }
        Ok(())
    }

    fn dividers(&mut self, composition: &Composition) -> Result<(), Error> {
        for area in composition.borders() {
            for row in 0..area.height() {
                let text = (0..area.width())
                    .map(|column| {
                        composition
                            .separator(Position {
                                column: area.column() + column,
                                row: area.row() + row,
                            })
                            .unwrap_or(' ')
                    })
                    .collect();
                self.blit(
                    *area,
                    &Row {
                        row,
                        text,
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
