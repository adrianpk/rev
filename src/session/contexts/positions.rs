use crate::buffer::{Buffer, BufferId, edit::Point};
use crate::context::{SourcePosition, Viewport};
use crate::coordinates::{Coordinates, LineByte, Scalar};
use crate::view::View;
use crate::window::ViewId;
use std::collections::BTreeMap;

use super::super::Session;

/// Source anchors and window-local Org choices survive live buffer switches.
/// This transient state is never serialized into durable sessions.
#[derive(Clone)]
pub(in crate::session) struct Position {
    point: Point,
    top: usize,
    wrap_start: Scalar,
    left: usize,
    org: BTreeMap<ViewId, crate::view::OrgPresentation>,
}
impl Position {
    fn from_view(view: &View, org: BTreeMap<ViewId, crate::view::OrgPresentation>) -> Self {
        Self {
            point: view.point,
            top: view.top,
            wrap_start: view.wrap_start,
            left: view.left,
            org,
        }
    }
    fn apply(&self, view: &mut View, buffer: &Buffer, id: ViewId) {
        if self.point.stamp == buffer.stamp() {
            view.point = self.point;
            view.top = self.top;
            view.wrap_start = self.wrap_start;
            view.left = self.left;
        }
        if let Some(state) = self.org.get(&id) {
            view.restore_org_presentation(state);
        }
    }
}
impl Session {
    pub(in crate::session) fn remember_context_position(&mut self) {
        let view = self.windows.selected();
        let buffer = view.point.stamp.buffer;
        let mut org = self
            .context_positions
            .remove(&buffer)
            .map(|p| p.org)
            .unwrap_or_default();
        // The live buffer catalog and live windows bound retention. Closed
        // buffers already retire their position entry through the normal owner.
        org.retain(|id, _| self.windows.get(*id).is_ok());
        org.insert(self.windows.selected_id(), view.org_presentation());
        self.context_positions
            .insert(buffer, Position::from_view(view, org));
    }
    pub(in crate::session) fn context_view(&self, buffer: &Buffer) -> View {
        let mut view = self.view().clone();
        view.rebind(buffer);
        if let Some(position) = self.context_positions.get(&buffer.id()) {
            position.apply(&mut view, buffer, self.windows.selected_id());
        }
        view
    }
    pub(in crate::session) fn capture_context_position(
        &self,
        id: BufferId,
    ) -> Result<(SourcePosition, Viewport), String> {
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let view = if self.view().point.stamp.buffer == id {
            self.view().clone()
        } else if let Some((_, view)) = self
            .windows
            .iter()
            .find(|(_, v)| v.point.stamp.buffer == id)
        {
            view.clone()
        } else {
            self.context_view(buffer)
        };
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let start = coordinates
            .from_line(LineByte {
                row: view.top,
                column: 0,
            })
            .map_err(|e| e.to_string())?;
        let anchor = if buffer.soft_wrap() {
            view.wrap_start.clamp(
                start,
                coordinates.line_end(start).map_err(|e| e.to_string())?,
            )
        } else {
            start
        };
        let cursor =
            SourcePosition::capture(buffer, view.point.position).map_err(|e| e.to_string())?;
        let anchor = SourcePosition::capture(buffer, anchor).map_err(|e| e.to_string())?;
        Ok((
            cursor,
            Viewport {
                anchor,
                horizontal_cells: view.left,
            },
        ))
    }
    pub(in crate::session) fn restore_context_position(
        &mut self,
        id: BufferId,
        entry: &crate::context::FileEntry,
    ) -> Result<View, String> {
        let buffer = self.buffers.get(id).map_err(|e| e.to_string())?;
        let mut view = View::with_settings(buffer, self.buffers.snapshot());
        let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
        let point = entry.cursor.restore(buffer).map_err(|e| e.to_string())?;
        view.point.position = if coordinates.is_boundary(point).map_err(|e| e.to_string())? {
            point
        } else {
            coordinates.previous(point).map_err(|e| e.to_string())?
        };
        if let Some(saved) = entry.viewport {
            let anchor = saved.anchor.restore(buffer).map_err(|e| e.to_string())?;
            let anchor = if coordinates.is_boundary(anchor).map_err(|e| e.to_string())? {
                anchor
            } else {
                coordinates.previous(anchor).map_err(|e| e.to_string())?
            };
            view.top = Coordinates::new(buffer)
                .map_err(|e| e.to_string())?
                .to_line(anchor)
                .map_err(|e| e.to_string())?
                .row;
            view.wrap_start = anchor;
            view.left = saved.horizontal_cells;
        }
        self.context_positions
            .insert(id, Position::from_view(&view, BTreeMap::new()));
        Ok(view)
    }
}
