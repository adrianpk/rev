use super::*;
use crate::git::families::{FamilyProof, Output};
use std::{
    ffi::OsStr,
    fs,
    io::Write,
    os::{
        fd::AsRawFd,
        unix::{ffi::OsStringExt, fs::OpenOptionsExt},
    },
};

impl Runner {
    fn output_remote(
        &mut self,
        endpoint: &OsStr,
        reference: &OsStr,
        object: ObjectId,
    ) -> Result<(), String> {
        if self.subtree_remote(endpoint, reference, object.format())? != Some(object) {
            return Err("Request-pull requires the exact reviewed commit at the explicit remote branch; no publication is implied".into());
        }
        Ok(())
    }
    fn output_bytes(&mut self, args: &[OsString]) -> Result<Vec<u8>, String> {
        let limit = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        let (bytes, limited) = self.command_os(args, Phase::Checking, false, Some(limit))?;
        if limited || bytes.len() > limit {
            return Err("Prepared output exceeds complete detail/scratch admission; no partial file is saved".into());
        }
        Ok(bytes)
    }
    pub(super) fn prepare_output(
        &mut self,
        input: &mut preparation::Preparation,
        destination: &Path,
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Output, String> {
        let family = input.family;
        let args = &mut input.args;
        check_output_source(source)?;
        check_output_destination(destination, &source.refs.observation.identity)?;
        let configuration = self.transport_config(admission)?;
        let format = source.refs.observation.identity.format;
        let remote = self.output_arguments(family, args, format, &configuration)?;
        args.insert(0, "--no-replace-objects".into());
        let bytes = self.admitted_output_bytes(family, args, admission)?;
        let hash = self.output_hash(&bytes, format)?;
        self.check_output_prepared(source, &configuration, admission)?;
        Ok(Output {
            configuration,
            bytes,
            hash,
            remote,
        })
    }
    fn publish_output_file(&mut self, plan: &FamilyPlan, bytes: &[u8]) -> Result<(), String> {
        let publication = OutputDirectory::open(plan)?;
        // Reuse the existing exclusively owned temporary-file lifecycle through
        // the captured directory FD. Parent renames cannot retarget its cleanup.
        let anchored = &publication.anchored;
        self.effects = Effects::Possible;
        let mut temporary = crate::file::temporary::Temporary::new(anchored)
            .map_err(|e| format!("Stage output file: {e}"))?;
        let published = publication.publish(
            &mut temporary,
            plan.destination.file_name().expect("checked output name"),
            bytes,
            self,
        );
        if let Err(e) = published {
            return Err(format!(
                "Publish output file: {}",
                temporary.failed(e, &mut crate::file::Files)
            ));
        }
        drop(temporary);
        if self.family_file(&plan.destination)?.as_deref() != Some(bytes) {
            return Err("Published output bytes differ from the accepted complete output".into());
        }
        Ok(())
    }
    pub(super) fn execute_output(
        &mut self,
        plan: &Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Output(output) = &plan.proof else {
            return Err("Output proof unavailable".into());
        };
        if self.transport_config(admission)? != output.configuration {
            return Err("Output configuration changed during review".into());
        }
        if let Some((endpoint, reference, object)) = &output.remote {
            self.output_remote(endpoint, reference, *object)?;
        }
        let (generated, facts) = self.execute_change_output(&plan.args, None);
        let failed = generated.is_err();
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Output producer cleanup remains owned; no replay or file deletion".into());
        }
        let verified = self.verify_output(plan, output, generated, admission);
        if let Err(e) = verified {
            self.operation_failure = Some(format!(
                "Output result is unverified: {e}; inspect the explicit path; no retry or rollback"
            ));
        }
        let result = self.family_observed_result(std::sync::Arc::clone(plan), None, admission);
        self.restore_execution(facts, failed);
        result
    }
    fn output_arguments(
        &mut self,
        family: Family,
        args: &mut [OsString],
        format: ObjectFormat,
        configuration: &[u8],
    ) -> Result<Option<(OsString, OsString, ObjectId)>, String> {
        let mut remote = None;
        match family {
            Family::FormatPatch => {
                self.output_range(args, format)?;
            }
            Family::SaveDiff => {
                for arg in &mut args[6..8] {
                    *arg = self.family_commit(arg, format)?.hex().into();
                }
            }
            Family::RequestPull => {
                remote = Some(self.output_pull_arguments(args, format, configuration)?);
            }
            _ => return Err("Typed output family unavailable".into()),
        }
        Ok(remote)
    }
    fn output_range(&mut self, args: &mut [OsString], format: ObjectFormat) -> Result<(), String> {
        let range = args[9].as_encoded_bytes();
        let offsets: Vec<_> = range
            .windows(2)
            .enumerate()
            .filter(|(_, w)| *w == b"..")
            .map(|(i, _)| i)
            .collect();
        if offsets.len() != 1 || offsets[0] == 0 || offsets[0] + 2 == range.len() {
            return Err("Format-patch requires one explicit local BASE..END commit range".into());
        }
        let split = offsets[0];
        let base = self.family_commit(&OsString::from_vec(range[..split].to_vec()), format)?;
        let end = self.family_commit(&OsString::from_vec(range[split + 2..].to_vec()), format)?;
        args[9] = format!("{}..{}", base.hex(), end.hex()).into();
        Ok(())
    }
    fn output_pull_arguments(
        &mut self,
        args: &mut [OsString],
        format: ObjectFormat,
        configuration: &[u8],
    ) -> Result<(OsString, OsString, ObjectId), String> {
        let endpoint = args[2].clone();
        check_output_endpoint(&endpoint, configuration)?;
        let end = args[3].as_encoded_bytes();
        let split = end.iter().position(|b| *b == b':');
        let (local, reference) = split.map_or((end, end), |i| (&end[..i], &end[i + 1..]));
        let local = OsString::from_vec(local.to_vec());
        let reference = OsString::from_vec(reference.to_vec());
        if !reference.as_encoded_bytes().starts_with(b"refs/heads/") {
            return Err(
                "Request-pull requires one full remote refs/heads branch, optionally LOCAL:REMOTE"
                    .into(),
            );
        }
        let base = self.family_commit(&args[1], format)?;
        let end = self.family_commit(&local, format)?;
        self.output_remote(&endpoint, &reference, end)?;
        args[1] = base.hex().into();
        args[3] =
            OsString::from_vec([end.hex().as_bytes(), b":", reference.as_encoded_bytes()].concat());
        Ok((endpoint, reference, end))
    }
    fn admitted_output_bytes(
        &mut self,
        family: Family,
        args: &[OsString],
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let bytes = self.output_bytes(args)?;
        admission.grow(bytes.capacity())?;
        if family != Family::SaveDiff && bytes.is_empty() {
            return Err("Selected range produced no complete output".into());
        }
        Ok(bytes)
    }
    fn output_hash(&mut self, bytes: &[u8], format: ObjectFormat) -> Result<ObjectId, String> {
        let input = crate::process::input::stage(Some(bytes))?;
        let hash = self
            .command_argv(
                self.git_invocation(
                    &[
                        "hash-object".into(),
                        "--no-filters".into(),
                        "--stdin".into(),
                    ],
                    input,
                ),
                Phase::Checking,
                false,
                None,
            )?
            .0;
        let hash = ObjectId::parse(
            hash.strip_suffix(b"\n")
                .ok_or("Output hash is incomplete")?,
            format,
        )?;
        Ok(hash)
    }
    fn check_output_prepared(
        &mut self,
        source: &Sequence,
        configuration: &[u8],
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.checked_sequence(source, admission)?;
        if self.transport_config(admission)? != configuration {
            return Err("Output configuration changed during preparation".into());
        }
        Ok(())
    }
    fn verify_output(
        &mut self,
        plan: &FamilyPlan,
        output: &Output,
        generated: Result<Vec<u8>, String>,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let bytes = generated?;
        admission.grow(bytes.capacity())?;
        if bytes != output.bytes {
            return Err(
                "Actual output differs from the reviewed complete bytes; no file published".into(),
            );
        }
        self.check_output_generation(plan, output, admission)?;
        self.publish_output_file(plan, &bytes)?;
        self.effects = Effects::Verified;
        Ok(())
    }
    fn check_output_generation(
        &mut self,
        plan: &FamilyPlan,
        output: &Output,
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.family_current(plan, None, admission)?;
        if self.transport_config(admission)? != output.configuration {
            return Err("Output configuration changed during generation".into());
        }
        if let Some((endpoint, reference, object)) = &output.remote {
            self.output_remote(endpoint, reference, *object)?;
        }
        Ok(())
    }
}

