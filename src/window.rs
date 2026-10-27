mod geometry;
mod layout;
mod lifecycle;
mod points;

pub use geometry::{Area, Geometry, Split};
pub use layout::Layout;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::buffer::{Buffer, BufferId, Buffers};
use crate::view::{Size, View};

static NEXT: AtomicU64 = AtomicU64::new(1);

/// Process-local view identity, distinct from buffer identity and never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViewId(u64);

impl ViewId {
    fn next() -> Result<Self, Error> {
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map(Self)
        .map_err(|_| Error::Exhausted)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing(ViewId),
    Exhausted,
    DoesNotFit,
    Focus,
    OnlyWindow,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(id) => write!(formatter, "view {id:?} is not live in this layout"),
            Self::Exhausted => formatter.write_str("view identities are exhausted"),
            Self::DoesNotFit => formatter.write_str("Split does not fit the available pane"),
            Self::Focus => formatter.write_str("a prompt owns input focus"),
            Self::OnlyWindow => formatter.write_str("Cannot delete the sole editing window"),
        }
    }
}
impl std::error::Error for Error {}

/// Sole owner of live views and their split tree. Selection always names one
/// registry entry; only this owner can publish structural changes.
pub struct Windows {
    views: BTreeMap<ViewId, View>,
    selected: ViewId,
    layout: Layout,
    ratio: f64,
}

impl Windows {
    pub fn new(view: View) -> Result<Self, Error> {
        Self::from_ratio(view, 0.5)
    }

    /// Apply the admitted global ratio to initial and resized layout projection.
    pub fn with_settings(view: View, settings: &crate::config::Snapshot) -> Result<Self, Error> {
        Self::from_ratio(view, settings.settings().windows.split_ratio)
    }

    fn from_ratio(view: View, ratio: f64) -> Result<Self, Error> {
        let id = ViewId::next()?;
        Ok(Self {
            views: BTreeMap::from([(id, view)]),
            selected: id,
            layout: Layout::Leaf(id),
            ratio,
        })
    }
    pub fn selected_id(&self) -> ViewId {
        self.selected
    }
    pub fn selected(&self) -> &View {
        &self.views[&self.selected]
    }
    pub(crate) fn selected_mut(&mut self) -> &mut View {
        // Construction and every structural publication retain a live selection.
        self.views
            .get_mut(&self.selected)
            .expect("selected view is a live registry entry")
    }
    pub fn get(&self, id: ViewId) -> Result<&View, Error> {
        self.views.get(&id).ok_or(Error::Missing(id))
    }

    /// Change one live view's native presentation leaves without touching point,
    /// text, layout or other views. The caller owns event-driven redraw scheduling.
    pub fn change_gutter(
        &mut self,
        id: ViewId,
        change: crate::gutter::Change,
    ) -> Result<bool, Error> {
        let view = self.views.get_mut(&id).ok_or(Error::Missing(id))?;
        Ok(view.gutter.apply(change))
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (ViewId, &View)> {
        self.views.iter().map(|(id, view)| (*id, view))
    }
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
    pub fn geometry(&self, size: Size) -> Option<Geometry> {
        self.layout.geometry(size, self.ratio)
    }
    pub(crate) fn panel_geometry(&self, area: Area) -> Option<Geometry> {
        self.layout.panel_geometry(area, self.ratio)
    }

    pub fn area(&self, id: ViewId, size: Size) -> Option<Area> {
        self.geometry(size)?
            .panes()
            .iter()
            .find(|(view, _)| *view == id)
            .map(|(_, area)| *area)
    }
    pub(crate) fn select(&mut self, id: ViewId) -> Result<(), Error> {
        self.get(id)?;
        self.selected = id;
        Ok(())
    }

    /// Fit the entire existing tree and both proposed children before allocating
    /// an identity or publishing a new leaf. Original focus and view are retained.
    pub fn split(&mut self, direction: Split, size: Size) -> Result<ViewId, Error> {
        let area = self.area(self.selected, size).ok_or(Error::DoesNotFit)?;
        area.divide(direction, self.ratio)
            .ok_or(Error::DoesNotFit)?;
        let id = ViewId::next()?;
        let view = self.selected().clone();
        let mut candidate = self.layout.clone();
        if !candidate.split(self.selected, id, direction) {
            return Err(Error::Missing(self.selected));
        }
        self.views.insert(id, view);
        self.layout = candidate;
        Ok(id)
    }

    pub(crate) fn reveal(
        &mut self,
        buffers: &Buffers,
        size: Size,
    ) -> Result<(), crate::view::Error> {
        let Some(geometry) = self.geometry(size) else {
            return Ok(());
        };
        for (id, area) in geometry.panes() {
            let view = self.views.get_mut(id).ok_or(Error::Missing(*id))?;
            view.reveal(buffers.get(view.point.stamp.buffer)?, area.view_size())?;
        }
        Ok(())
    }

    pub(crate) fn replace_buffer(&mut self, closed: BufferId, replacement: &Buffer) {
        for view in self
            .views
            .values_mut()
            .filter(|view| view.point.stamp.buffer == closed)
        {
            view.rebind(replacement);
        }
    }
}
