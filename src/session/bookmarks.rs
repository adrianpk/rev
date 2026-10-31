use crate::bookmark::Bookmarks;
use crate::buffer::edit::Point;
use crate::command::BookmarkCommand;
use crate::window::ViewId;

use super::{Action, Pending, Session};

pub(super) struct Prompt {
    command: BookmarkCommand,
    view: ViewId,
    point: Point,
    default: String,
}

impl Session {
    /// Read-only observation of this session's canonical bookmark collection.
    pub fn bookmarks(&self) -> &Bookmarks {
        &self.bookmarks
    }

    pub(super) fn bookmark_command(&mut self, command: BookmarkCommand) -> Action {
        let point = self.view().point;
        let setting = matches!(
            command,
            BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite
        );
        let default = if setting {
            let buffer = match self.buffer() {
                Ok(buffer) => buffer,
                Err(e) => return self.error(&e.to_string()),
            };
            if let Err(error) = Bookmarks::target(buffer, point) {
                return self.error(&error);
            }
            buffer
                .file()
                .and_then(|file| file.basename().to_str())
                .unwrap_or("")
                .to_owned()
        } else {
            if self.bookmarks.is_empty() {
                return self.message("No bookmarks in this session");
            }
            String::new()
        };
        let prompt = Prompt {
            command,
            view: self.windows.selected_id(),
            point,
            default,
        };
        self.bookmark_prompt(prompt, "", "")
    }

    fn bookmark_prompt(&mut self, prompt: Prompt, draft: &str, notice: &str) -> Action {
        let label = match prompt.command {
            BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite
                if !prompt.default.is_empty() =>
            {
                format!("Set bookmark (default {}): ", prompt.default)
            }
            BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite => "Set bookmark: ".into(),
            BookmarkCommand::Jump => "Jump to bookmark: ".into(),
            BookmarkCommand::Delete => "Delete bookmark: ".into(),
        };
        if let Err(error) = self.input.argument(&label, draft) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::Bookmark(prompt));
        self.message(notice)
    }

    pub(super) fn bookmark_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if self.windows.selected_id() != prompt.view || self.view().point != prompt.point {
            return self.error("Bookmark origin changed; nothing applied");
        }
        let name = if draft.is_empty() {
            prompt.default.as_str()
        } else {
            draft
        };
        let result = match prompt.command {
            BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite => {
                self.set_bookmark(name, prompt.command == BookmarkCommand::Set)
            }
            BookmarkCommand::Jump => self.visit_bookmark(name),
            BookmarkCommand::Delete => self.bookmarks.delete(name),
        };
        match result {
            Ok(()) => self.message(match prompt.command {
                BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite => "Bookmark set",
                BookmarkCommand::Jump => "Bookmark destination",
                BookmarkCommand::Delete => "Bookmark deleted",
            }),
            Err(error) => self.bookmark_prompt(prompt, draft, &error),
        }
    }

    fn set_bookmark(&mut self, name: &str, overwrite: bool) -> Result<(), String> {
        let point = self.view().point;
        let path = Bookmarks::target(self.buffer().map_err(|e| e.to_string())?, point)?;
        self.bookmarks.set(name, path, point, overwrite)?;
        Ok(())
    }

    fn visit_bookmark(&mut self, name: &str) -> Result<(), String> {
        let record = self
            .bookmarks
            .get(name)
            .ok_or("No bookmark with that name")?;
        let point = record.point();
        let visit = self
            .buffers
            .prepare_visit(record.path())
            .map_err(|e| e.to_string())?;
        let buffer = visit.buffer(&self.buffers).map_err(|e| e.to_string())?;
        if buffer.stamp() != point.stamp {
            return Err("Bookmark target is stale; set it again at a current location".into());
        }
        let tree = self
            .tree
            .opening_file(record.path())
            .map_err(|e| e.to_string())?;
        let mut view = self.view().clone();
        if view.point.stamp.buffer != buffer.id() {
            view.rebind(buffer);
        }
        view.point = point;
        view.check_point(buffer).map_err(|e| e.to_string())?;
        view.reset_navigation();
        view.reveal(buffer, self.pane_size())
            .map_err(|e| e.to_string())?;
        self.stop_typing().map_err(|e| e.to_string())?;
        self.buffers.publish_visit(visit);
        *self.windows.selected_mut() = view;
        if let Some(tree) = tree {
            self.tree = tree;
        }
        self.tree.edit();
        self.sync_input_context();
        Ok(())
    }
}