fn check_output_source(source: &Sequence) -> Result<(), String> {
    if source.family.is_some()
        || source.refs.completeness != Completeness::Complete
        || source
            .refs
            .observation
            .status
            .as_ref()
            .is_some_and(|s| s.completeness != Completeness::Complete)
    {
        return Err(
            "Output preparation requires complete saved/ref state and no active sequence".into(),
        );
    }
    Ok(())
}
fn check_output_destination(destination: &Path, identity: &Identity) -> Result<(), String> {
    if destination.starts_with(&identity.git_directory)
        || destination.starts_with(&identity.common_directory)
        || destination.components().any(|p| p.as_os_str() == ".git")
    {
        return Err("Output destination must be outside Git metadata".into());
    }
    Ok(())
}
fn check_output_endpoint(endpoint: &OsStr, configuration: &[u8]) -> Result<(), String> {
    let input = endpoint.as_encoded_bytes();
    if !Path::new(endpoint).is_absolute()
        && !input.windows(3).any(|w| w == b"://")
        && !input.contains(&b':')
    {
        return Err("Request-pull requires an explicit repository URL or absolute path".into());
    }
    for entry in configuration.split(|b| *b == 0) {
        if let Some(split) = entry.iter().position(|b| *b == b'\n') {
            let (key, value) = (&entry[..split], &entry[split + 1..]);
            if key.starts_with(b"url.") && key.ends_with(b".insteadof") && input.starts_with(value)
            {
                return Err("Request-pull endpoint has an active URL rewrite; use an explicit unrevised endpoint".into());
            }
        }
    }
    Ok(())
}

