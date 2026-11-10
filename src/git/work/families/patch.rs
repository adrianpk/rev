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
        check_patch_source(source)?;
        self.patch_capabilities()?;
        let input = self.patch_file(family, args, admission)?;
        let configuration = self.transport_config(admission)?;
        let hash = self.patch_hash(&input.bytes, source.refs.observation.identity.format)?;
        self.patch_arguments(family, args, &input.bytes)?;
        self.check_prepared_patch(source, &input, &configuration, admission)?;
        Ok(Patch {
            configuration,
            path: input.path,
            stamp: input.stamp,
            bytes: input.bytes,
            hash,
            mails: input.mails,
        })
    }
    fn patch_command(&mut self, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        self.command_argv(
            self.git_invocation(&args, crate::process::input::stage(input)?),
            Phase::Checking,
            false,
            None,
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
        let workspace = Arc::clone(self.workspace.as_ref().ok_or("Mailbox workspace missing")?);
        let count = self.mailbox_split(input, &workspace)?;
        let limit = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        let decoder = MailDecoder { workspace, limit };
        let mut steps = Vec::new();
        for index in 1..=count {
            let DecodedMail {
                author,
                email,
                date,
                message,
                patch,
            } = decoder.decode(self, index)?;
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
            let object = self.check_mailbox_commit(commit, parent, expected)?;
            parent = object;
        }
        if parent != after {
            return Err("Mailbox result is not the exact predicted linear lineage".into());
        }
        Ok(commits.len())
    }
    pub(super) fn execute_patch(
        &mut self,
        plan: &Arc<FamilyPlan>,
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
        self.workspace = Some(std::sync::Arc::clone(&workspace));
        let result = self.apply_patch(plan, admission);
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
        let prediction = self.predict_patch(plan.family, input, before, admission)?;
        PatchChange {
            plan,
            input,
            source,
            prediction,
        }
        .run(self, admission)
    }
    fn predict_patch(
        &mut self,
        family: Family,
        input: &Patch,
        before: ObjectId,
        admission: &mut Lease,
    ) -> Result<Prediction, String> {
        // Predict in the private index only after consent, before the real child.
        self.effects = Effects::Possible;
        self.patch_index(&["--no-replace-objects", "read-tree", &before.hex()], None)?;
        let (steps, count, tree) = if family == Family::ApplyMailbox {
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
        Ok(Prediction {
            before,
            steps,
            count,
            tree,
        })
    }
    fn patch_capabilities(&mut self) -> Result<(), String> {
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
        Ok(())
    }
    fn patch_file(
        &mut self,
        family: Family,
        args: &[OsString],
        admission: &mut Lease,
    ) -> Result<PatchFile, String> {
        let path = self.family_destination(args.last().ok_or("Patch input path required")?)?;
        let stamp = fingerprint(&fs::symlink_metadata(&path).map_err(|e| e.to_string())?);
        let bytes = self
            .family_file(&path)?
            .ok_or("Patch input file is absent")?;
        admission.grow(bytes.capacity())?;
        if bytes.is_empty() {
            return Err("Patch input must contain complete ordinary file bytes".into());
        }
        let mails = mailbox_count(family, &bytes, self.policy.max_refs.min(256))?;
        Ok(PatchFile {
            path,
            stamp,
            bytes,
            mails,
        })
    }
    fn patch_hash(&mut self, bytes: &[u8], format: ObjectFormat) -> Result<ObjectId, String> {
        let hash = self.patch_command(&["hash-object", "--no-filters", "--stdin"], Some(bytes))?;
        let hash = ObjectId::parse(
            hash.strip_suffix(b"\n").ok_or("Patch hash is incomplete")?,
            format,
        )?;
        Ok(hash)
    }
    fn patch_arguments(
        &mut self,
        family: Family,
        args: &mut Vec<OsString>,
        bytes: &[u8],
    ) -> Result<(), String> {
        *args.last_mut().expect("patch input") = "-".into();
        args.insert(0, "--no-replace-objects".into());
        if family == Family::ApplyMailbox {
            args.splice(0..0, ["-c".into(), "i18n.commitEncoding=UTF-8".into()]);
        } else {
            let mut check = args.clone();
            check.insert(2, "--check".into());
            self.command_argv(
                self.git_invocation(&check, crate::process::input::stage(Some(bytes))?),
                Phase::Checking,
                false,
                None,
            )?;
        }
        Ok(())
    }
    fn check_prepared_patch(
        &mut self,
        source: &Sequence,
        input: &PatchFile,
        configuration: &[u8],
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.checked_sequence(source, admission)?;
        if self.transport_config(admission)? != configuration
            || fingerprint(&fs::symlink_metadata(&input.path).map_err(|e| e.to_string())?)
                != input.stamp
            || self.family_file(&input.path)?.as_ref() != Some(&input.bytes)
        {
            return Err("Patch source, input or configuration changed during preparation".into());
        }
        Ok(())
    }
    fn mailbox_split(
        &mut self,
        input: &Patch,
        workspace: &crate::file::temporary::Directory,
    ) -> Result<usize, String> {
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
            .command_argv(
                self.git_invocation(&args, crate::process::input::stage(Some(&input.bytes))?),
                Phase::Checking,
                false,
                None,
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
        Ok(count)
    }
    fn mailbox_date(&mut self, date: &[u8]) -> Result<Vec<u8>, String> {
        let date_arg = OsString::from_vec([b"--since=".as_slice(), date].concat());
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
        Ok(date)
    }
    fn check_mailbox_commit(
        &mut self,
        commit: &[u8],
        parent: ObjectId,
        expected: &MailStep,
    ) -> Result<ObjectId, String> {
        let object = ObjectId::parse(commit, parent.format())?;
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
        Ok(object)
    }
}

/// Complete input bytes retain the reviewed file identity and envelope bound.
struct PatchFile {
    path: PathBuf,
    stamp: [u64; 8],
    bytes: Vec<u8>,
    mails: usize,
}

fn check_patch_source(source: &Sequence) -> Result<(), String> {
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
    Ok(())
}
fn mailbox_count(family: Family, bytes: &[u8], limit: usize) -> Result<usize, String> {
    // Git's own split/decode happens only after consent. This conservative
    // envelope bound admits at most 256 producer files, even for bad input.
    let mails = if family == Family::ApplyMailbox {
        let count = bytes
            .split(|b| *b == b'\n')
            .filter(|line| line.starts_with(b"From "))
            .count();
        if count == 0 || count > limit {
            return Err("Mailbox input exceeds its complete bounded envelope admission".into());
        }
        count
    } else {
        0
    };
    Ok(mails)
}

/// Mailinfo scratch files stay bound to the owned directory and detail cap.
struct MailDecoder {
    workspace: Arc<crate::file::temporary::Directory>,
    limit: usize,
}
struct MailHeaders {
    author: Vec<u8>,
    email: Vec<u8>,
    subject: Vec<u8>,
    date: Vec<u8>,
}
struct DecodedMail {
    author: Vec<u8>,
    email: Vec<u8>,
    date: Vec<u8>,
    message: Vec<u8>,
    patch: Vec<u8>,
}
impl MailDecoder {
    fn decode(&self, runner: &mut Runner, index: usize) -> Result<DecodedMail, String> {
        let info = self.info(runner, index)?;
        let MailHeaders {
            author,
            email,
            subject,
            date,
        } = mail_headers(&info)?;
        let date = runner.mailbox_date(&date)?;
        let message = self.message(runner, subject)?;
        let patch = self
            .workspace
            .read("patch", self.limit)
            .map_err(|e| e.to_string())?;
        Ok(DecodedMail {
            author,
            email,
            date,
            message,
            patch,
        })
    }
    fn info(&self, runner: &mut Runner, index: usize) -> Result<Vec<u8>, String> {
        let mail = self
            .workspace
            .read(&format!("{index:04}"), self.limit)
            .map_err(|e| e.to_string())?;
        let args = vec![
            "mailinfo".into(),
            "--encoding=UTF-8".into(),
            "--no-scissors".into(),
            self.workspace
                .entry("message")
                .map_err(|e| e.to_string())?
                .into_os_string(),
            self.workspace
                .entry("patch")
                .map_err(|e| e.to_string())?
                .into_os_string(),
        ];
        let info = runner
            .command_argv(
                runner.git_invocation(&args, crate::process::input::stage(Some(&mail))?),
                Phase::Checking,
                false,
                None,
            )?
            .0;
        Ok(info)
    }
    fn message(&self, runner: &mut Runner, subject: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut message = subject;
        message.extend_from_slice(b"\n\n");
        message.extend(
            self.workspace
                .read("message", self.limit)
                .map_err(|e| e.to_string())?,
        );
        if message.len() > self.limit {
            return Err("Mailbox message exceeds complete scratch admission".into());
        }
        let message = runner.patch_command(&["stripspace"], Some(&message))?;
        Ok(message)
    }
}
fn mail_headers(info: &[u8]) -> Result<MailHeaders, String> {
    let author = mailbox_field(info, b"Author: ")?;
    let email = mailbox_field(info, b"Email: ")?;
    let subject = mailbox_field(info, b"Subject: ")?;
    let date = mailbox_field(info, b"Date: ")?;
    Ok(MailHeaders {
        author,
        email,
        subject,
        date,
    })
}
fn mailbox_field(info: &[u8], prefix: &[u8]) -> Result<Vec<u8>, String> {
    let fields: Vec<_> = info
        .split(|b| *b == b'\n')
        .filter_map(|line| line.strip_prefix(prefix))
        .collect();
    if fields.len() != 1 || fields[0].is_empty() || fields[0].len() > 4096 || fields[0].contains(&0)
    {
        return Err("Mailbox metadata is absent, ambiguous or exceeds complete admission".into());
    }
    Ok(fields[0].to_vec())
}

/// Private-index prediction binds exact trees, message semantics and chain count.
struct Prediction {
    before: ObjectId,
    steps: Vec<MailStep>,
    count: usize,
    tree: Option<ObjectId>,
}

/// The checked input, source and independently predicted effects share one child.
struct PatchChange<'a> {
    plan: &'a Arc<FamilyPlan>,
    input: &'a Patch,
    source: &'a Arc<Sequence>,
    prediction: Prediction,
}
impl PatchChange<'_> {
    fn run(&self, runner: &mut Runner, admission: &mut Lease) -> Result<Candidate, String> {
        let plan = self.plan;
        let input = self.input;
        self.check_source(runner, admission)?;
        let (executed, facts) = runner.execute_change(
            &plan.args,
            crate::process::input::stage(Some(&input.bytes))?,
        );
        let failed = executed.is_err();
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err("Patch child cleanup remains owned; no retry, index restoration or workspace deletion".into());
        }
        let verified = self.verify(runner, executed, admission);
        if let Err(e) = &verified {
            runner.operation_failure = Some(format!(
                "Patch effects remain unverified: {e}; no retry or rollback"
            ));
        }
        let result = match verified {
            Ok(candidate) => Ok(candidate),
            Err(_) => runner.family_observed_result(std::sync::Arc::clone(plan), None, admission),
        };
        runner.restore_execution(facts, failed);
        result
    }
    fn check_source(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let plan = self.plan;
        let input = self.input;
        let source = self.source;
        runner.checked_sequence(source, admission)?;
        if runner.stash_evidence(source, admission)? != plan.evidence {
            return Err("Patch source changed during independent prediction".into());
        }
        runner.patch_input_current(input, admission)?;
        Ok(())
    }
    fn verify(
        &self,
        runner: &mut Runner,
        executed: Result<(), String>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let plan = self.plan;
        let source = self.source;
        let refs = runner.integration_refs(&source.refs.observation, admission)?;
        let actual = runner.read_sequence(&refs, admission)?;
        if actual.family.is_some() {
            return Ok(self.stopped(runner, actual));
        }
        executed?;
        Runner::patch_refs_preserved(source, &actual)?;
        let expected = self.expected_tree(runner, &actual)?;
        self.check_tree(runner, &actual, expected, admission)?;
        runner.effects = Effects::Verified;
        runner.family_observed_result(
            std::sync::Arc::clone(plan),
            (plan.family == Family::ApplyMailbox)
                .then_some(actual.refs.head)
                .flatten(),
            admission,
        )
    }
    fn stopped(&self, runner: &mut Runner, actual: Sequence) -> Candidate {
        let source = self.source;
        let before = self.prediction.before;
        let steps = &self.prediction.steps;
        runner.in_progress = actual.family == Some(SequenceFamily::Mailbox)
            && actual.original == Some(before)
            && runner.mailbox_lineage(source, &actual, steps).is_ok();
        runner.operation_failure = Some("Patch stopped in actual Git-owned sequence state; inspect operation status; no automatic continue, abort or retry".into());
        Candidate::Sequence(actual)
    }
    fn expected_tree(&self, runner: &mut Runner, actual: &Sequence) -> Result<ObjectId, String> {
        let plan = self.plan;
        let source = self.source;
        let before = self.prediction.before;
        let steps = &self.prediction.steps;
        let count = self.prediction.count;
        let tree = self.prediction.tree;
        let expected = if plan.family == Family::ApplyMailbox {
            if steps.len() != count || runner.mailbox_lineage(source, actual, steps)? != count {
                return Err("Mailbox did not produce every independently predicted commit".into());
            }
            steps.last().ok_or("Mailbox produced no commit")?.tree
        } else {
            if actual.refs.head != Some(before) {
                return Err("Plain patch changed HEAD".into());
            }
            tree.ok_or("Plain patch expected tree missing")?
        };
        Ok(expected)
    }
    fn check_tree(
        &self,
        runner: &mut Runner,
        actual: &Sequence,
        expected: ObjectId,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let plan = self.plan;
        let input = self.input;
        let bytes = runner.patch_command(&["--no-replace-objects", "write-tree"], None)?;
        if bytes != format!("{}\n", expected.hex()).as_bytes() {
            return Err("Actual index differs from independently applied patch tree".into());
        }
        runner.patch_command(
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
        if runner.stash_evidence(actual, admission)?[2] != plan.evidence[2]
            || runner.transport_config(admission)? != input.configuration
        {
            return Err(
                "Patch changed unrelated untracked sources or effective configuration".into(),
            );
        }
        Ok(())
    }
}
