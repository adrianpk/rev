use super::{Bookmarks, RecordId, Resolution};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Menu metadata has no source text, context copy or row-index authority.
pub(crate) struct Row {
    pub id: RecordId,
    pub target: u64,
    pub name: String,
    pub path: PathBuf,
    pub resolution: Resolution,
}
impl Bookmarks {
    pub(crate) fn rows(&self) -> Vec<Row> {
        self.iter()
            .map(|r| Row {
                id: r.id(),
                target: r.generation(),
                name: r.name().into(),
                path: r.path().into(),
                resolution: r.resolution(),
            })
            .collect()
    }
    pub(crate) fn delete_rows(
        &mut self,
        generation: u64,
        targets: &[(RecordId, u64)],
    ) -> Result<(), String> {
        let current: BTreeMap<_, _> = self.iter().map(|r| (r.id(), r.generation())).collect();
        if self.generation() != generation
            || targets
                .iter()
                .any(|(id, version)| current.get(id) != Some(version))
        {
            return Err("Bookmark collection changed; deletions were not applied".into());
        }
        if !targets.is_empty() {
            let removed: BTreeMap<_, _> = targets.iter().copied().collect();
            self.entries.retain(|_, r| !removed.contains_key(&r.id()));
            self.dirty = true;
            self.changed();
        }
        Ok(())
    }
}