/// The open parent descriptor anchors staging, publication and cleanup together.
struct OutputDirectory<'a> {
    parent: &'a Path,
    directory: fs::File,
    expected: [u64; 8],
    anchored: PathBuf,
}
impl<'a> OutputDirectory<'a> {
    fn open(plan: &'a FamilyPlan) -> Result<Self, String> {
        let expected = plan
            .parent_stamp
            .ok_or("Output parent evidence unavailable")?;
        if Runner::family_destination_stamp(&plan.destination)? != expected {
            return Err("Output destination or parent changed before file publication".into());
        }
        let parent = plan
            .destination
            .parent()
            .ok_or("Output parent unavailable")?;
        let flags = rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::NOFOLLOW;
        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(i32::try_from(flags.bits()).expect("bounded directory flags"))
            .open(parent)
            .map_err(|e| format!("Open output directory: {e}"))?;
        if fingerprint(&directory.metadata().map_err(|e| e.to_string())?) != expected {
            return Err("Output parent changed before descriptor capture".into());
        }
        let anchored = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()));
        Ok(Self {
            parent,
            directory,
            expected,
            anchored,
        })
    }
    fn check_parent(&self, runner: &Runner) -> std::io::Result<()> {
        let actual = fingerprint(&fs::metadata(self.parent)?);
        if actual[..2] != self.expected[..2]
            || actual[3] != self.expected[3]
            || fs::canonicalize(self.parent)? != self.parent
            || runner.cancel.is_cancelled()
            || Instant::now() >= runner.deadline
        {
            return Err(std::io::Error::other(
                "Output parent changed, cancelled or timed out before publication",
            ));
        }
        Ok(())
    }
    fn publish(
        &self,
        temporary: &mut crate::file::temporary::Temporary,
        name: &OsStr,
        bytes: &[u8],
        runner: &Runner,
    ) -> std::io::Result<()> {
        temporary.file.write_all(bytes)?;
        temporary.file.flush()?;
        temporary.file.sync_all()?;
        self.check_parent(runner)?;
        // Hard-link publication is atomic and cannot overwrite an entry
        // that appeared after review. Only our staging name is cleaned up.
        fs::hard_link(&temporary.path, self.anchored.join(name))?;
        self.directory.sync_all()
    }
}
