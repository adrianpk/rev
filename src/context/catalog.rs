use std::collections::BTreeSet;
use std::os::unix::fs::MetadataExt;

use super::*;

/// One selected session per registered owner. Enumeration does not create state.
#[derive(Clone, Debug, PartialEq)]
pub struct Catalog {
    pub revision: Revision,
    pub projects: Vec<Project>,
    pub workspaces: Vec<Workspace>,
    pub sessions: Vec<SavedSession>,
}
impl Default for Catalog {
    fn default() -> Self {
        Self {
            revision: Revision::INITIAL,
            projects: Vec::new(),
            workspaces: Vec::new(),
            sessions: Vec::new(),
        }
    }
}

/// Immutable candidate retains both expected state and preallocated identities.
/// Dropping it cancels preparation without touching the catalog or source files.
#[derive(Debug)]
pub struct Prepared {
    pub(crate) expected: Catalog,
    pub(crate) candidate: Catalog,
    pub(crate) bytes: Vec<u8>,
    observations: Vec<RootObservation>,
}
#[derive(Debug)]
struct RootObservation {
    path: NativePath,
    identity: Option<(u64, u64)>,
    require_directory: bool,
}
impl Prepared {
    pub fn candidate(&self) -> &Catalog {
        &self.candidate
    }
    pub(crate) fn check_roots(&self) -> Result<(), Error> {
        for observation in &self.observations {
            let metadata = std::fs::metadata(observation.path.as_path())
                .ok()
                .filter(|metadata| metadata.is_dir());
            if observation.require_directory && metadata.is_none() {
                return Err(Error::Unavailable("new or relocated root is unavailable"));
            }
            if let Some(expected) = observation.identity {
                let actual = metadata.map(|metadata| (metadata.dev(), metadata.ino()));
                if actual != Some(expected) {
                    return Err(Error::Conflict("prepared root identity changed"));
                }
            }
        }
        Ok(())
    }
}

impl Catalog {
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        super::codec::decode(bytes)
    }
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        super::codec::encode(self)
    }

    /// A complete multi-record update has one future publication point. All
    /// metadata/session/root revisions are checked before any file is staged.
    pub fn prepare(&self, mut candidate: Self) -> Result<Prepared, Error> {
        self.validate(false)?;
        if candidate.revision != self.revision {
            return Err(Error::Conflict("catalog revision"));
        }
        candidate.revision = self.revision.next()?;
        candidate.validate(false)?;
        self.check_changes(&candidate)?;
        let bytes = candidate.encode()?;
        let retained: BTreeSet<_> = self
            .roots()
            .map(|root| (root.id.clone(), root.path.clone()))
            .collect();
        let observations = candidate
            .roots()
            .map(|root| {
                let identity = std::fs::metadata(root.path.as_path())
                    .ok()
                    .filter(|metadata| metadata.is_dir())
                    .map(|metadata| (metadata.dev(), metadata.ino()));
                RootObservation {
                    path: root.path.clone(),
                    identity,
                    require_directory: !retained.contains(&(root.id.clone(), root.path.clone())),
                }
            })
            .collect();
        Ok(Prepared {
            expected: self.clone(),
            candidate,
            bytes,
            observations,
        })
    }

    fn roots(&self) -> impl Iterator<Item = &Root> {
        self.projects.iter().map(|project| &project.root).chain(
            self.workspaces
                .iter()
                .flat_map(|workspace| workspace.roots.iter()),
        )
    }

    pub fn session(&self, reference: &SessionRef) -> Option<&SavedSession> {
        self.sessions
            .iter()
            .find(|session| session.reference == *reference)
    }

    pub(crate) fn validate(&self, observe: bool) -> Result<(), Error> {
        if self.projects.len() > MAX_PROJECTS
            || self.workspaces.len() > MAX_WORKSPACES
            || self.sessions.len() != self.projects.len() + self.workspaces.len()
        {
            return Err(Error::Capacity("catalog records"));
        }
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for project in &self.projects {
            if !ids.insert(project.id.clone()) || !names.insert(project.name.clone()) {
                return Err(Error::Conflict("Project identity or name"));
            }
            validate_roots(std::slice::from_ref(&project.root), observe)?;
            let path = if observe {
                project.root.path.observed()
            } else {
                project.root.path.as_path().into()
            };
            if !paths.insert(path) {
                return Err(Error::Conflict("Project root"));
            }
            self.validate_owner(
                &Owner::Project(project.id.clone()),
                project.roots_revision,
                &project.session_ref,
                std::slice::from_ref(&project.root),
                observe,
            )?;
        }
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for workspace in &self.workspaces {
            if !ids.insert(workspace.id.clone()) || !names.insert(workspace.name.clone()) {
                return Err(Error::Conflict("Workspace identity or name"));
            }
            validate_roots(&workspace.roots, observe)?;
            self.validate_owner(
                &Owner::Workspace(workspace.id.clone()),
                workspace.roots_revision,
                &workspace.session_ref,
                &workspace.roots,
                observe,
            )?;
        }
        let mut owners = BTreeSet::new();
        for session in &self.sessions {
            if !owners.insert(session.reference.owner.clone()) {
                return Err(Error::Invalid("duplicate session owner"));
            }
        }
        Ok(())
    }

    fn validate_owner(
        &self,
        owner: &Owner,
        revision: Revision,
        reference: &SessionRef,
        roots: &[Root],
        observe: bool,
    ) -> Result<(), Error> {
        if reference.owner != *owner {
            return Err(Error::Invalid("session owner"));
        }
        let session = self
            .session(reference)
            .ok_or(Error::Invalid("session reference"))?;
        if session.roots_revision != revision {
            return Err(Error::Invalid("session membership revision"));
        }
        match &session.snapshot {
            Ok(snapshot) => validate_snapshot(snapshot, roots, observe),
            Err(_) if session.preserved.is_some() => Ok(()),
            Err(_) => Err(Error::Invalid("unpreserved damaged session")),
        }
    }

    fn check_changes(&self, next: &Self) -> Result<(), Error> {
        for project in &next.projects {
            let old = self.projects.iter().find(|old| old.id == project.id);
            let (previous, changed_roots) = match old {
                Some(old) => {
                    check_revision(old.revision, project.revision, old != project)?;
                    (
                        Some(old.roots_revision),
                        !same_membership(
                            std::slice::from_ref(&old.root),
                            std::slice::from_ref(&project.root),
                        ),
                    )
                }
                None => {
                    initial(project.revision)?;
                    (None, false)
                }
            };
            check_roots_revision(previous, project.roots_revision, changed_roots)?;
        }
        for workspace in &next.workspaces {
            let old = self.workspaces.iter().find(|old| old.id == workspace.id);
            let (previous, changed_roots) = match old {
                Some(old) => {
                    check_revision(old.revision, workspace.revision, old != workspace)?;
                    (
                        Some(old.roots_revision),
                        !same_membership(&old.roots, &workspace.roots),
                    )
                }
                None => {
                    initial(workspace.revision)?;
                    (None, false)
                }
            };
            check_roots_revision(previous, workspace.roots_revision, changed_roots)?;
        }
        for session in &next.sessions {
            match self
                .sessions
                .iter()
                .find(|old| old.reference.owner == session.reference.owner)
            {
                Some(old) => {
                    if session.snapshot.is_err() && old != session {
                        return Err(Error::Invalid("changed damaged session"));
                    }
                    check_revision(
                        old.reference.revision,
                        session.reference.revision,
                        old != session,
                    )?;
                }
                None => {
                    if session.snapshot.is_err() {
                        return Err(Error::Invalid("new damaged session"));
                    }
                    initial(session.reference.revision)?;
                }
            }
        }
        Ok(())
    }
}

