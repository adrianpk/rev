use super::*;

impl Runner {
    pub(in crate::git::work) fn checked_sequence(
        &mut self,
        expected: &Sequence,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        let refs = self.integration_refs(&expected.refs.observation, admission)?;
        let mut actual = self.read_sequence(&refs, admission)?;
        if let Some(progress) = &expected.application {
            self.bind_application(&mut actual, std::sync::Arc::clone(progress))?;
        }
        if let Some(plan) = expected.bisect.as_ref().and_then(|b| b.plan.clone()) {
            self.bind_bisect(&mut actual, plan)?;
        }
        if !expected.same(&actual) {
            return Err(
                "Git sequence, refs, index or working-tree evidence changed while review was open"
                    .into(),
            );
        }
        Ok(actual)
    }
    pub(super) fn control_sequence(
        &mut self,
        expected: &Arc<Sequence>,
        action: SequenceControl,
        family: Option<SequenceFamily>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        if expected.application.is_some() {
            let result = self.control_application(expected, action, family, admission);
            if result.is_err() && self.effects == Effects::Possible {
                self.operation_failure.get_or_insert_with(|| "Ordered control effects remain unverified; refresh actual Git state; no automatic retry or inferred abort".into());
            }
            return result;
        }
        let current = self.checked_sequence(expected, admission)?;
        current.control_admission(action, family)?;
        let actual = current.family.ok_or("Git sequence disappeared")?;
        let original = current
            .original
            .ok_or("Original sequence object is unavailable")?;
        Control {
            expected,
            action,
            actual,
            original,
        }
        .run(self, admission)
    }
}

