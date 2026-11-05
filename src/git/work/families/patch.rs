use super::*;
use crate::git::families::{FamilyProof, Patch};
use std::{fs, os::unix::ffi::OsStringExt};

struct MailStep {
    tree: ObjectId,
    author: Vec<u8>,
    email: Vec<u8>,
    date: Vec<u8>,
    message: Vec<u8>,
}
impl Runner {
    pub(super) fn prepare_patch(
        &mut self,
        family: Family,
        args: &mut Vec<OsString>,
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Patch, String> {
        if source.family.is_some()
            || source.refs.head.is_none()
            || source.refs.completeness != Completeness::Complete
            || source.refs.observation.status.as_ref().is_none_or(|s| {
                s.completeness != Completeness::Complete
                    || s.group(Group::Staged).next().is_some()
                    || s.group(Group::Unstaged).next().is_some()
                    || s.group(Group::Unmerged).next().is_some()
            })
        {
            return Err("Patch application requires complete refs, clean tracked saved state and no active sequence; no implicit save or stash".into());
        }
        let commands = self.command(&["--list-cmds=main,others"], Phase::Checking, false)?;
        for required in ["apply", "read-tree", "write-tree", "mailsplit", "mailinfo"] {
            if !commands
                .split(|b| *b == b'\n')
                .any(|s| s == required.as_bytes())
            {
                return Err(format!(
                    "Installed Git lacks patch verification capability: {required}"
                ));
            }
        }
        let path = self.family_destination(args.last().ok_or("Patch input path required")?)?;
        let stamp = fingerprint(&fs::symlink_metadata(&path).map_err(|e| e.to_string())?);
        let bytes = self
            .family_file(&path)?
            .ok_or("Patch input file is absent")?;
        admission.grow(bytes.capacity())?;
        if bytes.is_empty() {
            return Err("Patch input must contain complete ordinary file bytes".into());
        }
        // Git's own split/decode happens only after consent. This conservative
        // envelope bound admits at most 256 producer files, even for bad input.
        let mails = if family == Family::ApplyMailbox {
            let count = bytes
                .split(|b| *b == b'\n')
                .filter(|line| line.starts_with(b"From "))
                .count();
            if count == 0 || count > self.policy.max_refs.min(256) {
                return Err("Mailbox input exceeds its complete bounded envelope admission".into());
            }
            count
        } else {
            0
        };
        let configuration = self.transport_config(admission)?;
        let hash = self.patch_command(&["hash-object", "--no-filters", "--stdin"], Some(&bytes))?;
        let hash = ObjectId::parse(
            hash.strip_suffix(b"\n").ok_or("Patch hash is incomplete")?,
            source.refs.observation.identity.format,
        )?;
        *args.last_mut().expect("patch input") = "-".into();
        args.insert(0, "--no-replace-objects".into());
        if family == Family::ApplyMailbox {
            args.splice(0..0, ["-c".into(), "i18n.commitEncoding=UTF-8".into()]);
        } else {
            let mut check = args.clone();
            check.insert(2, "--check".into());
            self.command_input(
                &check,
                Phase::Checking,
                false,
                None,
                crate::process::input::stage(Some(&bytes))?,
            )?;
        }
        self.checked_sequence(source, admission)?;
        if self.transport_config(admission)? != configuration
            || fingerprint(&fs::symlink_metadata(&path).map_err(|e| e.to_string())?) != stamp
            || self.family_file(&path)?.as_ref() != Some(&bytes)
        {
            return Err("Patch source, input or configuration changed during preparation".into());
        }
        Ok(Patch {
            configuration,
            path,
            stamp,
            bytes,
            hash,
            mails,
        })
    }
    fn patch_command(&mut self, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        self.command_input(
            &args,
            Phase::Checking,
            false,
            None,
            crate::process::input::stage(input)?,
        )
        .map(|(b, _)| b)
    }
    /// The override belongs to this single child and is never inherited by the
    /// real mutator or a later observation. Its directory FD stays child-owned.
    fn patch_index(&mut self, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        self.private_index = Some(
            self.workspace
                .as_ref()
                .ok_or("Patch workspace missing")?
                .entry("index")
                .map_err(|e| e.to_string())?,
        );
        let result = self.patch_command(args, input);
        self.private_index = None;
        result
    }
    fn patch_tree(&mut self, format: ObjectFormat) -> Result<ObjectId, String> {
        let bytes = self.patch_index(&["--no-replace-objects", "write-tree"], None)?;
        self.workspace
            .as_ref()
            .expect("patch workspace")
            .read("index", budget::SCRATCH_BYTES)
            .map_err(|e| e.to_string())?;
        ObjectId::parse(
            bytes
                .strip_suffix(b"\n")
                .ok_or("Private index tree is incomplete")?,
            format,
        )
    }
    fn patch_expected(
        &mut self,
        bytes: &[u8],
        format: ObjectFormat,
    ) -> Result<Option<ObjectId>, String> {
        match self.patch_index(
            &[
                "--no-replace-objects",
                "apply",
                "--cached",
                "--binary",
                "--whitespace=nowarn",
                "--",
                "-",
            ],
            Some(bytes),
        ) {
            Ok(_) => self.patch_tree(format).map(Some),
            Err(_)
                if self.end == capture::End::Exited
                    && self.exit.is_some_and(|s| s.code() == Some(1))
                    && self.retained.is_none() =>
            {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
    fn patch_input_current(&mut self, input: &Patch, admission: &mut Lease) -> Result<(), String> {
        if fingerprint(&fs::symlink_metadata(&input.path).map_err(|e| e.to_string())?)
            != input.stamp
            || self.family_file(&input.path)?.as_ref() != Some(&input.bytes)
            || self.transport_config(admission)? != input.configuration
        {
            return Err("Patch input file or effective configuration changed during review".into());
        }
        Ok(())
    }
    fn mailbox_steps(
        &mut self,
        input: &Patch,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<(Vec<MailStep>, usize), String> {
        let workspace = self
            .workspace
            .as_ref()
            .ok_or("Mailbox workspace missing")?
            .clone();
        let mut output = OsString::from("-o");
        output.push(workspace.anchor());
        let args = vec![
            "mailsplit".into(),
            "-b".into(),
            "-d4".into(),
            "--mboxrd".into(),
            output,
            "--".into(),
            "-".into(),
        ];
        let bytes = self
            .command_input(
                &args,
                Phase::Checking,
                false,
                None,
                crate::process::input::stage(Some(&input.bytes))?,
            )?
            .0;
        let count: usize = std::str::from_utf8(&bytes)
            .map_err(|e| e.to_string())?
            .trim()
            .parse()
            .map_err(|_| "Mailbox count is not a complete integer")?;
        if count == 0 || count > input.mails {
            return Err("Actual mailbox split exceeds reviewed envelope admission".into());
        }
        let limit = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        let mut steps = Vec::new();
        for index in 1..=count {
            let mail = workspace
                .read(&format!("{index:04}"), limit)
                .map_err(|e| e.to_string())?;
            let args = vec![
                "mailinfo".into(),
                "--encoding=UTF-8".into(),
                "--no-scissors".into(),
                workspace
                    .entry("message")
                    .map_err(|e| e.to_string())?
                    .into_os_string(),
                workspace
                    .entry("patch")
                    .map_err(|e| e.to_string())?
                    .into_os_string(),
            ];
            let info = self
                .command_input(
                    &args,
                    Phase::Checking,
                    false,
                    None,
                    crate::process::input::stage(Some(&mail))?,
                )?
                .0;
            let field = |prefix: &[u8]| -> Result<Vec<u8>, String> {
                let fields: Vec<_> = info
                    .split(|b| *b == b'\n')
                    .filter_map(|line| line.strip_prefix(prefix))
                    .collect();
                if fields.len() != 1
                    || fields[0].is_empty()
                    || fields[0].len() > 4096
                    || fields[0].contains(&0)
                {
                    return Err(
                        "Mailbox metadata is absent, ambiguous or exceeds complete admission"
                            .into(),
                    );
                }
                Ok(fields[0].to_vec())
            };
            let author = field(b"Author: ")?;
            let email = field(b"Email: ")?;
            let subject = field(b"Subject: ")?;
            let date = field(b"Date: ")?;
            let date_arg = OsString::from_vec([b"--since=".as_slice(), &date].concat());
            let timestamp = self
                .command_os(
                    &["rev-parse".into(), date_arg],
                    Phase::Checking,
                    false,
                    None,
                )?
                .0;
            let date = timestamp
                .strip_prefix(b"--max-age=")
                .and_then(|s| s.strip_suffix(b"\n"))
                .filter(|s| !s.is_empty() && s.iter().all(u8::is_ascii_digit))
                .ok_or("Mailbox author date is not a complete timestamp")?
                .to_vec();
            let mut message = subject;
            message.extend_from_slice(b"\n\n");
            message.extend(
                workspace
                    .read("message", limit)
                    .map_err(|e| e.to_string())?,
            );
            if message.len() > limit {
                return Err("Mailbox message exceeds complete scratch admission".into());
            }
            let message = self.patch_command(&["stripspace"], Some(&message))?;
            let patch = workspace.read("patch", limit).map_err(|e| e.to_string())?;
            let Some(tree) = self.patch_expected(&patch, format)? else {
                break;
            };
            admission.grow(
                std::mem::size_of::<MailStep>()
                    + author.capacity()
                    + email.capacity()
                    + date.capacity()
                    + message.capacity(),
            )?;
            steps.push(MailStep {
                tree,
                author,
                email,
                date,
                message,
            });
        }
        Ok((steps, count))
    }
    fn patch_refs_preserved(source: &Sequence, actual: &Sequence) -> Result<(), String> {
        let before = &source.refs;
        let after = &actual.refs;
        let branch = before.head_ref.as_slice();
        if after.completeness != Completeness::Complete
            || after.head_ref != before.head_ref
            || after.remotes != before.remotes
            || before
                .entries
                .iter()
                .filter(|r| r.name != branch)
                .any(|r| after.reference(&r.name) != Some(r))
            || after
                .entries
                .iter()
                .filter(|r| r.name != branch)
                .any(|r| before.reference(&r.name) != Some(r))
        {
            return Err(
                "Patch result changed another ref, branch or complete reference evidence".into(),
            );
        }
        Ok(())
    }
    fn mailbox_lineage(
        &mut self,
        source: &Sequence,
        actual: &Sequence,
        steps: &[MailStep],
    ) -> Result<usize, String> {
        Self::patch_refs_preserved(source, actual)?;
        let before = source.refs.head.ok_or("Mailbox source HEAD missing")?;
        let after = actual.refs.head.ok_or("Mailbox result HEAD missing")?;
        let range = format!("{}..{}", before.hex(), after.hex());
        let max = format!("--max-count={}", steps.len() + 1);
        let commits = self.patch_command(
            &[
                "--no-replace-objects",
                "rev-list",
                "--reverse",
                "--topo-order",
                &max,
                &range,
            ],
            None,
        )?;
        let commits: Vec<_> = commits
            .split(|b| *b == b'\n')
            .filter(|s| !s.is_empty())
            .collect();
        if commits.len() > steps.len() {
            return Err("Mailbox produced an unpredicted commit chain".into());
        }
        let mut parent = before;
        for (commit, expected) in commits.iter().zip(steps) {
            let object = ObjectId::parse(commit, before.format())?;
            let bytes = self.patch_command(
                &[
                    "--no-replace-objects",
                    "show",
                    "--no-patch",
                    "--format=format:%T%x00%P%x00%an%x00%ae%x00%at%x00%B",
                    &object.hex(),
                ],
                None,
            )?;
            let fields: Vec<_> = bytes.splitn(6, |b| *b == 0).collect();
            if fields.len() != 6
                || fields[0] != expected.tree.hex().as_bytes()
                || fields[1] != parent.hex().as_bytes()
                || fields[2] != expected.author
                || fields[3] != expected.email
                || fields[4] != expected.date
                || fields[5] != expected.message
            {
                return Err("Actual mailbox commit tree, single parent, author, date or message differs from independently decoded patch semantics".into());
            }
            parent = object;
        }
        if parent != after {
            return Err("Mailbox result is not the exact predicted linear lineage".into());
        }
        Ok(commits.len())
    }
    pub(super) fn execute_patch(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Patch(input) = &plan.proof else {
            return Err("Patch proof unavailable".into());
        };
        self.patch_input_current(input, admission)?;
        let mut names = vec![
            "index".into(),
            "index.lock".into(),
            "message".into(),
            "patch".into(),
        ];
        names.extend((1..=input.mails).map(|i| OsString::from(format!("{i:04}"))));
        let size = 4096
            + names.capacity() * std::mem::size_of::<OsString>()
            + names.iter().map(OsString::capacity).sum::<usize>();
        admission.grow(size)?;
        self.driver_admission = Some(admission.split(size));
        let workspace = Arc::new(
            crate::file::temporary::Directory::new(&std::env::temp_dir(), names)
                .map_err(|e| format!("Create owned patch workspace: {e}"))?,
        );
        self.workspace = Some(workspace.clone());
        let result = self.apply_patch(&plan, admission);
        self.private_index = None;
        self.workspace = None;
        if self.retained.is_none()
            && let Err(e) = workspace.clean()
        {
            self.effects = Effects::Possible;
            self.operation_failure = Some(format!(
                "Patch workspace cleanup is unverified: {e}; unknown entries preserved"
            ));
            return Err(format!("Patch workspace cleanup: {e}"));
        }
        if let Err(e) = &result
            && self.effects == Effects::Possible
        {
            self.operation_failure = Some(format!(
                "Patch preparation/execution has possible object effects: {e}; no retry or rollback"
            ));
        }
        result
    }
    fn apply_patch(
        &mut self,
        plan: &Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Patch(input) = &plan.proof else {
            return Err("Patch proof unavailable".into());
        };
        let source = plan.source.as_ref().ok_or("Patch source unavailable")?;
        let before = source.refs.head.ok_or("Patch HEAD unavailable")?;
        // Build expected trees from decoded bytes in a private index. Object
        // writes begin only after consent; mailbox From labels are not proof.
        self.effects = Effects::Possible;
        self.patch_index(&["--no-replace-objects", "read-tree", &before.hex()], None)?;
        let (steps, count, tree) = if plan.family == Family::ApplyMailbox {
            let (steps, count) = self.mailbox_steps(input, before.format(), admission)?;
            (steps, count, None)
        } else {
            (
                Vec::new(),
                0,
                Some(
                    self.patch_expected(&input.bytes, before.format())?
                        .ok_or("Plain patch does not apply to the reviewed index")?,
                ),
            )
        };
        self.checked_sequence(source, admission)?;
        if self.stash_evidence(source, admission)? != plan.evidence {
            return Err("Patch source changed during independent prediction".into());
        }
        self.patch_input_current(input, admission)?;
        let (executed, facts) = self.execute_change(
            &plan.args,
            crate::process::input::stage(Some(&input.bytes))?,
        );
        let failed = executed.is_err();
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Patch child cleanup remains owned; no retry, index restoration or workspace deletion".into());
        }
        let verified = (|| {
            let refs = self.integration_refs(&source.refs.observation, admission)?;
            let actual = self.read_sequence(refs, admission)?;
            if actual.family.is_some() {
                self.in_progress = actual.family == Some(SequenceFamily::Mailbox)
                    && actual.original == Some(before)
                    && self.mailbox_lineage(source, &actual, &steps).is_ok();
                self.operation_failure = Some("Patch stopped in actual Git-owned sequence state; inspect operation status; no automatic continue, abort or retry".into());
                return Ok(Candidate::Sequence(actual));
            }
            executed?;
            Self::patch_refs_preserved(source, &actual)?;
            let expected = if plan.family == Family::ApplyMailbox {
                if steps.len() != count || self.mailbox_lineage(source, &actual, &steps)? != count {
                    return Err(
                        "Mailbox did not produce every independently predicted commit".into(),
                    );
                }
                steps.last().ok_or("Mailbox produced no commit")?.tree
            } else {
                if actual.refs.head != Some(before) {
                    return Err("Plain patch changed HEAD".into());
                }
                tree.ok_or("Plain patch expected tree missing")?
            };
            let bytes = self.patch_command(&["--no-replace-objects", "write-tree"], None)?;
            if bytes != format!("{}\n", expected.hex()).as_bytes() {
                return Err("Actual index differs from independently applied patch tree".into());
            }
            self.patch_command(
                &[
                    "diff-files",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--ignore-submodules=none",
                    "--quiet",
                    "--",
                ],
                None,
            )?;
            if self.stash_evidence(&actual, admission)?[2] != plan.evidence[2]
                || self.transport_config(admission)? != input.configuration
            {
                return Err(
                    "Patch changed unrelated untracked sources or effective configuration".into(),
                );
            }
            self.effects = Effects::Verified;
            self.family_observed_result(
                plan.clone(),
                (plan.family == Family::ApplyMailbox)
                    .then_some(actual.refs.head)
                    .flatten(),
                admission,
            )
        })();
        if let Err(e) = &verified {
            self.operation_failure = Some(format!(
                "Patch effects remain unverified: {e}; no retry or rollback"
            ));
        }
        let result = match verified {
            Ok(candidate) => Ok(candidate),
            Err(_) => self.family_observed_result(plan.clone(), None, admission),
        };
        self.restore_execution(facts, failed);
        result
    }
}
