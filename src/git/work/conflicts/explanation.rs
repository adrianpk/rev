use super::*;
use crate::git::ConflictExplanation;
use crate::git::conflicts::resolution::fingerprint;

impl Runner {
    pub(in crate::git::work) fn explain_conflict(
        &mut self,
        conflict: Arc<Conflict>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.checked_sequence(&conflict.sequence, admission)?;
        let index = std::fs::symlink_metadata(
            conflict
                .sequence
                .refs
                .observation
                .identity
                .git_directory
                .join("index"),
        )
        .map_err(|e| format!("Conflict explanation index source: {e}"))?;
        if !index.is_file() || index.file_type().is_symlink() {
            return Err("Conflict explanation requires an ordinary index source".into());
        }
        let saved = self.saved_result(&conflict, admission);
        if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
            return Err("Conflict explanation capture cancelled or timed out".into());
        }
        let size = std::mem::size_of::<ConflictExplanation>()
            + match &saved {
                Ok(Some(s)) => s.bytes.capacity(),
                Err(e) => e.capacity(),
                _ => 0,
            };
        admission.grow(size)?;
        let snapshot = ConflictExplanation {
            conflict,
            saved,
            index: fingerprint(&index),
            _admission: admission.split(size),
        };
        self.checked_sequence(&snapshot.conflict.sequence, admission)?;
        if snapshot.saved.is_ok() {
            snapshot.current()?;
        }
        Ok(Candidate::ConflictExplanation(snapshot))
    }
}
