use super::*;
use crate::git::families::{FamilyProof, Sparse};
use std::os::unix::ffi::OsStringExt;

fn sparse_policy(config: &[u8]) -> Vec<&[u8]> {
    config
        .split(|b| *b == 0)
        .filter(|entry| !entry.is_empty())
        .filter(|entry| {
            let key = entry.split(|b| *b == b'\n').next().unwrap_or_default();
            ![
                b"core.sparsecheckout".as_slice(),
                b"core.sparsecheckoutcone",
                b"index.sparse",
                b"extensions.worktreeconfig",
            ]
            .contains(&key)
        })
        .collect()
}
impl Runner {
    fn sparse_materialization(&mut self, flags: &[u8]) -> Result<(), String> {
        let mut count = 0;
        for field in flags.split(|b| *b == 0).filter(|f| !f.is_empty()) {
            count += 1;
            if count > self.policy.max_status_paths
                || self.cancel.is_cancelled()
                || Instant::now() >= self.deadline
            {
                return Err(
                    "Sparse materialization exceeds complete path/deadline admission".into(),
                );
            }
            if field.len() < 3 || field[1] != b' ' || !b"HS".contains(&field[0]) {
                return Err("Sparse index has an unsupported flag or unmerged entry".into());
            }
            let path = PathBuf::from(OsString::from_vec(field[2..].to_vec()));
            if path.as_os_str().len() > 4096
                || path
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(n) if n != ".git"))
            {
                return Err("Sparse index path is not a bounded literal repository path".into());
            }
            let present = match std::fs::symlink_metadata(self.directory.join(&path)) {
                Ok(_) => true,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
                Err(e) => return Err(format!("Observe sparse materialization: {e}")),
            };
            if present != (field[0] == b'H') {
                return Err("Sparse index/materialized files disagree; inspect excluded or missing saved files before retrying".into());
            }
        }
        Ok(())
    }
    fn sparse_state(&mut self, source: &Sequence, admission: &mut Lease) -> Result<Sparse, String> {
        let configuration = self.transport_config(admission)?;
        let enabled = self.transport_bool(b"core.sparsecheckout".to_vec())?;
        let cone = self.transport_bool(b"core.sparsecheckoutcone".to_vec())?;
        if enabled && !cone {
            return Err("Prepared sparse routes require cone mode; existing non-cone patterns remain unchanged".into());
        }
        let patterns = self
            .family_file(
                &source
                    .refs
                    .observation
                    .identity
                    .git_directory
                    .join("info/sparse-checkout"),
            )?
            .unwrap_or_default();
        let mut directories = Vec::new();
        if enabled {
            let list = self.command(&["sparse-checkout", "list"], Phase::Checking, false)?;
            for line in list.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
                if directories.len() == self.policy.max_status_paths {
                    return Err("Sparse directory list exceeds complete count admission".into());
                }
                let path = super::super::history::unquote_path(line)
                    .map_err(|e| format!("Sparse path response: {e}"))?;
                self.family_relative_destination(path.as_os_str())?;
                directories.push(path);
            }
            directories.sort();
            directories.dedup();
        }
        let flags = self.command(&["ls-files", "-t", "-z"], Phase::Checking, false)?;
        self.sparse_materialization(&flags)?;
        admission.grow(
            patterns.capacity()
                + flags.capacity()
                + directories.iter().map(PathBuf::capacity).sum::<usize>(),
        )?;
        Ok(Sparse {
            configuration,
            enabled,
            cone,
            patterns,
            directories,
            flags,
        })
    }
    pub(super) fn prepare_sparse(
        &mut self,
        family: Family,
        args: &[OsString],
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Sparse, String> {
        if source.family.is_some()
            || source.refs.completeness != Completeness::Complete
            || source
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Sparse changes require complete clean saved state and no active sequence; no implicit save or stash".into());
        }
        let state = self.sparse_state(source, admission)?;
        if matches!(family, Family::SparseReapply | Family::SparseAdd) && !state.enabled {
            return Err("Enable sparse cone mode before reapply or add".into());
        }
        if matches!(family, Family::SparseSet | Family::SparseAdd) {
            let start = if family == Family::SparseSet { 6 } else { 4 };
            for directory in &args[start..] {
                // Git stores cone paths as newline-delimited patterns. A literal
                // newline cannot round-trip even through its --stdin quoting.
                if directory.as_encoded_bytes().contains(&b'\n') {
                    return Err("Git cone patterns cannot represent a newline directory; no sparse changes started".into());
                }
                self.family_relative_destination(directory)?;
            }
        }
        Ok(state)
    }
    pub(super) fn execute_sparse(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Sparse(expected) = &plan.proof else {
            return Err("Sparse proof unavailable".into());
        };
        let source = plan.source.as_ref().expect("sparse source");
        if self.sparse_state(source, admission)? != *expected {
            return Err(
                "Sparse configuration, patterns or index flags changed during review".into(),
            );
        }
        let (executed, facts) = self.execute_change(&plan.args, None);
        let failed = executed.is_err();
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Sparse cleanup remains owned; no retry or rollback".into());
        }
        let verified: Result<(), String> = (|| {
            executed?;
            let current = self.family_current(&plan, None, admission)?;
            let actual = self.sparse_state(&current, admission)?;
            if sparse_policy(&actual.configuration) != sparse_policy(&expected.configuration) {
                return Err("Sparse command changed unrelated effective configuration".into());
            }
            let mut directories = expected.directories.clone();
            match plan.family {
                Family::SparseDisable => directories.clear(),
                Family::SparseSet => {
                    directories = plan.args[6..].iter().map(PathBuf::from).collect()
                }
                Family::SparseAdd => directories.extend(plan.args[4..].iter().map(PathBuf::from)),
                _ => {}
            }
            directories.sort();
            directories.dedup();
            // Cone mode collapses descendants of an already selected parent.
            // Compare that exact Git selection rather than the input spelling.
            let mut selected: Vec<PathBuf> = Vec::new();
            for directory in directories {
                if !selected.iter().any(|parent| directory.starts_with(parent)) {
                    selected.push(directory);
                }
            }
            let enabled = plan.family != Family::SparseDisable;
            if actual.enabled != enabled
                || (enabled && !actual.cone)
                || actual.directories != selected
            {
                return Err(
                    "Sparse cone configuration or selected directory postcondition differs".into(),
                );
            }
            self.effects = Effects::Verified;
            Ok(())
        })();
        if let Err(e) = verified {
            self.operation_failure = Some(format!(
                "Sparse result is unverified: {e}; inspect refreshed facts; no retry, force or rollback"
            ));
        }
        let result = self.family_observed_result(plan.clone(), None, admission);
        self.restore_execution(facts, failed);
        result
    }
}
