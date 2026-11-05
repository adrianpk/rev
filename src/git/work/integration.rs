use super::*;
use crate::git::integration::Request;
use std::{fs, io::Read, os::unix::ffi::OsStrExt};
mod sequence;

impl Runner {
    pub(super) fn integration(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::Observe(observation) => self
                .integration_refs(&observation, admission)
                .and_then(|refs| self.read_sequence(refs, admission))
                .map(Candidate::Sequence),
            Request::Start {
                refs, target, mode, ..
            } => self.start_integration(refs, &target, mode, admission),
        }
    }
    fn integration_refs(
        &mut self,
        expected: &Observation,
        admission: &mut Lease,
    ) -> Result<Arc<Refs>, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != expected.identity {
            return Err("Integration repository identity changed".into());
        }
        let id = self.progress.lock().expect("Git progress owner").operation;
        let mut current = Observation {
            context: expected.context,
            generation: id.0,
            identity,
            status,
            admission: admission.split(0),
        };
        current.admission = admission.split(current.bytes());
        self.read_refs(Arc::new(current), admission).map(Arc::new)
    }
    fn check_integration_source(
        &mut self,
        expected: &Refs,
        admission: &mut Lease,
    ) -> Result<Arc<Refs>, String> {
        let current = self.integration_refs(&expected.observation, admission)?;
        if !expected.same(&current) {
            return Err("Integration refs or HEAD changed while review was open".into());
        }
        if current
            .observation
            .status
            .as_ref()
            .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Integration requires a complete clean index and working tree; no implicit autostash".into());
        }
        Ok(current)
    }
    fn integration_policy(&mut self, refs: &Refs, mode: IntegrationMode) -> Result<(), String> {
        if mode != IntegrationMode::Rebase {
            let short = refs
                .head_ref
                .strip_prefix(b"refs/heads/")
                .ok_or("Integration requires a current branch")?;
            let key = [b"branch.".as_slice(), short, b".mergeoptions"].concat();
            let result = self.command_os(
                &["config".into(), "--get-all".into(), OsString::from_vec(key)],
                Phase::Checking,
                true,
                None,
            );
            self.stdout.clear();
            self.stderr.clear();
            let bytes = result
                .map_err(|_| "Integration policy unavailable; configuration output withheld")?
                .0;
            if !bytes.is_empty() {
                return Err("Configured branch mergeOptions require explicit revision before native integration".into());
            }
        }
        let keys = if mode == IntegrationMode::Rebase {
            &["rebase.autostash", "rebase.updaterefs"][..]
        } else {
            &["merge.autostash"][..]
        };
        for key in keys {
            let result = self.command(&["config", "--bool", "--get", key], Phase::Checking, true);
            self.stdout.clear();
            self.stderr.clear();
            let value = result.map_err(
                |_| "Integration policy boolean unavailable; configuration output withheld",
            )?;
            if value == b"true\n" {
                return Err("Configured autostash or expanded integration policy conflicts with this reviewed request".into());
            }
        }
        Ok(())
    }
    fn start_integration(
        &mut self,
        refs: Arc<Refs>,
        target_name: &[u8],
        mode: IntegrationMode,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let current = self.check_integration_source(&refs, admission)?;
        let state = self.read_sequence(current, admission)?;
        if state.family.is_some() {
            return Err(
                "A Git-owned sequence is already active; inspect it before integration".into(),
            );
        }
        self.integration_policy(&refs, mode)?;
        let target = refs
            .reference(target_name)
            .and_then(Reference::commit)
            .ok_or("Exact integration target expired")?;
        let before = refs.head.ok_or("Integration HEAD is unborn")?;
        self.check_integration_source(&refs, admission)?;
        let mut args: Vec<OsString> = if mode == IntegrationMode::Rebase {
            [
                "rebase",
                "--no-autostash",
                "--no-autosquash",
                "--no-update-refs",
                "--no-rebase-merges",
                "--no-fork-point",
            ]
            .into_iter()
            .map(OsString::from)
            .collect()
        } else {
            vec![
                "merge".into(),
                match mode {
                    IntegrationMode::FastForward => "--ff-only",
                    IntegrationMode::NoFastForward => "--no-ff",
                    _ => "--ff",
                }
                .into(),
                "--no-edit".into(),
                "--no-autostash".into(),
                "--".into(),
            ]
        };
        args.push(target.hex().into());
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Integration cleanup remains owned; inspect actual Git sequence state".into(),
            );
        }
        let result = (|| {
            let after = self.integration_refs(&refs.observation, admission)?;
            let sequence = self.read_sequence(after, admission)?;
            if sequence.family.is_some() {
                let expected = if mode == IntegrationMode::Rebase {
                    SequenceFamily::Rebase
                } else {
                    SequenceFamily::Merge
                };
                let lineage = sequence.original == Some(before)
                    && if mode == IntegrationMode::Rebase {
                        sequence.onto == Some(target) && sequence.branch == refs.head_ref
                    } else {
                        sequence.targets == [target]
                            && sequence.refs.head == Some(before)
                            && sequence.refs.head_ref == refs.head_ref
                    };
                self.in_progress = sequence.family == Some(expected) && lineage;
                self.operation_failure=Some("Git integration stopped in progress; explicit checked sequence control is required".into());
            } else {
                let after = &sequence.refs;
                let head = after.head.ok_or("Integration lost HEAD")?;
                let target_present = self
                    .command(
                        &["merge-base", "--is-ancestor", &target.hex(), &head.hex()],
                        Phase::Verifying,
                        true,
                    )
                    .is_ok()
                    && self.exit.is_some_and(|s| s.success());
                let same_branch = after.head_ref == refs.head_ref;
                let clean = after.observation.status.as_ref().is_some_and(|s| {
                    s.completeness == Completeness::Complete && s.paths.is_empty()
                });
                let policy = if mode == IntegrationMode::Rebase {
                    true
                } else {
                    let before_present = self
                        .command(
                            &["merge-base", "--is-ancestor", &before.hex(), &head.hex()],
                            Phase::Verifying,
                            true,
                        )
                        .is_ok()
                        && self.exit.is_some_and(|s| s.success());
                    let parent_ok = if head == before
                        || (mode != IntegrationMode::NoFastForward && head == target)
                    {
                        true
                    } else {
                        self.command(
                            &["show", "--no-patch", "--format=%P", &head.hex()],
                            Phase::Verifying,
                            false,
                        )? == format!("{} {}\n", before.hex(), target.hex()).as_bytes()
                    };
                    before_present
                        && parent_ok
                        && (mode != IntegrationMode::FastForward
                            || head == before
                            || head == target)
                };
                let other_refs = after.completeness == Completeness::Complete
                    && after.entries.len() == refs.entries.len()
                    && refs
                        .entries
                        .iter()
                        .filter(|r| r.name != refs.head_ref)
                        .all(|r| {
                            after.reference(&r.name).is_some_and(|actual| {
                                actual.object == r.object && actual.symbolic == r.symbolic
                            })
                        });
                if executed.is_ok()
                    && target_present
                    && same_branch
                    && clean
                    && policy
                    && other_refs
                {
                    self.effects = Effects::Verified;
                } else {
                    self.failure_verified = executed.is_err() && after.same(&refs);
                    self.operation_failure=Some("Integration failed or its exact branch/ancestry/policy postcondition differs; inspect actual facts".into());
                }
            }
            Ok(Candidate::Sequence(sequence))
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some(
                "Integration effects remain unverified; refresh sequence state; no automatic retry"
                    .into(),
            );
        }
        result
    }
}
