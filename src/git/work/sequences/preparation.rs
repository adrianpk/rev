use super::*;

impl Runner {
    pub(super) fn prepare_application(
        &mut self,
        history: Arc<History>,
        kind: ApplyKind,
        choices: &[RevisionChoice],
        admission: &mut Lease,
    ) -> Result<ApplyPlan, String> {
        self.verify_ancestry(&history)?;
        let (identity, status) = self.observe(admission)?;
        if identity != history.observation.identity
            || status != history.observation.status
            || status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Ordered revisions require unchanged complete clean History/Status and working tree".into());
        }
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
        admission
            .grow(bytes.len() * 3 + choices.len() * std::mem::size_of::<ApplyStep>() + 65536)?;
        let mut remaining = bytes.as_slice();
        let mut steps = Vec::with_capacity(choices.len());
        for choice in choices {
            let newline = remaining
                .iter()
                .position(|b| *b == b'\n')
                .ok_or("Incomplete source commit batch header")?;
            let mut fields = remaining[..newline].split(|b| *b == b' ');
            let (Some(object), Some(kind), Some(length)) =
                (fields.next(), fields.next(), fields.next())
            else {
                return Err("Incomplete source commit batch header fields".into());
            };
            if fields.next().is_some()
                || object != choice.object.hex().as_bytes()
                || kind != b"commit"
            {
                return Err("Ordered source is missing or is not the exact captured commit".into());
            }
            let length = std::str::from_utf8(length)
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|n| *n <= self.policy.max_detail_bytes)
                .ok_or("Ordered source exceeds complete-detail admission")?;
            remaining = &remaining[newline + 1..];
            let raw = remaining
                .get(..length)
                .ok_or("Incomplete source commit batch body")?;
            if remaining.get(length) != Some(&b'\n') {
                return Err("Source commit batch lacks its terminator".into());
            }
            remaining = &remaining[length + 1..];
            let headers = raw.split(|b| *b == b'\n').take_while(|l| !l.is_empty());
            if headers
                .filter_map(|l| l.strip_prefix(b"parent "))
                .map(|p| p.len() + 1)
                .sum::<usize>()
                > 4096
            {
                return Err("Ordered source parent metadata exceeds 4096-byte admission".into());
            }
            let (parents, message) =
                commit::commit_object(raw, history.observation.identity.format)?;
            let revision = history
                .revision(choice.object)
                .ok_or("Ordered revision expired")?;
            if parents != revision.parents
                || message.len() > MAX_COMMIT_BYTES
                || message.contains(&0)
            {
                return Err(
                    "Source parents changed or complete message exceeds native admission".into(),
                );
            }
            let author = raw
                .split(|b| *b == b'\n')
                .find_map(|l| l.strip_prefix(b"author "))
                .filter(|s| s.len() <= 4096)
                .ok_or("Source author metadata is missing or exceeds admission")?;
            let parent = choice.mainline.map(|n| parents[n - 1]);
            steps.push(ApplyStep {
                object: choice.object,
                mainline: choice.mainline,
                parent,
                subject: revision.subject.clone(),
                message: message.into(),
                author: author.into(),
            });
        }
        if !remaining.is_empty() {
            return Err("Source commit batch has unexpected trailing records".into());
        }
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
}