fn initial(revision: Revision) -> Result<(), Error> {
    if revision != Revision::INITIAL {
        return Err(Error::Conflict("initial revision"));
    }
    Ok(())
}
fn check_revision(old: Revision, new: Revision, changed: bool) -> Result<(), Error> {
    if new != if changed { old.next()? } else { old } {
        return Err(Error::Conflict("record or session revision"));
    }
    Ok(())
}
fn check_roots_revision(old: Option<Revision>, new: Revision, changed: bool) -> Result<(), Error> {
    match old {
        Some(old) => check_revision(old, new, changed),
        None => initial(new),
    }
}
fn same_membership(left: &[Root], right: &[Root]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(a, b)| a.id == b.id && a.path == b.path)
}
pub(crate) fn validate_roots(roots: &[Root], observe: bool) -> Result<(), Error> {
    if roots.len() > MAX_ROOTS {
        return Err(Error::Capacity("Workspace roots"));
    }
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for root in roots {
        NativePath::absolute(root.path.as_path())?;
        let path = if observe {
            root.path.observed()
        } else {
            root.path.as_path().into()
        };
        if !ids.insert(root.id.clone()) || !paths.insert(path) {
            return Err(Error::Conflict("root identity or path"));
        }
    }
    Ok(())
}
pub(crate) fn validate_snapshot(
    snapshot: &Snapshot,
    roots: &[Root],
    observe: bool,
) -> Result<(), Error> {
    if snapshot.files.len() > MAX_FILES {
        return Err(Error::Capacity("session files"));
    }
    let mut references = BTreeSet::new();
    let mut physical = BTreeSet::new();
    for entry in &snapshot.files {
        entry.cursor.validate()?;
        if let Some(viewport) = entry.viewport {
            viewport.anchor.validate()?;
            if i64::try_from(viewport.horizontal_cells).is_err() {
                return Err(Error::Invalid("horizontal scroll"));
            }
        }
        match &entry.file {
            FileReference::LooseAbsolute(path) => {
                NativePath::absolute(path.as_path())?;
            }
            FileReference::RootRelative { root, path } => {
                NativePath::relative(path.as_path())?;
                if !roots.iter().any(|entry| entry.id == *root) {
                    return Err(Error::Invalid("session root reference"));
                }
            }
        }
        if !references.insert(&entry.file) {
            return Err(Error::Invalid("duplicate session file"));
        }
        if observe && !physical.insert(entry.file.resolve(roots)?) {
            return Err(Error::Conflict("canonical session file"));
        }
    }
    if snapshot
        .active_file
        .as_ref()
        .is_some_and(|file| !references.contains(file))
    {
        return Err(Error::Invalid("active session file"));
    }
    Ok(())
}