/// One admitted control retains its source sequence and exact original lineage.
struct Control<'a> {
    expected: &'a Arc<Sequence>,
    action: SequenceControl,
    actual: SequenceFamily,
    original: ObjectId,
}
impl Control<'_> {
    fn run(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let expected = self.expected;
        let action = self.action;
        let actual = self.actual;
        let args: Vec<OsString> = match (actual, action) {
            (SequenceFamily::Merge, SequenceControl::Continue) => {
                vec!["commit".into(), "--no-edit".into()]
            }
            (SequenceFamily::Merge, SequenceControl::Abort) => {
                vec!["merge".into(), "--abort".into()]
            }
            (SequenceFamily::Rebase, control) => {
                vec!["rebase".into(), format!("--{}", control.label()).into()]
            }
            _ => return Err("No compatible Git sequence control is admitted".into()),
        };
        runner.checked_sequence(expected, admission)?;
        runner.accept_message = action == SequenceControl::Continue;
        let (executed, facts) = runner.execute_change(&args, None);
        runner.accept_message = false;
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err(
                "Sequence control cleanup remains owned; no abort or rollback is inferred".into(),
            );
        }
        let result = self.observe(runner, executed.is_ok(), admission);
        runner.restore_execution(facts, executed.is_err());
        if result.is_err() {
            runner.operation_failure = Some(
                "Sequence control effects remain unverified; refresh required; no automatic retry"
                    .into(),
            );
        }
        result
    }
    fn observe(
        &self,
        runner: &mut Runner,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let expected = self.expected;
        let refs = runner.integration_refs(&expected.refs.observation, admission)?;
        let after = runner.read_sequence(&refs, admission)?;
        if after.family.is_some() {
            runner.in_progress = self.still_active(&after);
            runner.operation_failure = Some(
                "Git sequence remains in progress; current step/conflicts were refreshed".into(),
            );
        } else {
            self.completed(runner, &after, executed)?;
        }
        Ok(Candidate::Sequence(after))
    }
    fn completed(
        &self,
        runner: &mut Runner,
        after: &Sequence,
        executed: bool,
    ) -> Result<(), String> {
        let expected = self.expected;
        let action = self.action;
        let refs = &after.refs;
        let valid = self.valid(runner, after)?;
        let expected_branch = self.expected_branch();
        let other_refs = self.other_refs(refs, expected_branch);
        if executed && valid && other_refs {
            runner.effects = Effects::Verified;
            runner.completion = Some(match action {
                SequenceControl::Abort => Outcome::Aborted,
                SequenceControl::Skip => Outcome::Skipped,
                _ => Outcome::Verified,
            });
        } else {
            runner.failure_verified = !executed && expected.same(after);
            runner.operation_failure=Some("Sequence control failed or exact resulting branch/lineage/index postcondition differs".into());
        }
        Ok(())
    }
    fn still_active(&self, after: &Sequence) -> bool {
        let actual = self.actual;
        let original = self.original;
        let expected = self.expected;
        after.family == Some(actual)
            && after.original == Some(original)
            && (actual != SequenceFamily::Rebase
                || (after.onto == expected.onto && after.branch == expected.branch))
    }
    fn other_refs(&self, refs: &Refs, expected_branch: &[u8]) -> bool {
        let expected = self.expected;
        refs.completeness == Completeness::Complete
            && refs.entries.len() == expected.refs.entries.len()
            && expected
                .refs
                .entries
                .iter()
                .filter(|r| r.name != expected_branch)
                .all(|r| {
                    refs.reference(&r.name)
                        .is_some_and(|a| a.object == r.object && a.symbolic == r.symbolic)
                })
    }
    fn expected_branch(&self) -> &[u8] {
        let expected = self.expected;
        let actual = self.actual;
        if actual == SequenceFamily::Rebase {
            if expected.branch == b"detached HEAD" {
                b"".as_slice()
            } else {
                expected.branch.as_slice()
            }
        } else {
            expected.refs.head_ref.as_slice()
        }
    }
    fn valid(&self, runner: &mut Runner, after: &Sequence) -> Result<bool, String> {
        let expected = self.expected;
        let action = self.action;
        let actual = self.actual;
        let original = self.original;
        let refs = &after.refs;
        let expected_branch = self.expected_branch();
        Ok(if action == SequenceControl::Abort {
            refs.head == Some(original) && refs.head_ref == expected_branch
        } else if actual == SequenceFamily::Merge {
            self.valid_merge(runner, refs, expected_branch)?
        } else {
            let head = refs.head.ok_or("Rebase continuation lost HEAD")?;
            let onto = expected.onto.ok_or("Rebase onto object is unavailable")?;
            let result = runner.command(
                &["merge-base", "--is-ancestor", &onto.hex(), &head.hex()],
                Phase::Verifying,
                true,
            );
            refs.head_ref == expected_branch
                && result.is_ok()
                && runner.exit.is_some_and(|s| s.success())
        })
    }
    fn valid_merge(
        &self,
        runner: &mut Runner,
        refs: &Refs,
        expected_branch: &[u8],
    ) -> Result<bool, String> {
        let expected = self.expected;
        let original = self.original;
        let head = refs.head.ok_or("Merge continuation lost HEAD")?;
        let parents = runner.command(
            &["show", "--no-patch", "--format=%P", &head.hex()],
            Phase::Verifying,
            false,
        )?;
        let wanted = std::iter::once(original)
            .chain(expected.targets.iter().copied())
            .map(|id| id.hex())
            .collect::<Vec<_>>()
            .join(" ")
            + "\n";
        let tree_matches = self.merge_tree(runner, refs, head)?;
        Ok(refs.head_ref == expected_branch
            && parents == wanted.as_bytes()
            && tree_matches
            && runner.command(
                &[
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "--binary",
                    "--",
                ],
                Phase::Verifying,
                false,
            )? == expected.worktree
            && runner.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?
                == expected.index)
    }
    fn merge_tree(&self, runner: &mut Runner, refs: &Refs, head: ObjectId) -> Result<bool, String> {
        let expected = self.expected;
        let tree = runner.command(
            &["ls-tree", "--full-tree", "-r", "-z", &head.hex()],
            Phase::Verifying,
            false,
        )?;
        let actual_tree = index::entries(
            &tree,
            refs.observation.identity.format,
            false,
            runner.policy.max_status_paths,
        )?;
        let expected_tree = index::entries(
            &expected.index,
            refs.observation.identity.format,
            true,
            runner.policy.max_status_paths,
        )?;
        Ok(actual_tree == expected_tree)
    }
}
