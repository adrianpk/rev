use super::super::commit::{check_status, same_index};
use super::*;

impl Runner {
    pub(super) fn commit(
        &mut self,
        request: super::super::commit::Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let expected = check_status(request.observation.status.as_ref())?;
        self.check_commit(&request.observation, admission)?;
        let before = self.commit_index()?;
        let count = before.iter().filter(|b| **b == 0).count();
        admission.grow(before.len() + count * 2 * std::mem::size_of::<index::Entry>())?;
        // Unsupported/incomplete full index evidence rejects before effects,
        // including unmerged stages and an entry-count admission overflow.
        index::entries(
            &before,
            request.observation.identity.format,
            true,
            self.policy.max_status_paths,
        )?;
        // A full semantic index snapshot covers unchanged paths as well as
        // Status's staged delta. Recheck after preparation, before launching
        // the sole mutator; no source save or hidden staging is involved.
        let input = crate::process::input::stage(Some(request.message.as_bytes()))?;
        self.check_commit(&request.observation, admission)?;
        if before != self.commit_index()? {
            return Err("Commit index changed during preparation; refresh required".into());
        }
        let args = ["commit", "--no-edit", "--cleanup=verbatim", "--file=-"].map(OsString::from);
        let (executed, facts) = self.execute_change(&args, input);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Commit child cleanup remains owned; inspect HEAD before retrying".into());
        }
        let result = self.verify_commit(&request, &before, admission);
        self.restore_execution(facts, executed.is_err());
        match result {
            Ok((identity, status, checked)) => {
                self.failure_verified = status.head == expected.head && checked.index_unchanged;
                match (executed, checked.result) {
                    (Ok(()), Ok(())) => self.effects = Effects::Verified,
                    (Err(error), _) => {
                        self.operation_failure = Some(format!(
                            "Commit child failed; draft retained; inspect its diagnostic: {error}"
                        ))
                    }
                    (_, Err(error)) => self.operation_failure = Some(error),
                }
                Ok(Candidate::Status(identity, Some(status)))
            }
            Err(error) => {
                self.operation_failure = Some(format!(
                    "Commit effects remain unverified; draft retained; inspect HEAD before retrying: {error}"
                ));
                Err(error)
            }
        }
    }

    fn check_commit(
        &mut self,
        observation: &Observation,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let (identity, status) = self.observe(admission)?;
        let actual = check_status(status.as_ref())?;
        let expected = check_status(observation.status.as_ref())?;
        if identity != observation.identity || !same_index(expected, actual) {
            return Err("Commit index or HEAD changed; refresh and reopen composition".into());
        }
        Ok(())
    }

    fn commit_index(&mut self) -> Result<Vec<u8>, String> {
        self.command(&["ls-files", "--stage", "-z"], Phase::Checking, false)
    }

    fn verify_commit(
        &mut self,
        request: &super::super::commit::Request,
        before: &[u8],
        admission: &mut Lease,
    ) -> Result<(Identity, Status, Checked), String> {
        let (identity, status) = self.observe(admission)?;
        let status = status.ok_or("Commit working-tree context disappeared")?;
        let index_unchanged = before == self.commit_index()?;
        let result = self
            .check_commit_object(request, &identity, &status, before, admission)
            .and_then(|()| {
                if index_unchanged {
                    Ok(())
                } else {
                    Err(
                        "Commit index changed; actual HEAD must be inspected before retrying"
                            .into(),
                    )
                }
            });
        Ok((
            identity,
            status,
            Checked {
                index_unchanged,
                result,
            },
        ))
    }

    fn check_commit_object(
        &mut self,
        request: &super::super::commit::Request,
        identity: &Identity,
        status: &Status,
        before: &[u8],
        admission: &mut Lease,
    ) -> Result<(), String> {
        if identity != &request.observation.identity {
            return Err("Commit repository identity changed".into());
        }
        check_status(Some(status))?;
        let expected_head = &check_status(request.observation.status.as_ref())?.head;
        let same_reference = match (expected_head, &status.head) {
            (Head::Unborn { branch: a }, Head::Branch { name: b, .. })
            | (Head::Branch { name: a, .. }, Head::Branch { name: b, .. }) => a == b,
            (Head::Detached { .. }, Head::Detached { .. }) => true,
            _ => false,
        };
        if !same_reference {
            return Err("Commit HEAD reference changed; inspect actual HEAD".into());
        }
        let head = head_object(&status.head).ok_or("Commit did not create a HEAD object")?;
        let old = head_object(&check_status(request.observation.status.as_ref())?.head);
        if Some(head) == old {
            return Err("Commit did not advance the checked HEAD".into());
        }
        let (object, limited) = self.command_os(
            &["cat-file".into(), "commit".into(), head.hex().into()],
            Phase::Verifying,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        if limited {
            return Err("Commit object exceeded its complete-detail cap".into());
        }
        let (parents, message) = commit_object(&object, identity.format)?;
        if parents != old.into_iter().collect::<Vec<_>>() || message != request.message.as_bytes() {
            return Err("Actual commit parents or message differ from the accepted composition; inspect HEAD".into());
        }
        let tree = self
            .command_os(
                &[
                    "ls-tree".into(),
                    "-r".into(),
                    "-z".into(),
                    "--full-tree".into(),
                    head.hex().into(),
                ],
                Phase::Verifying,
                false,
                None,
            )?
            .0;
        // Parse only after admitting both bounded semantic entry lists.
        let count =
            before.iter().filter(|b| **b == 0).count() + tree.iter().filter(|b| **b == 0).count();
        admission.grow(
            2 * (before.len() + tree.len()) + count * 2 * std::mem::size_of::<index::Entry>(),
        )?;
        let expected = index::entries(before, identity.format, true, self.policy.max_status_paths)?;
        let actual = index::entries(&tree, identity.format, false, self.policy.max_status_paths)?;
        if actual != expected {
            return Err(
                "Actual commit tree differs from the checked index; inspect HEAD before retrying"
                    .into(),
            );
        }
        Ok(())
    }
}

struct Checked {
    index_unchanged: bool,
    result: Result<(), String>,
}

fn head_object(head: &Head) -> Option<ObjectId> {
    match head {
        Head::Unborn { .. } => None,
        Head::Branch { object, .. } | Head::Detached { object } => Some(*object),
    }
}

fn commit_object(bytes: &[u8], format: ObjectFormat) -> Result<(Vec<ObjectId>, &[u8]), String> {
    let boundary = bytes
        .windows(2)
        .position(|p| p == b"\n\n")
        .ok_or("Commit object lacks its message boundary")?;
    let mut parents = Vec::new();
    for line in bytes[..boundary].split(|b| *b == b'\n') {
        if let Some(value) = line.strip_prefix(b"parent ") {
            if parents.len() >= 64 {
                return Err("Commit parent metadata exceeds its bound".into());
            }
            parents.push(ObjectId::parse(value, format)?);
        }
    }
    Ok((parents, &bytes[boundary + 2..]))
}
