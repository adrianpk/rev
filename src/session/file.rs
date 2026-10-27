use std::io;
use std::path::Path;

use crate::view::View;

use super::{Action, Pending, Session};

impl Session {
    /// Failed ingress changes only the error echo. Existing buffers, history,
    /// point, scroll and input context remain available without lossy replacement.
    pub fn load(&mut self, path: &Path) -> io::Result<()> {
        self.yank = None;
        self.kills.break_chain();
        let result = self.ingress(path);
        if let Err(error) = &result {
            self.error(&error.to_string());
        }
        result
    }

    fn ingress(&mut self, path: &Path) -> io::Result<()> {
        let visit = self.buffers.prepare_visit(path)?;
        let buffer = visit.buffer(&self.buffers).map_err(io::Error::other)?;
        let observed = buffer.file().map(|file| file.path().to_path_buf());
        let view = View::new(buffer);
        self.stop_typing().map_err(io::Error::other)?;
        // All recoverable work is complete before registry/view publication.
        let id = self.buffers.publish_visit(visit);
        if self.windows.selected().point.stamp.buffer != id {
            *self.windows.selected_mut() = view;
        }
        self.tree.edit();
        self.input.cancel();
        self.pending = None;
        self.echo_kind = crate::display::Message::Normal;
        self.echo.clear();
        if let Some(path) = observed
            && let Err(error) = self.tree.observe_path(&path)
        {
            self.echo = format!("Opened {}; tree refresh failed: {error}", path.display());
            self.echo_kind = crate::display::Message::Warning;
        }
        Ok(())
    }

    pub(super) fn find_file(&mut self) -> Action {
        self.start_path()
    }

    pub(super) fn argument(&mut self, path: String) -> Action {
        match self.pending.take() {
            Some(Pending::Path(navigation)) => self.open_draft(*navigation, &path),
            Some(Pending::TreeRename(rename)) => self.tree_renamed(rename, &path),
            Some(Pending::TreeCreate(creation)) => self.tree_created(creation, &path),
            Some(Pending::Replace(state)) => self.replace_argument(*state, path),
            Some(Pending::Write { buffer, after }) => {
                self.save_to(buffer, Some(Path::new(&path)), after)
            }
            Some(Pending::Buffer { command, default }) => {
                self.buffer_argument(command, default, &path)
            }
            _ => self.message("No file argument is pending"),
        }
    }
}
