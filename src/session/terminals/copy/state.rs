use super::{search::Search, snapshot::Snapshot};
use crate::{
    buffer::{Buffer, Span},
    command::{Motion, RegionCommand, Scroll},
    coordinates::{Coordinates, Scalar},
    term::{Session, SessionId, Viewport},
    view::{Size, View},
};

pub(in crate::session::terminals) struct State {
    pub id: SessionId,
    pub buffer: Buffer,
    pub view: View,
    pub charge: usize,
    pub mode: bool,
    pub(super) search: Option<Search>,
    pub(super) last_query: Option<super::query::Query>,
    pointer_anchor: Option<Scalar>,
    pub(super) pointer_cell: Option<(u16, u16)>,
    pub(super) finding: Option<super::navigation::Finding>,
    pub(super) page_overlap: usize,
}

impl State {
    pub fn capture(
        id: SessionId,
        session: &Session,
        viewport: Viewport,
        settings: &crate::config::Snapshot,
    ) -> Result<Self, String> {
        let snapshot = Snapshot::capture(session)?;
        let charge = snapshot.charge();
        let mut buffer =
            Buffer::terminal_copy(&snapshot.text, settings).map_err(|e| e.to_string())?;
        let view = snapshot.view(session, &mut buffer, viewport, settings)?;
        let pointer_anchor = buffer.mark();
        Ok(Self {
            id,
            buffer,
            view,
            charge,
            mode: false,
            search: None,
            last_query: None,
            pointer_anchor,
            pointer_cell: None,
            finding: None,
            page_overlap: settings.settings().scroll.page_overlap,
        })
    }

    pub fn reveal(&mut self, size: Size) -> Result<(), String> {
        self.view
            .reveal(&self.buffer, size)
            .map_err(|e| e.to_string())
    }
    pub fn motion(&mut self, motion: Motion) -> Result<(), String> {
        let point = motion
            .target(
                &Coordinates::new(&self.buffer).map_err(|e| e.to_string())?,
                self.view.point.position,
            )
            .map_err(|e| e.to_string())?;
        self.view.point.position = point;
        self.view.reset_navigation();
        Ok(())
    }
    pub fn scroll(&mut self, scroll: Scroll, distance: usize, size: Size) -> Result<(), String> {
        let result = if scroll == Scroll::Recenter {
            self.view.recenter(&self.buffer, size)
        } else {
            self.view.scroll(&self.buffer, scroll, distance)
        };
        result.map_err(|e| e.to_string())
    }
    pub fn region(&mut self, command: RegionCommand) -> Result<(), String> {
        match command {
            RegionCommand::Set => self
                .buffer
                .activate_mark(self.view.point)
                .map_err(|e| e.to_string()),
            RegionCommand::Exchange => {
                self.view.point = self
                    .buffer
                    .exchange_mark(self.view.point)
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
        }
    }
    pub fn selected(&self) -> Result<Span, String> {
        self.buffer
            .region(self.view.point)
            .map_err(|e| e.to_string())?
            .ok_or("No terminal copy selection is active".into())
    }
    pub fn clear_selection(&mut self) -> bool {
        let active = self.buffer.mark_active();
        self.buffer.deactivate_region();
        active
    }
    pub fn pointer(
        &mut self,
        event: crossterm::event::MouseEvent,
        extend: bool,
    ) -> Result<(), String> {
        let row = (self.view.top + usize::from(event.row)).min(self.buffer.len_lines() - 1);
        let coordinates = Coordinates::new(&self.buffer).map_err(|e| e.to_string())?;
        let point = coordinates
            .from_cell(crate::coordinates::Cell {
                row,
                column: self.view.left + usize::from(event.column),
            })
            .map_err(|e| e.to_string())?;
        if !extend {
            self.pointer_anchor = Some(point);
        }
        let anchor = self.pointer_anchor.unwrap_or(point);
        let (mark, end) = if point < anchor {
            (
                coordinates
                    .next_observed(anchor)
                    .map_err(|e| e.to_string())?,
                point,
            )
        } else {
            (
                anchor,
                coordinates
                    .next_observed(point)
                    .map_err(|e| e.to_string())?,
            )
        };
        self.view.point.position = mark;
        self.buffer
            .activate_mark(self.view.point)
            .map_err(|e| e.to_string())?;
        self.view.point.position = end;
        Ok(())
    }
    pub fn frame(
        &self,
        frame: &mut crate::display::Frame,
        panel: crate::panel::Panel,
        label: &str,
    ) -> Result<(), crate::view::Error> {
        frame.terminal_copy(&self.buffer, &self.view, panel, label)?;
        if let Some(found) = self.search.as_ref().and_then(|s| s.found.as_ref()) {
            frame.search(found, &self.buffer, &self.view, Some(panel))?;
        }
        Ok(())
    }
}
