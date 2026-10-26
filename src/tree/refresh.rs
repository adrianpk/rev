use std::collections::BTreeSet;
use std::io;

use super::{Entry, Tree};

impl Tree {
    /// Reload only previously expanded branches, iteratively. New directories
    /// stay collapsed. A failed branch collapses while other observations stay
    /// usable; selection falls back to the nearest remaining visible ancestor.
    pub fn refresh(&mut self) -> io::Result<()> {
        let Some(root) = self.root.clone() else {
            return Ok(());
        };
        let selected = self
            .selected()
            .map(|entry| entry.path.clone())
            .unwrap_or_else(|| root.clone());
        let expanded: BTreeSet<_> = self
            .entries
            .iter()
            .filter(|entry| entry.expanded)
            .map(|entry| entry.path.clone())
            .collect();
        self.entries.clear();
        self.selected = 0;
        self.top = 0;
        let observed = Entry::observe(root.clone(), 0);
        self.entries.push(Entry::root(root));
        self.entries[0] = observed?;
        let result = self.refresh_expanded(&expanded);
        self.selected = selected
            .ancestors()
            .find_map(|path| self.entries.iter().position(|entry| entry.path == path))
            .unwrap_or(0);
        result
    }

    fn refresh_expanded(&mut self, expanded: &BTreeSet<std::path::PathBuf>) -> io::Result<()> {
        let mut first_error = None;
        let mut index = 0;
        while index < self.entries.len() {
            let entry = &self.entries[index];
            if entry.directory && expanded.contains(&entry.path) {
                match entry.children() {
                    Ok(children) => {
                        self.entries[index].expanded = true;
                        self.entries.splice(index + 1..index + 1, children);
                    }
                    Err(error) => {
                        first_error.get_or_insert(error);
                    }
                }
            }
            index += 1;
        }
        first_error.map_or(Ok(()), Err)
    }
}
