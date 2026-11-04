use super::{Session, snapshot::Snapshot};
use crate::{command::Motion, coordinates::Coordinates, session::Action};

impl Session {
    pub(in crate::session::terminals) fn terminal_copy_line(&mut self, end: bool) -> Action {
        if !self.terminal_copy_active() {
            return self.error("Terminal copy mode is not active");
        }
        let key = self.copy_key().expect("active copy view");
        let Some(size) = self.copy_size(key) else {
            return self.error("Terminal copy presentation is suspended");
        };
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("active copy view");
        let motion = if end {
            Motion::LineEnd
        } else {
            Motion::LineStart
        };
        let result = copy.motion(motion).and_then(|_| copy.reveal(size));
        self.copy_result(result)
    }
    pub(in crate::session::terminals) fn terminal_copy_cursor(&mut self) -> Action {
        match self.reset_copy_cursor() {
            Ok(()) => self.message("Terminal copy point aligned with live cursor"),
            Err(error) => self.error(&error),
        }
    }
    fn reset_copy_cursor(&mut self) -> Result<(), String> {
        if !self.terminal_copy_active() {
            return Err("Terminal copy mode is not active".into());
        }
        let key = self.copy_key().expect("active copy view");
        let size = self
            .copy_size(key)
            .ok_or("Terminal copy presentation is suspended")?;
        let copy = self
            .terminals
            .copies
            .get_mut(&key)
            .expect("active copy view");
        let session = self
            .terminals
            .sessions
            .get(copy.id)
            .ok_or("Terminal is unavailable")?;
        let snapshot = Snapshot::capture(session)?;
        let text = copy
            .buffer
            .slice(
                &copy
                    .buffer
                    .span(0..copy.buffer.len_chars())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        if text != snapshot.text.as_str() {
            return Err(
                "Live cursor is not represented by this frozen copy; re-enter copy mode to refresh"
                    .into(),
            );
        }
        let point = snapshot.point(session, session.screen().grid().cursor.point);
        let coordinates = Coordinates::new(&copy.buffer).map_err(|e| e.to_string())?;
        copy.view.point.position = if coordinates.is_boundary(point).map_err(|e| e.to_string())? {
            point
        } else {
            coordinates.previous(point).map_err(|e| e.to_string())?
        };
        copy.view.reset_navigation();
        copy.reveal(size)
    }
}
