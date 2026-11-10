use super::*;

impl Runner {
    pub(super) fn prepare_application(
        &mut self,
        history: Arc<History>,
        kind: ApplyKind,
        choices: &[RevisionChoice],
        admission: &mut Lease,
    ) -> Result<ApplyPlan, String> {
        self.check_application_history(&history, admission)?;
        let refs = self.application_refs(&history, admission)?;
        let bytes = self.application_objects(&history, choices, admission)?;
        admission
            .grow(bytes.len() * 3 + choices.len() * std::mem::size_of::<ApplyStep>() + 65536)?;
        let steps = SourceBatch {
            remaining: &bytes,
            limit: self.policy.max_detail_bytes,
        }
        .steps(&history, choices)?;
        self.check_integration_source(&refs, admission)?;
        self.verify_ancestry(&history)?;
        let mut plan = ApplyPlan {
            refs,
            history,
            kind,
            steps,
            _admission: admission.split(0),
        };
        admission.grow(plan.bytes())?;
        plan._admission = admission.split(plan.bytes());
        Ok(plan)
    }
    fn check_application_history(
        &mut self,
        history: &History,
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.verify_ancestry(history)?;
        let (identity, status) = self.observe(admission)?;
        if identity != history.observation.identity
            || status != history.observation.status
            || status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Ordered revisions require unchanged complete clean History/Status and working tree".into());
        }
        Ok(())
    }
    fn application_refs(
        &mut self,
        history: &History,
        admission: &mut Lease,
    ) -> Result<Arc<Refs>, String> {
        let refs =
            Arc::new(self.read_refs(std::sync::Arc::clone(&history.observation), admission)?);
        if refs.head.is_none()
            || refs.head_ref.is_empty()
            || refs.completeness != Completeness::Complete
        {
            return Err(
                "Ordered revisions require complete Refs and one checked current branch".into(),
            );
        }
        if self.read_sequence(&refs, admission)?.family.is_some() {
            return Err("A Git-owned sequence is already active".into());
        }
        Ok(refs)
    }
    fn application_objects(
        &mut self,
        history: &History,
        choices: &[RevisionChoice],
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let size = choices.len()
            * (match history.observation.identity.format {
                ObjectFormat::Sha1 => 40,
                ObjectFormat::Sha256 => 64,
            } + 1);
        admission.grow(size * 2)?;
        let mut input = Vec::with_capacity(size);
        for choice in choices {
            input.extend_from_slice(choice.object.hex().as_bytes());
            input.push(b'\n');
        }
        let file = crate::process::input::stage(Some(&input))?;
        self.driver_admission = Some(admission.split(size));
        let output = self.command_argv(
            self.git_invocation(
                &[
                    "--no-replace-objects".into(),
                    "cat-file".into(),
                    "--batch".into(),
                ],
                file,
            ),
            Phase::Checking,
            false,
            None,
        );
        if self.retained.is_none() {
            self.driver_admission = None;
        }
        let (bytes, _) = output?;
        Ok(bytes)
    }
}

/// Complete batch framing is consumed once, in the captured revision order.
struct SourceBatch<'a> {
    remaining: &'a [u8],
    limit: usize,
}
impl<'a> SourceBatch<'a> {
    fn steps(
        mut self,
        history: &History,
        choices: &[RevisionChoice],
    ) -> Result<Vec<ApplyStep>, String> {
        let mut steps = Vec::with_capacity(choices.len());
        for choice in choices {
            let raw = self.take(choice)?;
            steps.push(source_step(raw, history, choice)?);
        }
        if !self.remaining.is_empty() {
            return Err("Source commit batch has unexpected trailing records".into());
        }
        Ok(steps)
    }
    fn take(&mut self, choice: &RevisionChoice) -> Result<&'a [u8], String> {
        let header = source_header(self.remaining, choice, self.limit)?;
        self.remaining = &self.remaining[header.offset..];
        let raw = self
            .remaining
            .get(..header.length)
            .ok_or("Incomplete source commit batch body")?;
        if self.remaining.get(header.length) != Some(&b'\n') {
            return Err("Source commit batch lacks its terminator".into());
        }
        self.remaining = &self.remaining[header.length + 1..];
        Ok(raw)
    }
}
struct Header {
    offset: usize,
    length: usize,
}
fn source_header(bytes: &[u8], choice: &RevisionChoice, limit: usize) -> Result<Header, String> {
    let newline = bytes
        .iter()
        .position(|b| *b == b'\n')
        .ok_or("Incomplete source commit batch header")?;
    let mut fields = bytes[..newline].split(|b| *b == b' ');
    let (Some(object), Some(kind), Some(length)) = (fields.next(), fields.next(), fields.next())
    else {
        return Err("Incomplete source commit batch header fields".into());
    };
    if fields.next().is_some() || object != choice.object.hex().as_bytes() || kind != b"commit" {
        return Err("Ordered source is missing or is not the exact captured commit".into());
    }
    let length = std::str::from_utf8(length)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n <= limit)
        .ok_or("Ordered source exceeds complete-detail admission")?;
    Ok(Header {
        offset: newline + 1,
        length,
    })
}
fn check_source_parents(raw: &[u8]) -> Result<(), String> {
    let headers = raw.split(|b| *b == b'\n').take_while(|l| !l.is_empty());
    if headers
        .filter_map(|l| l.strip_prefix(b"parent "))
        .map(|p| p.len() + 1)
        .sum::<usize>()
        > 4096
    {
        return Err("Ordered source parent metadata exceeds 4096-byte admission".into());
    }
    Ok(())
}
fn source_step(
    raw: &[u8],
    history: &History,
    choice: &RevisionChoice,
) -> Result<ApplyStep, String> {
    check_source_parents(raw)?;
    let (parents, message) = commit::commit_object(raw, history.observation.identity.format)?;
    let revision = history
        .revision(choice.object)
        .ok_or("Ordered revision expired")?;
    if parents != revision.parents || message.len() > MAX_COMMIT_BYTES || message.contains(&0) {
        return Err("Source parents changed or complete message exceeds native admission".into());
    }
    let author = raw
        .split(|b| *b == b'\n')
        .find_map(|l| l.strip_prefix(b"author "))
        .filter(|s| s.len() <= 4096)
        .ok_or("Source author metadata is missing or exceeds admission")?;
    let parent = choice.mainline.map(|n| parents[n - 1]);
    Ok(ApplyStep {
        object: choice.object,
        mainline: choice.mainline,
        parent,
        subject: revision.subject.clone(),
        message: message.into(),
        author: author.into(),
    })
}
