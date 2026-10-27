use crate::display::Position;
use crate::window::{Area, ViewId, Windows};

use super::{Panel, border};

/// A disposable projection of the existing layout. It retains live ViewIds,
/// compact separator spans, one prompt row and optional completion content.
/// An undersized surface suspends the entire editing projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    panels: Vec<(ViewId, Panel)>,
    borders: Vec<Area>,
    auxiliary: Option<Panel>,
    prompt: Area,
}

impl Composition {
    /// Reserve the prompt first, then up to the requested auxiliary rows plus
    /// one shared separator. Every editing leaf keeps its recursive minimum
    /// of three columns, one content row and one status row.
    pub fn new(windows: &Windows, outer: Area, auxiliary_rows: u16) -> Option<Self> {
        let minimum = windows.layout().panel_minimum()?;
        let available = outer.height().checked_sub(1)?;
        let surplus = available.checked_sub(minimum.height)?;
        let rows = auxiliary_rows.min(surplus.saturating_sub(1));
        let height = available - rows - u16::from(rows > 0);
        let editing = Area::new(outer.column(), outer.row(), outer.width(), height)?;
        let geometry = windows.layout().panel_geometry(editing)?;
        let panels = geometry
            .panes()
            .iter()
            .map(|(id, area)| (*id, Panel::new(*area, true, *id == windows.selected_id())))
            .collect();
        let mut borders = geometry.borders().to_vec();
        let auxiliary = if rows > 0 {
            let row = outer.row() + height;
            borders.push(Area::new(outer.column(), row, outer.width(), 1)?);
            Some(Panel::new(
                Area::new(outer.column(), row + 1, outer.width(), rows)?,
                false,
                false,
            ))
        } else {
            None
        };
        Some(Self {
            panels,
            borders,
            auxiliary,
            prompt: Area::new(outer.column(), outer.row() + available, outer.width(), 1)?,
        })
    }

    pub fn panels(&self) -> &[(ViewId, Panel)] {
        &self.panels
    }
    pub fn borders(&self) -> &[Area] {
        &self.borders
    }
    pub fn auxiliary(&self) -> Option<Panel> {
        self.auxiliary
    }
    pub fn prompt(&self) -> Area {
        self.prompt
    }

    /// Join neighboring reserved separator cells, including T/cross junctions.
    /// No outer frame or terminal-sized raster is allocated.
    pub fn separator(&self, position: Position) -> Option<char> {
        border::glyph(&self.borders, position)
    }
}
