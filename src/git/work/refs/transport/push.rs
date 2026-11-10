use super::*;

fn values<'a>(config: &'a [u8], key: &'a [u8]) -> impl Iterator<Item = &'a [u8]> {
    config.split(|b| *b == 0).filter_map(move |record| {
        record
            .strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(b"\n"))
    })
}
fn one(config: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let mut items = values(config, key);
    let value = items
        .next()
        .filter(|v| !v.is_empty())
        .ok_or("Push requires explicit configured upstream remote and destination")?;
    if items.next().is_some() {
        return Err("Push tracking configuration is ambiguous".into());
    }
    Ok(value.into())
}
impl Runner {
    fn push_endpoint(&mut self, remote: &[u8]) -> Result<Vec<u8>, String> {
        let (bytes, _) = self.command_os(
            &[
                "remote".into(),
                "get-url".into(),
                "--push".into(),
                "--all".into(),
                "--".into(),
                OsString::from_vec(remote.into()),
            ],
            Phase::Checking,
            false,
            None,
        )?;
        let endpoint = bytes
            .strip_suffix(b"\n")
            .ok_or("Push endpoint response is incomplete")?;
        if endpoint.is_empty()
            || endpoint.len() > 4096
            || endpoint.contains(&0)
            || endpoint.contains(&b'\n')
        {
            return Err("Push requires one bounded effective configured endpoint; multiple push URLs are not admitted".into());
        }
        Ok(endpoint.into())
    }
    pub(in crate::git::work) fn transport_bool(&mut self, key: Vec<u8>) -> Result<bool, String> {
        let bytes = self
            .command_os(
                &[
                    "config".into(),
                    "--bool".into(),
                    "--get".into(),
                    OsString::from_vec(key),
                ],
                Phase::Checking,
                true,
                None,
            )?
            .0;
        match bytes.as_slice() {
            b"true\n" => Ok(true),
            b"false\n" | b"" => Ok(false),
            _ => Err("Push policy boolean is unavailable".into()),
        }
    }
    fn push_policy(
        &mut self,
        config: &[u8],
        source: &[u8],
        remote: &[u8],
        destination: &[u8],
    ) -> Result<Vec<u8>, String> {
        if self.transport_bool(b"push.followtags".to_vec())? {
            return Err("push.followTags conflicts with the reviewed single-branch push; revise configuration explicitly".into());
        }
        if self.transport_bool([b"remote.".as_slice(), remote, b".mirror"].concat())? {
            return Err(
                "Remote mirror push conflicts with the reviewed single-branch request".into(),
            );
        }
        let key = [b"remote.".as_slice(), remote, b".push"].concat();
        let expected = [source, b":", destination].concat();
        let mut specs = values(config, &key);
        if let Some(spec) = specs.next()
            && (spec != expected || specs.next().is_some())
        {
            return Err(
                "Configured push refspecs conflict with one non-forced upstream branch".into(),
            );
        }
        let recurse = values(config, b"push.recursesubmodules")
            .last()
            .unwrap_or(b"no");
        if ![b"no".as_slice(), b"check"].contains(&recurse) {
            return Err(
                "Recursive submodule publication conflicts with the single-branch request".into(),
            );
        }
        Ok(recurse.into())
    }
    pub(in crate::git::work::refs) fn prepare_push(
        &mut self,
        refs: Arc<Refs>,
        source: &[u8],
        selected_remote: Option<&[u8]>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_ref_snapshot(&refs, admission)?;
        let config = self.transport_config(admission)?;
        let upstream = Upstream::read(&config, source)?;
        upstream.check(self, &refs, selected_remote, &config)?;
        upstream.prepare(self, refs, config, admission)
    }
    fn remote_object(&mut self, intent: &PushIntent) -> Result<Option<ObjectId>, String> {
        let result = self.command_os(
            &[
                "ls-remote".into(),
                "--refs".into(),
                "--".into(),
                OsString::from_vec(intent.endpoint.clone()),
                OsString::from_vec(intent.destination.clone()),
            ],
            Phase::Verifying,
            false,
            None,
        );
        // The actual query uses the privately captured PUSH endpoint, which
        // may differ from the fetch URL. Never retain that endpoint in argv.
        self.argv = vec![
            self.policy.program.clone().into(),
            "ls-remote".into(),
            "--refs".into(),
            "<reviewed configured push endpoint>".into(),
            OsString::from_vec(intent.destination.clone()),
        ];
        let bytes = result?.0;
        if bytes.is_empty() {
            return Ok(None);
        }
        let line = bytes
            .strip_suffix(b"\n")
            .ok_or("Remote verification response is incomplete")?;
        let tab = line
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("Remote verification lacks its ref boundary")?;
        if line[tab + 1..] != intent.destination {
            return Err("Remote verification returned a different or ambiguous ref".into());
        }
        Ok(Some(ObjectId::parse(
            &line[..tab],
            intent.refs.observation.identity.format,
        )?))
    }
    pub(in crate::git::work::refs) fn push(
        &mut self,
        intent: &Arc<PushIntent>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let recurse = self.check_push_review(intent, admission)?;
        // Explicit negative expansion flags enforce the already-compatible
        // reviewed policy across a last-moment configuration race. They never
        // admit conflicting configuration by silently overriding it.
        let args = vec![
            "push".into(),
            "--porcelain".into(),
            "--no-force".into(),
            "--no-mirror".into(),
            "--no-follow-tags".into(),
            "--no-tags".into(),
            "--no-prune".into(),
            "--no-delete".into(),
            "--no-all".into(),
            OsString::from_vec([b"--recurse-submodules=".as_slice(), &recurse].concat()),
            "--".into(),
            OsString::from_vec(intent.remote.clone()),
            OsString::from_vec(
                [
                    intent.object.hex().as_bytes(),
                    b":",
                    intent.destination.as_slice(),
                ]
                .concat(),
            ),
        ];
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Push cleanup remains owned; remote publication is unresolved".into());
        }
        let result = self.observe_push(intent, executed.is_ok(), admission);
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some("Remote publication remains unresolved after possible push effects; no automatic retry".into());
        }
        result
    }
    fn check_prepared_push(
        &mut self,
        config: &[u8],
        remote: &[u8],
        endpoint: &[u8],
        admission: &mut Lease,
    ) -> Result<(), String> {
        if self.transport_config(admission)? != config || self.push_endpoint(remote)? != endpoint {
            return Err("Push configuration changed during preparation; prepare again".into());
        }
        Ok(())
    }
    fn check_push_review(
        &mut self,
        intent: &PushIntent,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        self.check_ref_snapshot(&intent.refs, admission)?;
        let config = self.transport_config(admission)?;
        if config != intent.config || self.push_endpoint(&intent.remote)? != intent.endpoint {
            return Err(
                "Push configuration changed while review was open; prepare and review again".into(),
            );
        }
        let recurse =
            self.push_policy(&config, &intent.source, &intent.remote, &intent.destination)?;
        self.check_ref_snapshot(&intent.refs, admission)?;
        if self.transport_config(admission)? != intent.config {
            return Err("Push configuration changed before launch; review again".into());
        }
        Ok(recurse)
    }
    fn observe_push(
        &mut self,
        intent: &PushIntent,
        executed: bool,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let after = self.refresh_ref_result(&intent.refs, admission)?;
        let remote = self.remote_object(intent);
        let stable = self.transport_config(admission)? == intent.config;
        match remote {
            Ok(Some(object)) if object == intent.object && stable && executed => self.effects = Effects::Verified,
            Ok(_) if stable && !executed => {
                self.failure_verified = true;
                self.operation_failure = Some("Push child failed; current remote destination was observed; inspect exit; no automatic retry".into());
            }
            _ => self.operation_failure = Some("Remote publication remains unverified or differs from the reviewed object/configuration; no automatic retry".into()),
        }
        Ok(Candidate::Refs {
            refs: after,
            changed: true,
        })
    }
}

