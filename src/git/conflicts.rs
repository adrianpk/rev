//! Immutable exact conflict sides. Git stage numbers are retained independently
//! of operation-relative labels; display text never supplies path authority.
use super::*;
mod resolution;
pub(super) use resolution::ResolutionRequest;
pub use resolution::{Resolution, ResolutionAction, SavedResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictRole {
    Base,
    Current,
    Applied,
}
impl ConflictRole {
    pub const ALL: [Self; 3] = [Self::Base, Self::Current, Self::Applied];
    pub fn stage(self) -> usize {
        match self {
            Self::Base => 1,
            Self::Current => 2,
            Self::Applied => 3,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Base => "Base",
            Self::Current => "Current",
            Self::Applied => "Applied",
        }
    }
    pub fn meaning(self, family: Option<SequenceFamily>) -> &'static str {
        match (family, self) {
            (Some(SequenceFamily::Rebase), Self::Current) => "rebased destination / onto side",
            (Some(SequenceFamily::Rebase), Self::Applied) => "original commit being replayed",
            (Some(SequenceFamily::Revert), Self::Base) => "commit being reverted",
            (Some(SequenceFamily::Revert), Self::Applied) => "selected parent of reverted commit",
            (Some(SequenceFamily::CherryPick), Self::Applied) => "commit being cherry-picked",
            (Some(SequenceFamily::Merge), Self::Applied) => "merged revision side",
            (_, Self::Base) => "Git index stage 1 ancestor",
            (_, Self::Current) => "current HEAD side",
            (_, Self::Applied) => "Git index stage 3 applied side; lineage unavailable",
        }
    }
}
#[derive(Debug)]
pub struct ConflictSide {
    pub mode: u32,
    pub object: ObjectId,
    pub bytes: Vec<u8>,
    pub completeness: Completeness,
    pub binary: bool,
    pub text: bool,
    pub lines: Vec<std::ops::Range<usize>>,
}
#[derive(Debug)]
pub struct Conflict {
    pub id: OperationId,
    pub sequence: Arc<Sequence>,
    pub change: PathChange,
    pub sides: [Option<ConflictSide>; 3],
    pub completeness: Completeness,
    pub(in crate::git) _admission: Lease,
}
impl Conflict {
    pub fn side(&self, role: ConflictRole) -> Option<&ConflictSide> {
        self.sides[role.stage() - 1].as_ref()
    }
    pub fn kind(&self) -> &'static str {
        match (self.change.index, self.change.worktree) {
            (b'U', b'U') => "both modified",
            (b'A', b'A') => "both added",
            (b'D', b'D') => "both deleted",
            (b'U', b'D') => "deleted by applied side / possible rename-delete",
            (b'D', b'U') => "deleted by current side / possible rename-delete",
            (b'A', b'U') => "added by current side / structural conflict",
            (b'U', b'A') => "added by applied side / structural conflict",
            _ => "unsupported unmerged kind",
        }
    }
}
pub(super) struct Request {
    pub sequence: Arc<Sequence>,
    pub path: PathBuf,
    pub _admission: Lease,
}
impl Repositories {
    pub fn inspect_conflict(
        &mut self,
        sequence: Arc<Sequence>,
        path: &Path,
    ) -> Result<OperationId, String> {
        self.check_sequence(&sequence)?;
        if !sequence.conflicts.iter().any(|p| p == path) {
            return Err("Select one exact observed conflict path".into());
        }
        let root = self.mutation_identity_root(&sequence.refs.observation.identity)?;
        let admission = self.budget.reserve(path.as_os_str().len() + 4096)?;
        self.start(
            root,
            Some(sequence.refs.observation.context),
            work::Request::Conflict(Request {
                sequence,
                path: path.into(),
                _admission: admission,
            }),
        )
    }
}
