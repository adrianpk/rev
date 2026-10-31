mod list;
mod navigation;
mod ordered;
pub(super) use list::List;
pub(super) use ordered::Iteration;
mod markers;
mod persistence;
mod quit;
pub(super) use navigation::Navigation;
pub(super) use persistence::{Persistence, Reload};

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
    pub fn bookmarks(&self) -> crate::bookmark::Collection<'_> {
        self.buffers.bookmarks()
    }

    pub(super) fn bookmark_command(&mut self, command: BookmarkCommand) -> Action {
        if let Some(action) = self.bookmark_action(command) {
            return action;
        }
        let notice = self.ensure_bookmarks().err().unwrap_or_default();
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
                .filter(|name| !name.chars().any(char::is_control))
                .unwrap_or("")
                .to_owned()
        } else {
            if self.buffers.bookmarks().is_empty() {
                return if notice.is_empty() {
                    self.message("No bookmarks in this session")
                } else {
                    self.error(&notice)
                };
            }
            String::new()
        };
        let prompt = Prompt {
            command,
            view: self.windows.selected_id(),
            point,
            default,
        };
        self.bookmark_prompt(prompt, "", &notice)
    }

    fn bookmark_action(&mut self, command: BookmarkCommand) -> Option<Action> {
        Some(match command {
            BookmarkCommand::Markers => self.toggle_bookmark_markers(),
            BookmarkCommand::List => self.list_bookmarks(),
            BookmarkCommand::Next => self.step_bookmark(false),
            BookmarkCommand::Previous => self.step_bookmark(true),
            _ => return self.bookmark_storage_action(command),
        })
    }
    fn bookmark_storage_action(&mut self, command: BookmarkCommand) -> Option<Action> {
        Some(match command {
            BookmarkCommand::MenuDelete
            | BookmarkCommand::MenuUnmark
            | BookmarkCommand::MenuExecute
            | BookmarkCommand::MenuVisit => self.message("This command requires the bookmark list"),
            BookmarkCommand::Save => match self.save_bookmarks() {
                Ok(()) => self.message("Bookmarks saved"),
                Err(error) => self.error(&error),
            },
            BookmarkCommand::Reload => self.reload_bookmarks(),
            _ => return None,
        })
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
            _ => {
                unreachable!("storage commands have no name argument")
            }
        };
        let names = self
            .buffers
            .bookmarks()
            .iter()
            .map(|record| record.name().to_owned())
            .collect();
        if let Err(error) = self.input.names_argument(&label, draft, names) {
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
        if prompt.command == BookmarkCommand::Jump {
            let name = name.to_owned();
            return match self.visit_bookmark(&name) {
                Ok(()) => Action::Redraw,
                Err(error) => self.bookmark_prompt(prompt, draft, &error),
            };
        }
        match self.apply_bookmark_argument(prompt.command, name) {
            Ok(message) => self.bookmark_change_message(message),
            Err(error) => self.bookmark_prompt(prompt, draft, &error),
        }
    }

    fn apply_bookmark_argument(
        &mut self,
        command: BookmarkCommand,
        name: &str,
    ) -> Result<&'static str, String> {
        match command {
            BookmarkCommand::Set | BookmarkCommand::SetNoOverwrite => {
                self.set_bookmark(name, command == BookmarkCommand::Set)?;
                Ok("Bookmark set")
            }
            BookmarkCommand::Delete => {
                self.buffers.bookmarks_mut().delete(name)?;
                Ok("Bookmark deleted")
            }
            BookmarkCommand::Jump => unreachable!("jump owns its cancellable navigation"),
            _ => unreachable!("storage commands have no name argument"),
        }
    }

    fn set_bookmark(&mut self, name: &str, overwrite: bool) -> Result<(), String> {
        let point = self.view().point;
        let path = Bookmarks::target(self.buffer().map_err(|e| e.to_string())?, point)?;
        let context = crate::bookmark::Context::capture_with(
            self.buffer().map_err(|e| e.to_string())?,
            point.position,
            self.buffers.snapshot().settings().bookmarks.context_chars,
        )
        .map_err(|e| e.to_string())?;
        self.buffers.bookmarks_mut().set(
            name,
            path,
            crate::bookmark::Captured::new(point, context),
            overwrite,
        )?;
        Ok(())
    }
}
