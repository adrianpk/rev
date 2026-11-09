use super::super::record::Record;
use crate::context::{self, Catalog, Name, NativePath, RootId, SavedSession, Snapshot, Workspace};

#[derive(Clone)]
pub(super) enum Mutation {
    Rename(Name),
    Forget,
    Relocate {
        root: RootId,
        path: NativePath,
    },
    Workspace {
        destination: Workspace,
        consume: bool,
    },
}
#[derive(Clone)]
pub(in crate::session) struct Change {
    pub(super) record: Record,
    pub(super) mutation: Mutation,
}
impl Change {
    pub(in crate::session::contexts) fn forgets(&self, active: &context::Context) -> bool {
        self.record.context() == *active && matches!(self.mutation, Mutation::Forget)
    }
    pub(super) fn leaves(&self, active: &context::Context) -> bool {
        match &self.mutation {
            Mutation::Workspace { consume: false, .. } => true,
            Mutation::Relocate { .. } | Mutation::Forget => {
                self.record.context() == *active
                    && !(matches!(self.mutation, Mutation::Forget)
                        && matches!(self.record, Record::Project(_)))
            }
            _ => false,
        }
    }
    pub(super) fn captures(&self, active: &context::Context) -> bool {
        self.record.context() == *active
            && matches!(
                self.mutation,
                Mutation::Workspace { .. } | Mutation::Relocate { .. }
            )
    }
    pub(super) fn destination(&self) -> Option<Record> {
        match &self.mutation {
            Mutation::Workspace { destination, .. } => Some(Record::Workspace(destination.clone())),
            Mutation::Relocate { .. } => Some(self.record.clone()),
            _ => None,
        }
    }
    pub(in crate::session::contexts) fn apply(
        &self,
        catalog: &mut Catalog,
        captured: Option<Snapshot>,
    ) -> Result<(), String> {
        if !self.record.matches(catalog) {
            return Err("Selected context changed; reopen its picker".into());
        }
        match &self.mutation {
            Mutation::Forget => remove(catalog, &self.record),
            Mutation::Rename(name) => self.rename(catalog, name)?,
            Mutation::Relocate { root, path } => self.relocate(catalog, root, path, captured)?,
            Mutation::Workspace {
                destination,
                consume,
            } => self.convert_workspace(catalog, destination, *consume, captured)?,
        }
        Ok(())
    }
    fn convert_workspace(
        &self,
        catalog: &mut Catalog,
        destination: &Workspace,
        consume: bool,
        captured: Option<Snapshot>,
    ) -> Result<(), String> {
        let mut snapshot = captured
            .map(Ok)
            .unwrap_or_else(|| snapshot(catalog, &self.record))?;
        let old = &self.record.roots()[0].id;
        let new = &destination.roots[0].id;
        for entry in &mut snapshot.files {
            remap(&mut entry.file, old, new);
        }
        if let Some(active) = &mut snapshot.active_file {
            remap(active, old, new);
        }
        if consume {
            remove(catalog, &self.record);
        }
        Record::Workspace(destination.clone()).insert(catalog, snapshot)?;
        Ok(())
    }

    pub(super) fn relocation_conflicts(&self, catalog: &Catalog) -> bool {
        let Mutation::Relocate { root, path } = &self.mutation else {
            return false;
        };
        match &self.record {
            Record::Project(p) => catalog.projects.iter().any(|other| {
                other.id != p.id
                    && super::super::projects::same_directory(
                        other.root.path.as_path(),
                        path.as_path(),
                    )
            }),
            Record::Workspace(w) => w.roots.iter().any(|other| {
                other.id != *root
                    && super::super::projects::same_directory(other.path.as_path(), path.as_path())
            }),
        }
    }

    pub(super) fn observations(&self) -> Option<Vec<(std::path::PathBuf, (u64, u64))>> {
        let changed_roots: Vec<_> = match &self.mutation {
            Mutation::Relocate { path, .. } => vec![path],
            Mutation::Workspace { destination, .. } => {
                destination.roots.iter().map(|r| &r.path).collect()
            }
            _ => Vec::new(),
        };
        changed_roots
            .into_iter()
            .map(|path| {
                super::super::workspaces::observe(path.as_path())
                    .map(|i| (path.as_path().to_owned(), i))
            })
            .collect()
    }

    fn relocate(
        &self,
        catalog: &mut Catalog,
        root: &RootId,
        path: &NativePath,
        captured: Option<Snapshot>,
    ) -> Result<(), String> {
        let mut next = self.record.clone();
        let (revision, roots_revision, reference, roots) = match &mut next {
            Record::Project(p) => (
                &mut p.revision,
                &mut p.roots_revision,
                &mut p.session_ref,
                std::slice::from_mut(&mut p.root),
            ),
            Record::Workspace(w) => (
                &mut w.revision,
                &mut w.roots_revision,
                &mut w.session_ref,
                w.roots.as_mut_slice(),
            ),
        };
        let selected = roots
            .iter_mut()
            .find(|r| r.id == *root)
            .ok_or("Selected root disappeared")?;
        selected.path = path.clone();
        *revision = revision.next().map_err(|e| e.to_string())?;
        *roots_revision = roots_revision.next().map_err(|e| e.to_string())?;
        reference.revision = reference.revision.next().map_err(|e| e.to_string())?;
        let snapshot = captured
            .map(Ok)
            .unwrap_or_else(|| snapshot(catalog, &self.record))?;
        let session = catalog
            .sessions
            .iter_mut()
            .find(|s| s.reference == *self.record.session_ref())
            .ok_or("Context session disappeared")?;
        *session = SavedSession::new(reference.clone(), *roots_revision, snapshot);
        replace(catalog, next);
        Ok(())
    }

    fn rename(&self, catalog: &mut Catalog, name: &Name) -> Result<(), String> {
        let mut next = self.record.clone();
        match &mut next {
            Record::Project(p) => {
                p.name = name.clone();
                p.revision = p.revision.next().map_err(|e| e.to_string())?;
            }
            Record::Workspace(w) => {
                w.name = name.clone();
                w.revision = w.revision.next().map_err(|e| e.to_string())?;
            }
        }
        replace(catalog, next);
        Ok(())
    }
}
fn snapshot(catalog: &Catalog, record: &Record) -> Result<Snapshot, String> {
    catalog
        .session(record.session_ref())
        .ok_or("Context session disappeared")?
        .snapshot
        .clone()
        .map_err(|e| format!("Damaged context session: {e}; saved bytes retained"))
}
fn remap(file: &mut context::FileReference, old: &RootId, new: &RootId) {
    if let context::FileReference::RootRelative { root, .. } = file
        && root == old
    {
        *root = new.clone();
    }
}
fn remove(catalog: &mut Catalog, record: &Record) {
    match record {
        Record::Project(p) => catalog.projects.retain(|r| r.id != p.id),
        Record::Workspace(w) => catalog.workspaces.retain(|r| r.id != w.id),
    }
    catalog
        .sessions
        .retain(|s| s.reference.owner != record.session_ref().owner);
}
fn replace(catalog: &mut Catalog, record: Record) {
    match record {
        Record::Project(p) => {
            let target = catalog
                .projects
                .iter_mut()
                .find(|r| r.id == p.id)
                .expect("selected Project checked");
            *target = p
        }
        Record::Workspace(w) => {
            let target = catalog
                .workspaces
                .iter_mut()
                .find(|r| r.id == w.id)
                .expect("selected Workspace checked");
            *target = w
        }
    }
}
