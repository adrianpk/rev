use super::*;
use crate::git::integration::Request;
use std::{fs, io::Read, os::unix::ffi::OsStrExt};
mod autosquash;
mod control;
mod sequence;

impl Runner {
    pub(super) fn integration(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::PrepareAutosquash { refs, target, .. } => self
                .prepare_autosquash(refs, &target, admission)
                .map(Candidate::Autosquash),
            Request::Autosquash(plan) => Integration {
                refs: &plan.refs,
                target_name: &plan.target_name,
                mode: IntegrationMode::Rebase,
                plan: Some(&plan),
            }
            .run(self, admission),
            Request::Control {
                sequence,
                action,
                family,
            } => self.control_sequence(&sequence, action, family, admission),
            Request::Observe {
                observation,
                application,
                bisect,
            } => self.observe_integration(&observation, application, bisect, admission),
            Request::Start {
                refs, target, mode, ..
            } => Integration {
                refs: &refs,
                target_name: &target,
                mode,
                plan: None,
            }
            .run(self, admission),
        }
    }
    pub(super) fn integration_refs(
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
    pub(super) fn check_integration_source(
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
    fn observe_integration(
        &mut self,
        observation: &Observation,
        application: Option<Arc<ApplyProgress>>,
        bisect: Option<Arc<BisectPlan>>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let refs = self.integration_refs(observation, admission)?;
        let mut sequence = self.read_sequence(&refs, admission)?;
        if let Some(progress) = application {
            self.bind_application(&mut sequence, progress)?;
        }
        if let Some(plan) = bisect {
            self.bind_bisect(&mut sequence, plan)?;
        }
        Ok(Candidate::Sequence(sequence))
    }
}

/// One checked integration target and optional reviewed autosquash source.
struct Integration<'a> {
    refs: &'a Arc<Refs>,
    target_name: &'a [u8],
    mode: IntegrationMode,
    plan: Option<&'a AutosquashPlan>,
}
impl Integration<'_> {
    fn run(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let refs = self.refs;
        let target_name = self.target_name;
        let mode = self.mode;
        self.check_available(runner, admission)?;
        runner.integration_policy(refs, mode)?;
        let target = refs
            .reference(target_name)
            .and_then(Reference::commit)
            .ok_or("Exact integration target expired")?;
        let before = refs.head.ok_or("Integration HEAD is unborn")?;
        runner.check_integration_source(refs, admission)?;
        let args = self.arguments(runner, target, admission)?;
        let (executed, facts) = runner.execute_change(&args, None);
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err(
                "Integration cleanup remains owned; inspect actual Git sequence state".into(),
            );
        }
        let result = Lineage {
            integration: self,
            before,
            target,
        }
        .observe(runner, executed.is_ok(), admission);
        runner.restore_execution(facts, executed.is_err());
        if result.is_err() {
            runner.operation_failure = Some(
                "Integration effects remain unverified; refresh sequence state; no automatic retry"
                    .into(),
            );
        }
        result
    }
    fn check_available(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let refs = self.refs;
        let current = runner.check_integration_source(refs, admission)?;
        let state = runner.read_sequence(&current, admission)?;
        if state.family.is_some() {
            return Err(
                "A Git-owned sequence is already active; inspect it before integration".into(),
            );
        }
        Ok(())
    }
    fn arguments(
        &self,
        runner: &mut Runner,
        target: ObjectId,
        admission: &mut Lease,
    ) -> Result<Vec<OsString>, String> {
        let mode = self.mode;
        let plan = self.plan;
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
        if let Some(plan) = plan {
            args = runner.autosquash_args(plan, admission)?;
        } else {
            args.push(target.hex().into());
        }
        Ok(args)
    }
}

