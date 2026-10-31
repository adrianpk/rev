use super::*;
use crate::bookmark::{RecordId, list::Row};
use crate::command::{Command, Motion, Scroll};
use std::collections::BTreeSet;

/// A native read-only workspace, attached to one pane, like other result views.
/// Rows and marks address stable records; the backing source is never replaced.
pub(in crate::session) struct List {
    pub view: ViewId,
    origin: Point,
    generation: u64,
    rows: Vec<Row>,
    cursor: usize,
    top: usize,
    marked: BTreeSet<RecordId>,
}
impl Session {
    pub(in crate::session) fn bookmark_list_active(&self) -> bool {
        self.bookmark_list
            .as_ref()
            .is_some_and(|s| s.view == self.windows.selected_id())
    }
    pub(in crate::session) fn list_bookmarks(&mut self) -> Action {
        if self.input.focus() != crate::input::Focus::Editing {
            return self.message("A prompt owns input focus");
        }
        let notice = self.ensure_bookmarks().err().unwrap_or_default();
        if self.bookmark_list_active() {
            self.refresh_bookmark_list();
            return self.message(&notice);
        }
        let records = self.bookmarks();
        let state = List {
            view: self.windows.selected_id(),
            origin: self.view().point,
            generation: records.generation(),
            rows: records.rows(),
            cursor: 0,
            top: 0,
            marked: BTreeSet::new(),
        };
        drop(records);
        self.bookmark_list = Some(state);
        self.tree.edit();
        self.sync_input_context();
        self.message(if notice.is_empty() {
            "Bookmarks: RET visits; d marks; u unmarks; x deletes; q returns"
        } else {
            &notice
        })
    }
    fn refresh_bookmark_list(&mut self) {
        let Some(state) = &mut self.bookmark_list else {
            return;
        };
        let selected = state.rows.get(state.cursor).map(|r| r.id);
        let records = self.buffers.bookmarks();
        let rows = records.rows();
        state.cursor = selected
            .and_then(|id| rows.iter().position(|r| r.id == id))
            .unwrap_or(state.cursor.min(rows.len().saturating_sub(1)));
        state.rows = rows;
        state.generation = records.generation();
        state.marked.clear();
        state.top = state.top.min(state.cursor);
    }
    pub(in crate::session) fn bookmark_list_command(&mut self, command: Command) -> Option<Action> {
        if !self.bookmark_list_active() {
            return None;
        }
        if command == Command::ExecuteExtended {
            return None;
        }
        if command == Command::KeyboardQuit {
            if matches!(self.pending, Some(Pending::Command)) {
                self.pending = None;
                self.input.cancel();
                return Some(self.message("Bookmark command canceled"));
            }
            return Some(self.close_bookmark_list());
        }
        if command == Command::Quit {
            self.bookmark_list = None;
            self.sync_input_context();
            return None;
        }
        if matches!(
            command,
            Command::Window(_)
                | Command::Split(_)
                | Command::Bookmark(BookmarkCommand::Save | BookmarkCommand::Reload)
        ) {
            return None;
        }
        let generation = self.bookmark_list.as_ref().expect("active list").generation;
        if generation != self.bookmarks().generation() {
            self.refresh_bookmark_list();
            return Some(
                self.message(
                    "Bookmark collection changed; marks cleared and list refreshed; retry",
                ),
            );
        }
        Some(match command {
            Command::Bookmark(BookmarkCommand::List) => self.list_bookmarks(),
            Command::Bookmark(BookmarkCommand::MenuDelete) => self.mark_bookmark_row(true),
            Command::Bookmark(BookmarkCommand::MenuUnmark) => self.mark_bookmark_row(false),
            Command::Bookmark(BookmarkCommand::MenuExecute) => self.delete_bookmark_rows(),
            Command::Bookmark(BookmarkCommand::MenuVisit) | Command::Newline => {
                self.visit_bookmark_row()
            }
            Command::Scroll(Scroll::NextLine) => self.scroll_bookmark_list(1, false),
            Command::Scroll(Scroll::PreviousLine) => self.scroll_bookmark_list(1, true),
            Command::Scroll(Scroll::PageForward) => self.scroll_bookmark_list(
                self.pane_size().height.saturating_sub(2).max(1) as usize,
                false,
            ),
            Command::Scroll(Scroll::PageBackward) => self.scroll_bookmark_list(
                self.pane_size().height.saturating_sub(2).max(1) as usize,
                true,
            ),
            Command::Motion(Motion::BufferStart) => self.scroll_bookmark_list(usize::MAX, true),
            Command::Motion(Motion::BufferEnd) => self.scroll_bookmark_list(usize::MAX, false),
            _ => self.message(
                "Read-only bookmark list; RET visits, d/u mark/unmark, x deletes, q returns",
            ),
        })
    }
    fn scroll_bookmark_list(&mut self, distance: usize, backward: bool) -> Action {
        let extent = self.pane_size().height.saturating_sub(2).max(1) as usize;
        let state = self.bookmark_list.as_mut().expect("list row navigation");
        state.cursor = if backward {
            state.cursor.saturating_sub(distance)
        } else {
            state
                .cursor
                .saturating_add(distance)
                .min(state.rows.len().saturating_sub(1))
        };
        state.top = state
            .top
            .min(state.cursor)
            .max(state.cursor.saturating_sub(extent - 1));
        Action::Redraw
    }
    fn mark_bookmark_row(&mut self, marked: bool) -> Action {
        let state = self.bookmark_list.as_mut().expect("list marking");
        let Some(row) = state.rows.get(state.cursor) else {
            return self.message("No bookmarks in the list");
        };
        if marked {
            state.marked.insert(row.id);
        } else {
            state.marked.remove(&row.id);
        }
        self.scroll_bookmark_list(1, false)
    }
    fn delete_bookmark_rows(&mut self) -> Action {
        let state = self.bookmark_list.as_ref().expect("list deletion");
        let targets: Vec<_> = state
            .rows
            .iter()
            .filter(|r| state.marked.contains(&r.id))
            .map(|r| (r.id, r.target))
            .collect();
        if targets.is_empty() {
            return self.message("No bookmarks marked for deletion");
        }
        let result = self
            .buffers
            .bookmarks_mut()
            .delete_rows(state.generation, &targets);
        if let Err(error) = result {
            return self.error(&error);
        }
        self.refresh_bookmark_list();
        self.bookmark_change_message(&format!("Deleted {} bookmarks", targets.len()))
    }
    fn visit_bookmark_row(&mut self) -> Action {
        let state = self.bookmark_list.as_ref().expect("list visit");
        let Some(row) = state.rows.get(state.cursor) else {
            return self.message("No bookmarks in the list");
        };
        let records = self.bookmarks();
        if !records
            .get(&row.name)
            .is_some_and(|r| r.id() == row.id && r.generation() == row.target)
        {
            drop(records);
            return self.error("Bookmark row changed; refresh required");
        }
        let name = row.name.clone();
        drop(records);
        match self.visit_bookmark_from(&name, navigation::Origin::List(state.generation)) {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error),
        }
    }
    fn close_bookmark_list(&mut self) -> Action {
        let state = self.bookmark_list.take().expect("list return");
        self.input.cancel();
        self.pending = None;
        self.sync_input_context();
        let current = self.buffers.get(state.origin.stamp.buffer).is_ok_and(|b| {
            b.stamp() == state.origin.stamp && self.view().point.stamp.buffer == b.id()
        });
        if !current {
            return self.message(
                "Bookmark return location changed or closed; current source view retained",
            );
        }
        self.windows.selected_mut().point = state.origin;
        self.message("Returned from bookmarks; unexecuted marks discarded")
    }
    pub(in crate::session) fn bookmark_list_frame(
        &self,
        frame: &mut crate::display::Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), crate::view::Error> {
        let Some(state) = &self.bookmark_list else {
            return Ok(());
        };
        let Some((_, panel)) =
            composition.and_then(|c| c.panels().iter().find(|(id, _)| *id == state.view))
        else {
            return Ok(());
        };
        frame.bookmarks(
            crate::display::bookmarks::Display {
                rows: &state.rows,
                cursor: state.cursor,
                top: state.top,
                marked: &state.marked,
                stale: state.generation != self.bookmarks().generation(),
            },
            *panel,
        )
    }
}
