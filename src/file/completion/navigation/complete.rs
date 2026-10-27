use std::io;

use unicode_segmentation::UnicodeSegmentation;

use super::{Candidate, Navigation};

impl Navigation {
    /// Explicit Tab refreshes the component and completes a selected/unique name
    /// or a whole-grapheme shared prefix. It never opens a file. Incomplete lists
    /// require an explicit choice rather than implying uniqueness from a subset.
    pub fn complete(&mut self) -> io::Result<Option<String>> {
        self.refresh_choice()?;
        let candidates = self.candidates();
        if candidates.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "No matching paths"));
        }
        if !self.chosen && self.diagnostic().is_some() {
            return Err(io::Error::other(super::super::INCOMPLETE_MESSAGE));
        }
        let Some(completed) = self.completed_name(&candidates)? else {
            return Ok(None);
        };
        let query = self
            .query
            .as_ref()
            .expect("candidate observation has a query");
        let parent = &self.draft[..self.draft.len() - query.prefix().len()];
        Ok(Some(format!("{parent}{completed}")))
    }

    fn completed_name(&self, candidates: &[&Candidate]) -> io::Result<Option<String>> {
        if self.chosen || candidates.len() == 1 {
            return selected_name(candidates[self.selected]).map(Some);
        }
        let shared = shared_name(candidates)?;
        let query = self
            .query
            .as_ref()
            .expect("candidate observation has a query");
        Ok((shared.len() > query.prefix().len()).then_some(shared))
    }

    fn refresh_choice(&mut self) -> io::Result<()> {
        let selected = if self.chosen {
            self.candidates()
                .get(self.selected)
                .map(|entry| entry.name().map(str::to_owned).ok_or_else(unsupported))
                .transpose()?
        } else {
            None
        };
        let draft = self.draft.clone();
        self.update(&draft, true);
        if let Some(error) = &self.error {
            return Err(io::Error::other(error.clone()));
        }
        if let Some(name) = selected {
            match self
                .candidates()
                .iter()
                .position(|entry| entry.name() == Some(&name))
            {
                Some(index) => self.selected = index,
                None => {
                    self.chosen = false;
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Selected path is no longer present",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn selected_name(candidate: &Candidate) -> io::Result<String> {
    candidate.path()?;
    let name = candidate.name().expect("supported candidate has a name");
    Ok(format!(
        "{name}{}",
        if candidate.is_directory() { "/" } else { "" }
    ))
}

fn shared_name(candidates: &[&Candidate]) -> io::Result<String> {
    let mut names = candidates
        .iter()
        .map(|candidate| candidate.name().ok_or_else(unsupported));
    let mut prefix = names.next().expect("nonempty candidate list")?.to_owned();
    for name in names {
        prefix = prefix
            .graphemes(true)
            .zip(name?.graphemes(true))
            .take_while(|(left, right)| left == right)
            .map(|(cluster, _)| cluster)
            .collect();
    }
    Ok(prefix)
}

fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "unsupported non-UTF-8 filename")
}