/// Exact configured upstream; no endpoint or destination is inferred.
struct Upstream<'a> {
    source: &'a [u8],
    branch: Vec<u8>,
    remote: Vec<u8>,
    destination: Vec<u8>,
}
impl<'a> Upstream<'a> {
    fn read(config: &[u8], source: &'a [u8]) -> Result<Self, String> {
        let short = source
            .strip_prefix(b"refs/heads/")
            .ok_or("Push source is not a local branch")?;
        let branch = [b"branch.".as_slice(), short, b"."].concat();
        let remote = one(config, &[branch.as_slice(), b"remote"].concat())?;
        let destination = one(config, &[branch.as_slice(), b"merge"].concat())?;
        Ok(Self {
            source,
            branch,
            remote,
            destination,
        })
    }
    fn check(
        &self,
        runner: &mut Runner,
        refs: &Refs,
        selected_remote: Option<&[u8]>,
        config: &[u8],
    ) -> Result<(), String> {
        let remote = &self.remote;
        let destination = &self.destination;
        let branch = &self.branch;
        if remote == b"."
            || !refs.remotes.contains(remote)
            || selected_remote.is_some_and(|r| r != remote)
        {
            return Err(
                "Selected remote contradicts the source branch's configured upstream".into(),
            );
        }
        Runner::configured_transport(config, remote)?;
        if !destination.starts_with(b"refs/heads/") {
            return Err("Push destination must be one full upstream branch ref".into());
        }
        runner.command_os(
            &[
                "check-ref-format".into(),
                OsString::from_vec(destination.clone()),
            ],
            Phase::Checking,
            false,
            None,
        )?;
        for key in [
            [branch.as_slice(), b"pushremote"].concat(),
            b"remote.pushdefault".to_vec(),
        ] {
            if values(config, &key).any(|r| r != remote) {
                return Err("Configured push remote contradicts the configured upstream; explicit revision required".into());
            }
        }
        Ok(())
    }
    fn prepare(
        self,
        runner: &mut Runner,
        refs: Arc<Refs>,
        config: Vec<u8>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let source = self.source;
        let remote = &self.remote;
        let destination = &self.destination;
        // Git supplies its resolved upstream relation. Raw tracking keys and
        // the captured exact ref must agree, including the remote-side ref.
        let relation = runner.command_os(&["for-each-ref".into(), "--format=%(refname)%00%(upstream)%00%(upstream:remotename)%00%(upstream:remoteref)%00".into(), OsString::from_vec(source.into())], Phase::Checking, false, None)?.0;
        let reference = refs.reference(source).ok_or("Push source expired")?;
        self.check_relation(&relation, reference)?;
        let recurse = runner.push_policy(&config, source, remote, destination)?;
        let endpoint = runner.push_endpoint(remote)?;
        runner.check_ref_snapshot(&refs, admission)?;
        runner.check_prepared_push(&config, remote, &endpoint, admission)?;
        let mut intent = PushIntent {
            object: reference.object,
            source: source.into(),
            remote: self.remote,
            destination: self.destination,
            config,
            endpoint,
            recurse,
            refs,
            _admission: admission.split(0),
        };
        admission.grow(intent.bytes())?;
        intent._admission = admission.split(intent.bytes());
        Ok(Candidate::PushIntent(intent))
    }
    fn check_relation(&self, relation: &[u8], reference: &Reference) -> Result<(), String> {
        let source = self.source;
        let remote = &self.remote;
        let destination = &self.destination;
        let expected = [
            source,
            b"\0",
            reference.upstream.as_slice(),
            b"\0",
            remote.as_slice(),
            b"\0",
            destination.as_slice(),
            b"\0\n",
        ]
        .concat();
        if reference.upstream.is_empty() || relation != expected.as_slice() {
            return Err(
                "Push upstream relation is missing or contradictory; no destination is inferred"
                    .into(),
            );
        }
        Ok(())
    }
}
