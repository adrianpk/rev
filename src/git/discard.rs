//! Whole-path working-tree replacement from one captured index entry.
use super::*;

impl Repositories {
    /// Replace one tracked unstaged path from its checked index entry. Native
    /// callers own the one-shot confirmation and canonical source admission.
    /// No untracked deletion, hunk/group discard or HEAD reset is implied.
    pub fn discard(&mut self, diff: Arc<Diff>) -> Result<OperationId, String> {
        self.check_discard(&diff)?;
        let root = self.mutation_root(&diff)?;
        let context = diff.observation.context;
        self.start(root, Some(context), work::Request::Discard(diff))
    }

    pub(crate) fn check_discard(&self, diff: &Diff) -> Result<(), String> {
        if !self.detail_current(diff)
            || diff
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete)
            || diff.group != Group::Unstaged
            || !matches!(
                diff.change.modes.get(1),
                Some(0o100644 | 0o100755 | 0o120000)
            )
            || diff.change.renamed_in(diff.group)
            || matches!(diff.kind, DiffKind::Unmerged | DiffKind::Malformed(_))
        {
            return Err(
                "Discard requires one complete current tracked unstaged path with a regular-file or symlink index entry"
                    .into(),
            );
        }
        Ok(())
    }

    pub(crate) fn source_admission(&self, bytes: usize) -> Result<Lease, String> {
        self.budget.reserve(bytes)
    }
}
