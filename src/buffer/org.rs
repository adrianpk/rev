use super::{Buffer, Error, Stamp};
use crate::org::Index;

impl Buffer {
    /// Optional derived state: ordinary buffers never start analysis implicitly.
    pub fn org_index(&self) -> Option<&Index> {
        self.org.as_ref()
    }

    /// Publish only observations of this exact canonical revision.
    pub fn install_org_index(&mut self, index: Index) -> Result<(), Error> {
        index.check(self)?;
        self.org = Some(index);
        Ok(())
    }

    /// Move retained offsets into owned analysis without cloning the index.
    pub fn take_org_index(&mut self) -> Option<Index> {
        self.org.take()
    }

    pub(super) fn edit_org(
        &mut self,
        before: Stamp,
        after: Stamp,
        edits: &[tree_sitter::InputEdit],
    ) {
        if let Some(index) = &mut self.org {
            index.edited(before, after, edits);
        }
    }
}
