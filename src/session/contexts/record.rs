use crate::context::{
    Catalog, Context, Project, Root, SavedSession, SessionRef, Snapshot, Workspace,
};

/// One persistent context owner. A Workspace contains independent roots and
/// never borrows Project registration or session authority through provenance.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::session) enum Record {
    Project(Project),
    Workspace(Workspace),
}
impl Record {
    pub(in crate::session) fn context(&self) -> Context {
        match self {
            Self::Project(p) => Context::Project(p.id.clone()),
            Self::Workspace(w) => Context::Workspace(w.id.clone()),
        }
    }
    pub(super) fn kind(&self) -> &'static str {
        match self {
            Self::Project(_) => "Project",
            Self::Workspace(_) => "Workspace",
        }
    }
    pub(in crate::session) fn name(&self) -> &str {
        match self {
            Self::Project(p) => p.name.as_str(),
            Self::Workspace(w) => w.name.as_str(),
        }
    }
    pub(in crate::session) fn roots(&self) -> &[Root] {
        match self {
            Self::Project(p) => std::slice::from_ref(&p.root),
            Self::Workspace(w) => &w.roots,
        }
    }
    pub(in crate::session) fn session_ref(&self) -> &SessionRef {
        match self {
            Self::Project(p) => &p.session_ref,
            Self::Workspace(w) => &w.session_ref,
        }
    }
    pub(in crate::session) fn roots_revision(&self) -> crate::context::Revision {
        match self {
            Self::Project(p) => p.roots_revision,
            Self::Workspace(w) => w.roots_revision,
        }
    }
    pub(in crate::session) fn current(&self, catalog: &Catalog) -> Option<Self> {
        match self {
            Self::Project(p) => catalog
                .projects
                .iter()
                .find(|n| n.id == p.id)
                .cloned()
                .map(Self::Project),
            Self::Workspace(w) => catalog
                .workspaces
                .iter()
                .find(|n| n.id == w.id)
                .cloned()
                .map(Self::Workspace),
        }
    }
    pub(in crate::session) fn matches(&self, catalog: &Catalog) -> bool {
        self.current(catalog).as_ref() == Some(self)
    }
    pub(in crate::session) fn insert(
        self,
        catalog: &mut Catalog,
        snapshot: Snapshot,
    ) -> Result<(), String> {
        if self.current(catalog).is_some() {
            return Err("Context identity is already registered".into());
        }
        catalog.sessions.push(SavedSession::new(
            self.session_ref().clone(),
            self.roots_revision(),
            snapshot,
        ));
        match self {
            Self::Project(p) => catalog.projects.push(p),
            Self::Workspace(w) => catalog.workspaces.push(w),
        }
        Ok(())
    }
    pub(in crate::session) fn checkpoint(
        &self,
        catalog: &mut Catalog,
        snapshot: Snapshot,
        membership: Option<&Workspace>,
    ) -> Result<(), String> {
        if !self.matches(catalog) {
            return Err("Active context revision changed".into());
        }
        let next = self.checkpoint_metadata(catalog, membership)?;
        let session = catalog
            .sessions
            .iter_mut()
            .find(|s| s.reference == *self.session_ref())
            .ok_or("Active context session disappeared")?;
        *session = SavedSession::new(next.session_ref().clone(), next.roots_revision(), snapshot);
        Ok(())
    }
    fn checkpoint_metadata(
        &self,
        catalog: &mut Catalog,
        membership: Option<&Workspace>,
    ) -> Result<Self, String> {
        let mut next = self.clone();
        match &mut next {
            Self::Project(p) => {
                p.revision = p.revision.next().map_err(|e| e.to_string())?;
                p.session_ref.revision =
                    p.session_ref.revision.next().map_err(|e| e.to_string())?;
                *catalog
                    .projects
                    .iter_mut()
                    .find(|n| n.id == p.id)
                    .ok_or("Project disappeared")? = p.clone();
            }
            Self::Workspace(w) => {
                if let Some(membership) = membership {
                    w.roots = membership.roots.clone();
                    w.roots_revision = w.roots_revision.next().map_err(|e| e.to_string())?;
                }
                w.revision = w.revision.next().map_err(|e| e.to_string())?;
                w.session_ref.revision =
                    w.session_ref.revision.next().map_err(|e| e.to_string())?;
                *catalog
                    .workspaces
                    .iter_mut()
                    .find(|n| n.id == w.id)
                    .ok_or("Workspace disappeared")? = w.clone();
            }
        }
        Ok(next)
    }
}
