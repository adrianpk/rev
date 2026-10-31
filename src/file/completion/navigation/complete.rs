use std::io;

use super::Navigation;

impl Navigation {
    /// Explicit Tab refreshes and fills the highlighted candidate, including
    /// the default first row. A bounded subset is never treated as uniqueness.
    /// Completion neither accepts a root nor opens a file.
    pub fn complete(&mut self) -> io::Result<Option<String>> {
        let draft = self.draft.clone();
        self.update(&draft, true);
        if let Some(error) = &self.error {
            return Err(io::Error::other(error.clone()));
        }
        let candidates = self.candidates();
        let candidate = candidates
            .get(self.selected)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No matching paths"))?;
        candidate.path()?;
        let name = candidate.name().expect("supported candidate has a name");
        let query = self
            .query
            .as_ref()
            .expect("candidate observation has a query");
        let parent = &self.draft[..self.draft.len() - query.prefix().len()];
        Ok(Some(format!(
            "{parent}{name}{}",
            if candidate.is_directory() { "/" } else { "" }
        )))
    }
}
