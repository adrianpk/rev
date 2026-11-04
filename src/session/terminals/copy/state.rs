use super::{search::Search, snapshot::Snapshot};
use crate::{
    buffer::{Buffer, Span},
    command::{Motion, Navigation, RegionCommand, Scroll, navigation::Target},
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
    pointer_anchor: Option<Scalar>,
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
        let mut view = View::with_settings(&buffer, settings);
        view.gutter.apply(crate::gutter::Change::Visible(false));
        view.bookmark_markers = false;
        view.preview.enabled = false;
        view.top = snapshot.top(viewport, &buffer)?;
        view.point.position = if viewport.offset == 0 {
            snapshot.point(session, session.screen().grid().cursor.point)
        } else {
            Coordinates::new(&buffer)
                .map_err(|e| e.to_string())?
                .from_cell(crate::coordinates::Cell {
                    row: view.top,
                    column: 0,
                })
                .map_err(|e| e.to_string())?
        };
        if let Some(selection) = viewport.selection {
            let anchor = snapshot.point(session, selection.anchor);
            let end = snapshot.point(session, selection.end);
            view.point.position = anchor.min(end);
            buffer
                .activate_mark(view.point)
                .map_err(|e| e.to_string())?;
            view.point.position = Coordinates::new(&buffer)
                .map_err(|e| e.to_string())?
                .next_observed(anchor.max(end))
                .map_err(|e| e.to_string())?;
        }
        let coordinates = Coordinates::new(&buffer).map_err(|e| e.to_string())?;
        if !coordinates
            .is_boundary(view.point.position)
            .map_err(|e| e.to_string())?
        {
            view.point.position = coordinates
                .previous(view.point.position)
                .map_err(|e| e.to_string())?;
        }
        Ok(Self {
            id,
            buffer,
            view,
            charge,
            mode: false,
            search: None,
            pointer_anchor: None,
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
    pub fn navigate(&mut self, request: Navigation, size: Size) -> Result<(), String> {
        match request.target {
            Target::Line(direction) => self.scroll(
                if direction == crate::search::Direction::Backward {
                    Scroll::PreviousLine
                } else {
                    Scroll::NextLine
                },
                request.amount(),
                size,
            ),
            Target::Page { backward, half } => self.scroll(
                if backward {
                    Scroll::PageBackward
                } else {
                    Scroll::PageForward
                },
                usize::from(size.text_rows()).max(1) / if half { 2 } else { 1 } * request.amount(),
                size,
            ),
            Target::Align(_) => self.scroll(Scroll::Recenter, 0, size),
            _ => self.navigate_text(request),
        }
    }
    fn navigate_text(&mut self, request: Navigation) -> Result<(), String> {
        let coordinates = Coordinates::new(&self.buffer).map_err(|e| e.to_string())?;
        self.view.point.position = coordinates
            .navigation(self.view.point.position, request.target, request.amount())
            .map_err(|e| e.to_string())?;
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
