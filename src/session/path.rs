use std::io;
use std::path::PathBuf;

use crate::file::completion::{Acceptance, Navigation};
use crate::input::Selection;

use super::{Action, Pending, Session};

impl Session {
    pub(super) fn start_path(&mut self) -> Action {
        let base = match self.path_base() {
            Ok(base) => base,
            Err(error) => return self.error(&error.to_string()),
        };
        if let Err(error) = self.input.path_argument() {
            return self.error(&error.to_string());
        }
        let home = std::env::var_os("HOME").map(PathBuf::from);
        self.pending = Some(Pending::Path(Box::new(Navigation::new(base, home))));
        self.message("")
    }

    fn path_base(&self) -> io::Result<PathBuf> {
        let file = self.buffer().map_err(io::Error::other)?.file();
        match file.and_then(|file| file.path().parent()) {
            Some(parent) => Ok(parent.to_owned()),
            None => std::env::current_dir(),
        }
    }

    pub(super) fn path_navigation(&self) -> Option<&Navigation> {
        match &self.pending {
            Some(Pending::Path(path)) => Some(path),
            _ => None,
        }
    }

    pub(super) fn sync_path(&mut self) {
        if let (Some(Pending::Path(path)), Some(prompt)) = (&mut self.pending, self.input.prompt())
        {
            path.update(prompt.text(), false);
        }
    }

    pub(super) fn reveal_paths(&mut self, rows: u16) {
        if let Some(Pending::Path(path)) = &mut self.pending {
            path.reveal(rows);
        }
    }

    pub(super) fn select_candidate(&mut self, selection: Selection) -> Action {
        if let Some(Pending::Path(path)) = &mut self.pending {
            path.navigate(selection == Selection::Previous);
            return Action::Redraw;
        }
        self.select_buffer(selection)
    }

    pub(super) fn accept_path(&mut self) -> Action {
        self.input.suppress_repeat();
        let Some(navigation) = self.path_navigation() else {
            return self.message("No path acceptance is pending");
        };
        match navigation.accept() {
            Ok(Acceptance::Directory(draft)) => return self.descend_path(&draft),
            Ok(Acceptance::File(path)) => {
                let _ = self.load(&path);
            }
            Err(error) => return self.error(&error.to_string()),
        }
        Action::Redraw
    }

    fn descend_path(&mut self, draft: &str) -> Action {
        if let Err(error) = self.input.complete(draft) {
            return self.notice(&error.to_string());
        }
        self.sync_path();
        self.message("")
    }

    pub(super) fn complete_path(&mut self) -> Action {
        let Some(Pending::Path(path)) = &mut self.pending else {
            return self.message("No path completion is pending");
        };
        let completed = match path.complete() {
            Ok(completed) => completed,
            Err(error) => return self.notice(&error.to_string()),
        };
        if let Some(completed) = completed
            && let Err(error) = self.input.complete(&completed)
        {
            return self.notice(&error.to_string());
        }
        self.sync_path();
        self.message("")
    }
}
