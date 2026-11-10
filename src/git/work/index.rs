use super::*;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Entry {
    path: PathBuf,
    mode: Option<u32>,
    object: Option<ObjectId>,
}
struct Prepared {
    args: Vec<OsString>,
    input: Option<fs::File>,
    expected: Vec<Entry>,
}
#[derive(Clone)]
pub(super) struct Execution {
    argv: Vec<OsString>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: Option<std::process::ExitStatus>,
    truncated: bool,
    end: capture::End,
}

impl Execution {
    pub(super) fn stdout(&self) -> &[u8] {
        &self.stdout
    }
    pub(super) fn code(&self) -> Option<i32> {
        self.exit.and_then(|s| s.code())
    }
}

mod entries;
mod hunks;
mod paths;
mod verification;

pub(super) use entries::entries;

impl Runner {
    pub(super) fn change_index(
        &mut self,
        request: &super::super::index::Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let diff = request.targets[0].diff();
        self.check_index_comparisons(&request.targets, admission)?;
        let Prepared {
            args,
            input,
            expected,
        } = match &request.targets[0] {
            IndexTarget::Path(_) => {
                self.path_group(request.direction, &request.targets, admission)?
            }
            IndexTarget::Hunk { .. } => {
                let hunks: Vec<_> = request
                    .targets
                    .iter()
                    .filter_map(|t| match t {
                        IndexTarget::Hunk { hunk, .. } => Some(*hunk),
                        _ => None,
                    })
                    .collect();
                self.hunk_action(request.direction, diff, &hunks, admission)?
            }
        };
        // All captured comparisons are checked again after hashing/filtering
        // and patch preflight. No subset has effects while another is checked.
        self.check_index_comparisons(&request.targets, admission)?;
        let (executed, facts) = self.execute_change(&args, input);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Index child cleanup remains owned; effects require later inspection".into(),
            );
        }
        // A failed/interrupted changing child is never replayed. Refresh is
        // attempted under the remaining original deadline and cancellation.
        let result = self.verify_index_result(diff, &expected, admission);
        self.restore_execution(facts, executed.is_err());
        self.publish_index_result(executed, result)
    }

    fn publish_index_result(
        &mut self,
        executed: Result<(), String>,
        result: Result<(Candidate, Result<(), String>), String>,
    ) -> Result<Candidate, String> {
        match result {
            Ok((candidate, verified)) => {
                match (executed, verified) {
                    (Ok(_), Ok(())) => self.effects = Effects::Verified,
                    (Err(error), _) => {
                        self.operation_failure = Some(format!(
                            "Index child failed after possible effects: {error}"
                        ))
                    }
                    (_, Err(error)) => self.operation_failure = Some(error),
                }
                Ok(candidate)
            }
            Err(error) => {
                self.operation_failure = Some(format!(
                    "Index effects remain unverified; refresh required: {error}"
                ));
                Err(error)
            }
        }
    }

    pub(super) fn execute_change(
        &mut self,
        args: &[OsString],
        input: Option<fs::File>,
    ) -> (Result<(), String>, Execution) {
        let (result, facts) = self.execute_change_output(args, input);
        (result.map(|_| ()), facts)
    }
    pub(super) fn execute_change_output(
        &mut self,
        args: &[OsString],
        input: Option<fs::File>,
    ) -> (Result<Vec<u8>, String>, Execution) {
        self.execute_change_argv(self.git_argv(args), input)
    }
    pub(super) fn execute_change_argv(
        &mut self,
        argv: Vec<OsString>,
        input: Option<fs::File>,
    ) -> (Result<Vec<u8>, String>, Execution) {
        let executed = self
            .command_argv(Invocation { argv, input }, Phase::Executing, false, None)
            .map(|(bytes, _)| bytes);
        self.effects = if self.exit.is_some() || self.retained.is_some() {
            Effects::Possible
        } else {
            Effects::NotStarted
        };
        let facts = Execution {
            argv: std::mem::take(&mut self.argv),
            stdout: std::mem::take(&mut self.stdout),
            stderr: std::mem::take(&mut self.stderr),
            exit: self.exit.take(),
            truncated: self.truncated,
            end: self.end,
        };
        (executed, facts)
    }
    pub(super) fn restore_execution(&mut self, facts: Execution, failed: bool) {
        self.argv = facts.argv;
        self.stdout = facts.stdout;
        self.stderr = facts.stderr;
        self.exit = facts.exit;
        self.truncated = facts.truncated;
        if failed {
            self.end = facts.end;
        }
    }

    pub(super) fn check_index_comparisons(
        &mut self,
        targets: &[IndexTarget],
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.check_index_status(targets, admission)?;
        for (i, selected) in targets.iter().enumerate() {
            let diff = selected.diff();
            if targets[..i].iter().any(|t| t.diff().id == diff.id) {
                continue;
            }
            let target = diff::Target {
                observation: std::sync::Arc::clone(&diff.observation),
                group: diff.group,
                change: diff.change.clone(),
            };
            let (bytes, limited) = self.command_os(
                &self.diff_args(&target),
                Phase::Checking,
                target.group == Group::Untracked,
                Some(self.policy.max_detail_bytes),
            )?;
            if limited || bytes != diff.evidence {
                return Err(
                    "Index group comparison changed or is incomplete; refresh required".into(),
                );
            }
        }
        self.check_index_status(targets, admission)
    }

    fn check_index_status(
        &mut self,
        targets: &[IndexTarget],
        admission: &mut Lease,
    ) -> Result<(), String> {
        let (identity, status) = self.observe(admission)?;
        let current = status.as_ref().ok_or("Working-tree Status unavailable")?;
        for target in targets {
            let diff = target.diff();
            let prior = diff
                .observation
                .status
                .as_ref()
                .ok_or("Captured Status unavailable")?;
            if identity != diff.observation.identity
                || current.head != prior.head
                || current.completeness != Completeness::Complete
                || current
                    .group(diff.group)
                    .find(|p| p.path == diff.change.path)
                    != Some(&diff.change)
            {
                return Err(
                    "Index group path, layer or repository changed; refresh required".into(),
                );
            }
        }
        Ok(())
    }
}
