use crate::display::Position;
use crate::tree::Tree;
use crate::view::Size;
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
    viewport: Area,
    tree: Option<Panel>,
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
        let geometry = windows.panel_geometry(editing)?;
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
            viewport: editing,
            tree: None,
            auxiliary,
            prompt: Area::new(outer.column(), outer.row() + available, outer.width(), 1)?,
        })
    }

    /// Compose tree and editing geometry without changing either semantic owner.
    /// An editing tree that does not fit is suspended while tree/prompt survive.
    /// Auxiliary rows reserve full-width content above the global prompt only
    /// after all editing minima fit; tree and editing share its separator.
    pub fn workspace(
        windows: &Windows,
        tree: &Tree,
        size: Size,
        auxiliary_rows: u16,
    ) -> Option<Self> {
        if size.width == 0 || size.height == 0 {
            return None;
        }
        let width = tree.width(size);
        let column = width + u16::from(width > 0);
        let outer = Area::new(column, 0, size.width - column, size.height)?;
        let viewport = Area::new(column, 0, outer.width(), size.height - 1)?;
        let mut composition = Self::new(windows, outer, auxiliary_rows).unwrap_or(Self {
            panels: Vec::new(),
            borders: Vec::new(),
            auxiliary: None,
            viewport,
            tree: None,
            prompt: Area::new(0, size.height - 1, size.width, 1)?,
        });
        composition.prompt = Area::new(0, size.height - 1, size.width, 1)?;
        composition.expand_auxiliary(size.width)?;
        composition.attach_tree(tree, width)?;
        Some(composition)
    }

    /// Put a completion query before its candidate rows within the reserved
    /// completion region. Editing, tree and shared separators retain their bounds.
    pub fn completion(windows: &Windows, tree: &Tree, size: Size, rows: u16) -> Option<Self> {
        let mut composition = Self::workspace(windows, tree, size, rows)?;
        if let Some(auxiliary) = composition.auxiliary {
            let content = auxiliary.content();
            composition.prompt = Area::new(0, content.row(), size.width, 1)?;
            composition.auxiliary = Some(Panel::new(
                Area::new(0, content.row() + 1, size.width, content.height())?,
                false,
                false,
            ));
        }
        Some(composition)
    }

    fn expand_auxiliary(&mut self, width: u16) -> Option<()> {
        if let Some(panel) = self.auxiliary {
            let content = panel.content();
            self.auxiliary = Some(Panel::new(
                Area::new(0, content.row(), width, content.height())?,
                false,
                panel.active(),
            ));
            *self.borders.last_mut()? = Area::new(0, self.viewport.height(), width, 1)?;
        }
        Some(())
    }

    fn attach_tree(&mut self, tree: &Tree, width: u16) -> Option<()> {
        if width == 0 {
            return Some(());
        }
        let height = self.viewport.height();
        self.tree = Some(Panel::new(
            Area::new(0, 0, width, height)?,
            true,
            tree.focused(),
        ));
        self.borders.push(Area::new(width, 0, 1, height)?);
        for (_, panel) in &mut self.panels {
            panel.active &= !tree.focused();
        }
        Some(())
    }

    pub fn tree(&self) -> Option<Panel> {
        self.tree
    }

    /// Editing sizes borrow the global prompt reservation for existing view
    /// calculations; the actual viewport excludes that row.
    pub fn editing_size(&self) -> Size {
        self.viewport.view_size()
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.prompt.column() + self.prompt.width(),
            height: self
                .auxiliary
                .map_or(self.prompt.row() + self.prompt.height(), |panel| {
                    (self.prompt.row() + self.prompt.height())
                        .max(panel.outer().row() + panel.outer().height())
                }),
        }
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
