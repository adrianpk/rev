use super::{Key, Session};
use crate::{
    coordinates::{Cell, Coordinates, Scalar},
    session::Action,
    view::View,
};
use std::path::PathBuf;

struct Location {
    path: PathBuf,
    row: usize,
    column: usize,
}
struct Error {
    point: Scalar,
    location: Location,
}

impl Session {
    pub(in crate::session::terminals) fn terminal_next_error(&mut self) -> Action {
        let fresh = !self.terminal_copy_active();
        if fresh {
            self.terminal_copy_mode();
        }
        match self.visit_terminal_error(fresh) {
            Ok(()) => self.message("Terminal error location opened"),
            Err(error) => self.error(&error),
        }
    }
    fn visit_terminal_error(&mut self, first: bool) -> Result<(), String> {
        let key = self
            .copy_key()
            .ok_or("Error navigation requires terminal focus")?;
        let (id, error) = self.next_copy_error(key, first)?;
        let directory = self
            .terminals
            .contexts
            .get(&id)
            .ok_or("Terminal creation context unavailable")?;
        let path = directory.join(&error.location.path);
        if !path.is_file() {
            return Err("Recognized terminal error file is unavailable; source retained".into());
        }
        let visit = self
            .buffers
            .prepare_visit(&path)
            .map_err(|e| e.to_string())?;
        let view = self.error_location_view(&visit, &error.location)?;
        self.stop_typing().map_err(|e| e.to_string())?;
        self.buffers.publish_visit(visit);
        self.terminals
            .copies
            .get_mut(&key)
            .expect("observed error copy")
            .view
            .point
            .position = error.point;
        self.terminal_return();
        *self.windows.selected_mut() = view;
        self.tree.edit();
        self.sync_input_context();
        Ok(())
    }
    fn next_copy_error(
        &self,
        key: Key,
        first: bool,
    ) -> Result<(crate::term::SessionId, Error), String> {
        let copy = self
            .terminals
            .copies
            .get(&key)
            .ok_or("Terminal copy is unavailable")?;
        let text = copy
            .buffer
            .slice(
                &copy
                    .buffer
                    .span(0..copy.buffer.len_chars())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
        let mut point = 0;
        for line in text.lines() {
            let position = Scalar(point);
            point += line.len_chars();
            if !first && position <= copy.view.point.position {
                continue;
            }
            if line.len_bytes() > crate::term::CONTROL_BYTES {
                continue;
            }
            if let Some(location) = parse(&line.to_string()) {
                return Ok((
                    copy.id,
                    Error {
                        point: position,
                        location,
                    },
                ));
            }
        }
        Err("No recognized retained error location in that direction".into())
    }
    fn error_location_view(
        &self,
        visit: &crate::buffer::Visit,
        location: &Location,
    ) -> Result<View, String> {
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        if !buffer.file().is_some_and(|file| file.is_present())
            || location.row >= buffer.len_lines()
        {
            return Err("Recognized terminal error source location is unavailable".into());
        }
        let point = error_position(buffer, location)?;
        let mut view = View::with_settings(buffer, self.buffers.snapshot());
        view.point.position = point;
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        Ok(view)
    }
}

// Admit a concrete compiler location with a positive line/optional column and
// an explicit error label. Ordinary prompt text and malformed positions cannot
// authorize source navigation. Paths remain literal data in creation context.
fn parse(line: &str) -> Option<Location> {
    let (path, rest) = line.trim().split_once(':')?;
    if path.is_empty() || path.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let (row, rest) = rest.split_once(':')?;
    let row = positive(row)?;
    let (column, message) = position(rest)?;
    message.trim_start().strip_prefix("error:")?;
    Some(Location {
        path: path.into(),
        row,
        column,
    })
}
fn position(rest: &str) -> Option<(usize, &str)> {
    if rest.trim_start().starts_with("error:") {
        return Some((0, rest));
    }
    let (column, message) = rest.split_once(':')?;
    Some((positive(column)?, message))
}
fn positive(value: &str) -> Option<usize> {
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse::<usize>().ok()?.checked_sub(1)
}

fn error_position(buffer: &crate::buffer::Buffer, location: &Location) -> Result<Scalar, String> {
    let coordinates = Coordinates::new(buffer).map_err(|e| e.to_string())?;
    let cell = Cell {
        row: location.row,
        column: location.column,
    };
    let point = coordinates.from_cell(cell).map_err(|e| e.to_string())?;
    let observed = coordinates.to_cell(point).map_err(|e| e.to_string())?;
    if observed.row != cell.row || observed.column != cell.column {
        return Err("Recognized terminal error column is not represented in source".into());
    }
    Ok(point)
}
