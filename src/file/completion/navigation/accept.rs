use std::io;
use std::path::PathBuf;

use super::Navigation;

pub(crate) enum Acceptance {
    File(PathBuf),
    Directory(String),
}

impl Navigation {
    /// Return observes only the accepted identity. A visual default is not a
    /// choice: a freely typed draft may name a new file beside its suggestions.
    pub(crate) fn accept(&self) -> io::Result<Acceptance> {
        let path = self.accepted_path()?;
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_dir() => self.accept_directory(&path),
            Err(error) if self.chosen => Err(path_error(&path, error)),
            _ => Ok(Acceptance::File(path)),
        }
    }

    fn accept_directory(&self, path: &std::path::Path) -> io::Result<Acceptance> {
        // Validate access before changing the draft. Actual bounded enumeration
        // remains with the navigation owner after the prompt accepts its text.
        std::fs::read_dir(path).map_err(|error| path_error(path, error))?;
        Ok(Acceptance::Directory(self.directory_draft()?))
    }

    fn accepted_path(&self) -> io::Result<PathBuf> {
        if self.chosen {
            return self.candidates()[self.selected].path().map(PathBuf::from);
        }
        if self.draft.is_empty() {
            return Ok(PathBuf::new());
        }
        self.typed_path()
            .ok_or_else(|| io::Error::other(self.diagnostic().unwrap_or("Invalid file path")))
    }

    fn directory_draft(&self) -> io::Result<String> {
        let mut draft = self.draft.clone();
        if self.chosen {
            let candidates = self.candidates();
            let candidate = candidates[self.selected];
            candidate.path()?;
            let query = self.query.as_ref().expect("observed choice has a query");
            draft.truncate(draft.len() - query.prefix().len());
            draft.push_str(candidate.name().expect("supported candidate has a name"));
        }
        if !draft.ends_with('/') {
            draft.push('/');
        }
        Ok(draft)
    }
}

fn path_error(path: &std::path::Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("accept {path:?}: {error}"))
}
