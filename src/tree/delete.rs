use std::io;

use crate::file::Deletion;

use super::Tree;

impl Tree {
    pub fn prepare_delete(&self) -> io::Result<Deletion> {
        let entry = self
            .selected()
            .ok_or_else(|| io::Error::other("no tree selection"))?;
        Deletion::capture(entry.path())
    }
}
