use super::resolution::fingerprint;
use super::*;

/// Immutable explanation data has no resolution or sequence-control authority.
#[derive(Debug)]
pub struct ConflictExplanation {
    pub conflict: Arc<Conflict>,
    pub saved: Result<Option<SavedResult>, String>,
    pub(in crate::git) index: [u64; 8],
    pub(in crate::git) _admission: Lease,
}
impl ConflictExplanation {
    /// Currency is conservative: changed index/file metadata invalidates the
    /// historical source. Every changing request still needs exact Git proof.
    pub fn current(&self) -> Result<(), String> {
        let identity = &self.conflict.sequence.refs.observation.identity;
        let index = std::fs::symlink_metadata(identity.git_directory.join("index"))
            .map_err(|_| "Conflict index source is unavailable")?;
        if !index.is_file() || fingerprint(&index) != self.index {
            return Err("Conflict index changed; explanation is a historical snapshot".into());
        }
        let root = identity
            .worktree
            .as_ref()
            .ok_or("Conflict worktree source is unavailable")?;
        let actual = std::fs::symlink_metadata(root.join(&self.conflict.change.path));
        match (&self.saved, actual) {
            (Ok(Some(saved)), Ok(m)) if m.is_file() && fingerprint(&m) == saved.fingerprint => {
                Ok(())
            }
            (Ok(None), Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            (Err(_), _) => {
                Err("Working result was unavailable in the captured conflict context".into())
            }
            _ => Err("Conflict saved result changed; explanation is a historical snapshot".into()),
        }
    }
}
impl Repositories {
    pub fn explain_conflict(&mut self, conflict: Arc<Conflict>) -> Result<OperationId, String> {
        self.check_conflict(&conflict)?;
        let context = conflict.sequence.refs.observation.context;
        let root = self.mutation_identity_root(&conflict.sequence.refs.observation.identity)?;
        self.start(
            root,
            Some(context),
            work::Request::ConflictExplain(conflict),
        )
    }
}
