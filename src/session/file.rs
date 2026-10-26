use std::io;
use std::path::Path;

use crate::command::Command;
use crate::view::View;

use super::{Action, Session};

impl Session {
    /// Failed ingress changes only the error echo. Existing buffers, history,
    /// point, scroll and input context remain available without lossy replacement.
    pub fn load(&mut self, path: &Path) -> io::Result<()> {
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
        if self.view.point.stamp.buffer != id {
            self.view = view;
        }
        self.input.cancel();
        self.pending = None;
        self.echo.clear();
        Ok(())
    }

    pub(super) fn find_file(&mut self) -> Action {
        match self.input.argument("Find file: ", "") {
            Ok(()) => {
                self.pending = Some(Command::FindFile);
                self.message("")
            }
            Err(error) => self.message(&error.to_string()),
        }
    }

    pub(super) fn argument(&mut self, path: String) -> Action {
        if self.pending.take() != Some(Command::FindFile) {
            return self.message("No file argument is pending");
        }
        // Input emits an argument only for the implemented find-file owner.
        // Completion consumes Enter; a failed visit leaves editing available.
        let _ = self.load(Path::new(&path));
        Action::Redraw
    }
}