/// Reviewed lineage survives launch and binds both active and completed results.
struct Lineage<'a, 'b> {
    integration: &'a Integration<'b>,
    before: ObjectId,
    target: ObjectId,
}
impl Lineage<'_, '_> {
    fn observe(
        &self,
        runner: &mut Runner,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let refs = self.integration.refs;
        let after = runner.integration_refs(&refs.observation, admission)?;
        let sequence = runner.read_sequence(&after, admission)?;
        if sequence.family.is_some() {
            runner.in_progress = self.still_active(&sequence);
            runner.operation_failure=Some("Git integration stopped in progress; explicit checked sequence control is required".into());
        } else {
            self.completed(runner, &sequence, executed)?;
        }
        Ok(Candidate::Sequence(sequence))
    }
    fn still_active(&self, sequence: &Sequence) -> bool {
        let mode = self.integration.mode;
        let refs = self.integration.refs;
        let before = self.before;
        let target = self.target;
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
        sequence.family == Some(expected) && lineage
    }
    fn completed(
        &self,
        runner: &mut Runner,
        sequence: &Sequence,
        executed: bool,
    ) -> Result<(), String> {
        let refs = self.integration.refs;
        let target = self.target;
        let after = &sequence.refs;
        let head = after.head.ok_or("Integration lost HEAD")?;
        let target_present = runner
            .command(
                &["merge-base", "--is-ancestor", &target.hex(), &head.hex()],
                Phase::Verifying,
                true,
            )
            .is_ok()
            && runner.exit.is_some_and(|s| s.success());
        let identity = self.result_identity(after);
        let policy = self.policy(runner, sequence, head)?;
        let other_refs = self.other_refs(after);
        if executed && target_present && identity && policy && other_refs {
            runner.effects = Effects::Verified;
        } else {
            runner.failure_verified = !executed && after.same(refs);
            runner.operation_failure=Some("Integration failed or its exact branch/ancestry/policy postcondition differs; inspect actual facts".into());
        }
        Ok(())
    }
    fn result_identity(&self, after: &Refs) -> bool {
        let refs = self.integration.refs;
        let same_branch = after.head_ref == refs.head_ref;
        let clean = after
            .observation
            .status
            .as_ref()
            .is_some_and(|s| s.completeness == Completeness::Complete && s.paths.is_empty());
        same_branch && clean
    }
    fn other_refs(&self, after: &Refs) -> bool {
        let refs = self.integration.refs;
        after.completeness == Completeness::Complete
            && after.entries.len() == refs.entries.len()
            && refs
                .entries
                .iter()
                .filter(|r| r.name != refs.head_ref)
                .all(|r| {
                    after.reference(&r.name).is_some_and(|actual| {
                        actual.object == r.object && actual.symbolic == r.symbolic
                    })
                })
    }
    fn policy(
        &self,
        runner: &mut Runner,
        sequence: &Sequence,
        head: ObjectId,
    ) -> Result<bool, String> {
        let mode = self.integration.mode;
        let refs = self.integration.refs;
        let before = self.before;
        let after = &sequence.refs;
        Ok(if mode == IntegrationMode::Rebase {
            // Git's original-object provenance binds a changing rebase
            // to the reviewed source; ancestry alone cannot do so.
            (head == before && after.same(refs))
                || sequence
                    .files
                    .iter()
                    .find(|f| f.name == b"ORIG_HEAD")
                    .is_some_and(|f| {
                        ObjectId::parse(
                            f.bytes.strip_suffix(b"\n").unwrap_or(&f.bytes),
                            refs.observation.identity.format,
                        )
                        .ok()
                            == Some(before)
                    })
        } else {
            self.merge_policy(runner, head)?
        })
    }
    fn merge_policy(&self, runner: &mut Runner, head: ObjectId) -> Result<bool, String> {
        let mode = self.integration.mode;
        let before = self.before;
        let target = self.target;
        let before_present = runner
            .command(
                &["merge-base", "--is-ancestor", &before.hex(), &head.hex()],
                Phase::Verifying,
                true,
            )
            .is_ok()
            && runner.exit.is_some_and(|s| s.success());
        let parent_ok =
            if head == before || (mode != IntegrationMode::NoFastForward && head == target) {
                true
            } else {
                runner.command(
                    &["show", "--no-patch", "--format=%P", &head.hex()],
                    Phase::Verifying,
                    false,
                )? == format!("{} {}\n", before.hex(), target.hex()).as_bytes()
            };
        Ok(before_present
            && parent_ok
            && (mode != IntegrationMode::FastForward || head == before || head == target))
    }
}
