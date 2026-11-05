use super::*;

impl Runner {
    pub(in crate::git::work) fn checked_sequence(
        &mut self,
        expected: &Sequence,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        let refs = self.integration_refs(&expected.refs.observation, admission)?;
        let mut actual = self.read_sequence(refs, admission)?;
        if let Some(progress) = &expected.application {
            self.bind_application(&mut actual, progress.clone())?;
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
        expected: Arc<Sequence>,
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
        let current = self.checked_sequence(&expected, admission)?;
        current.control_admission(action, family)?;
        let actual = current.family.ok_or("Git sequence disappeared")?;
        let original = current
            .original
            .ok_or("Original sequence object is unavailable")?;
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
        self.checked_sequence(&expected, admission)?;
        self.accept_message = action == SequenceControl::Continue;
        let (executed, facts) = self.execute_change(&args, None);
        self.accept_message = false;
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Sequence control cleanup remains owned; no abort or rollback is inferred".into(),
            );
        }
        let result = (|| {
            let refs = self.integration_refs(&expected.refs.observation, admission)?;
            let after = self.read_sequence(refs, admission)?;
            if after.family.is_some() {
                self.in_progress = after.family == Some(actual)
                    && after.original == Some(original)
                    && (actual != SequenceFamily::Rebase
                        || (after.onto == expected.onto && after.branch == expected.branch));
                self.operation_failure = Some(
                    "Git sequence remains in progress; current step/conflicts were refreshed"
                        .into(),
                );
            } else {
                let refs = &after.refs;
                let expected_branch = if actual == SequenceFamily::Rebase {
                    if expected.branch == b"detached HEAD" {
                        b"".as_slice()
                    } else {
                        expected.branch.as_slice()
                    }
                } else {
                    expected.refs.head_ref.as_slice()
                };
                let valid = if action == SequenceControl::Abort {
                    refs.head == Some(original) && refs.head_ref == expected_branch
                } else if actual == SequenceFamily::Merge {
                    let head = refs.head.ok_or("Merge continuation lost HEAD")?;
                    let parents = self.command(
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
                    let tree = self.command(
                        &["ls-tree", "--full-tree", "-r", "-z", &head.hex()],
                        Phase::Verifying,
                        false,
                    )?;
                    let actual_tree = index::entries(
                        &tree,
                        refs.observation.identity.format,
                        false,
                        self.policy.max_status_paths,
                    )?;
                    let expected_tree = index::entries(
                        &expected.index,
                        refs.observation.identity.format,
                        true,
                        self.policy.max_status_paths,
                    )?;
                    refs.head_ref == expected_branch
                        && parents == wanted.as_bytes()
                        && actual_tree == expected_tree
                        && self.command(
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
                        && self.command(&["ls-files", "--stage", "-z"], Phase::Verifying, false)?
                            == expected.index
                } else {
                    let head = refs.head.ok_or("Rebase continuation lost HEAD")?;
                    let onto = expected.onto.ok_or("Rebase onto object is unavailable")?;
                    let result = self.command(
                        &["merge-base", "--is-ancestor", &onto.hex(), &head.hex()],
                        Phase::Verifying,
                        true,
                    );
                    refs.head_ref == expected_branch
                        && result.is_ok()
                        && self.exit.is_some_and(|s| s.success())
                };
                let other_refs = refs.completeness == Completeness::Complete
                    && refs.entries.len() == expected.refs.entries.len()
                    && expected
                        .refs
                        .entries
                        .iter()
                        .filter(|r| r.name != expected_branch)
                        .all(|r| {
                            refs.reference(&r.name)
                                .is_some_and(|a| a.object == r.object && a.symbolic == r.symbolic)
                        });
                if executed.is_ok() && valid && other_refs {
                    self.effects = Effects::Verified;
                    self.completion = Some(match action {
                        SequenceControl::Abort => Outcome::Aborted,
                        SequenceControl::Skip => Outcome::Skipped,
                        _ => Outcome::Verified,
                    });
                } else {
                    self.failure_verified = executed.is_err() && expected.same(&after);
                    self.operation_failure=Some("Sequence control failed or exact resulting branch/lineage/index postcondition differs".into());
                }
            }
            Ok(Candidate::Sequence(after))
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some(
                "Sequence control effects remain unverified; refresh required; no automatic retry"
                    .into(),
            );
        }
        result
    }
}
