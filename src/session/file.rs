use std::io;
use std::path::Path;

use crate::command::Command;
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
            self.echo = error.to_string();
        }
        result
    }

    fn ingress(&mut self, path: &Path) -> io::Result<()> {
        let visit = self.buffers.prepare_visit(path)?;
        let view = View::new(visit.buffer(&self.buffers).map_err(io::Error::other)?);
        self.stop_typing().map_err(io::Error::other)?;
        // All recoverable work is complete before registry/view publication.
        let id = self.buffers.publish_visit(visit);
        if self.windows.selected().point.stamp.buffer != id {
            *self.windows.selected_mut() = view;
        }
        self.input.cancel();
        self.pending = None;
        self.echo.clear();
        Ok(())
    }

    pub(super) fn find_file(&mut self) -> Action {
        self.file_argument(Command::FindFile, "Find file: ")
    }

    pub(super) fn file_argument(&mut self, command: Command, label: &'static str) -> Action {
        match self.input.argument(label, "") {
            Ok(()) => {
                self.pending = Some(Pending::Argument(command));
                self.message("")
            }
            Err(error) => self.message(&error.to_string()),
        }
    }

    pub(super) fn argument(&mut self, path: String) -> Action {
        match self.pending.take() {
            Some(Pending::Argument(Command::FindFile)) => {
                let _ = self.load(Path::new(&path));
                Action::Redraw
            }
            Some(Pending::Write { buffer, close }) => {
                self.save_to(buffer, Some(Path::new(&path)), close)
            }
            Some(Pending::Buffer { command, default }) => {
                self.buffer_argument(command, default, &path)
            }
            _ => self.message("No file argument is pending"),
        }
    }
}
