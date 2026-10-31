use std::io;
use std::path::PathBuf;

use super::Navigation;

impl Navigation {
    /// Return observes only the accepted identity. A visual default is not a
    /// choice: a freely typed draft may name a new file beside its suggestions.
    pub(crate) fn accept(&self) -> io::Result<PathBuf> {
        let path = self.accepted_path()?;
        if self.chosen {
            std::fs::metadata(&path).map_err(|error| path_error(&path, error))?;
        }
        Ok(path)
    }

    fn accepted_path(&self) -> io::Result<PathBuf> {
        if self.chosen {
            return self.candidates()[self.selected].path().map(PathBuf::from);
        }
        self.typed_path()
            .ok_or_else(|| io::Error::other(self.diagnostic().unwrap_or("Invalid file path")))
    }
}

fn path_error(path: &std::path::Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("accept {path:?}: {error}"))
}
