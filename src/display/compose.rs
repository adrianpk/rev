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
        let outer = Area::new(0, 0, size.width, size.height).ok_or(CoordinateError::Overflow)?;
        let composition = Composition::new(windows, outer, 0);
        Self::composition(buffers, windows, size, echo, composition.as_ref())
    }

    /// Draw a supplied native composition without recomputing its placement.
    pub fn compose_in(
        buffers: &Buffers,
        windows: &Windows,
        composition: &Composition,
        echo: &str,
    ) -> Result<Self, Error> {
        Self::composition(
            buffers,
            windows,
            composition.size(),
            echo,
            Some(composition),
        )
    }

    fn composition(
        buffers: &Buffers,
        windows: &Windows,
        size: Size,
        echo: &str,
        composition: Option<&Composition>,
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
        if let Some(composition) = composition {
            frame.panes(buffers, windows, composition)?;
            frame.dividers(composition)?;
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
            let view = windows.get(*id)?;
            let buffer = buffers.get(view.point.stamp.buffer)?;
            self.panel(buffer, view, *panel)?;
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
                    Row {
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

    pub(super) fn blit(&mut self, area: Area, source: Row) -> Result<(), Error> {
        let row = area
            .row()
            .checked_add(source.row)
            .ok_or(CoordinateError::Overflow)?;
        let target = self
            .rows
            .get_mut(usize::from(row))
            .ok_or(CoordinateError::Overflow)?;
        target.text = text(&target.text, source.text, area, self.size.width)?;
        target.syntax.extend(source.syntax);
        if source.selection.is_some() {
            target.selection = source.selection;
        }
        if source.search.is_some() {
            target.search = source.search;
        }
        target.band = if area.width() == self.size.width {
            source.band
        } else {
            Band::Mixed
        };
        Ok(())
    }
}

impl Row {
    pub(super) fn projected(mut self, panel: Panel) -> Self {
        self.syntax = self
            .syntax
            .into_iter()
            .filter_map(|mut paint| {
                paint.columns = panel.clip(self.row, paint.columns)?.1;
                Some(paint)
            })
            .collect();
        self.selection = self
            .selection
            .and_then(|range| panel.clip(self.row, range).map(|(_, columns)| columns));
        self.search = self
            .search
            .and_then(|range| panel.clip(self.row, range).map(|(_, columns)| columns));
        self
    }
}

fn text(target: &str, source: String, area: Area, width: u16) -> Result<String, Error> {
    if area.column() == 0 && area.width() == width {
        return Ok(source);
    }
    let end = area
        .column()
        .checked_add(area.width())
        .ok_or(CoordinateError::Overflow)?;
    let left = clip::line(Glyphs::for_text(target)?, 0, area.column())?;
    let right = clip::line(Glyphs::for_text(target)?, usize::from(end), width - end)?;
    Ok(format!("{left}{source}{right}"))
}
