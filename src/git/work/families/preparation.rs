use super::*;
use crate::git::families::{FamilyProof, Shell, Submodule};

/// One validated command retains its captured authority while argv is resolved.
pub(super) struct Preparation {
    pub family: Family,
    pub args: Vec<OsString>,
    pub observation: Option<Arc<Observation>>,
    pub output: Option<PathBuf>,
    pub shell: Option<Shell>,
}

impl Runner {
    pub(super) fn prepare_family(
        &mut self,
        mut input: Preparation,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let directory_stamp = self.prepare_family_directory(&input)?;
        let destination = input.destination(self)?;
        let parent_stamp = input.parent_stamp(&destination)?;
        input.resolve_destination(&destination);
        let source = input.source(self, admission)?;
        let evidence = self.family_source_evidence(source.as_ref(), admission)?;
        let proof = input.proof(self, &destination, source.as_ref(), admission)?;
        let size = std::mem::size_of::<FamilyPlan>()
            + input.args.capacity() * std::mem::size_of::<OsString>()
            + input.args.iter().map(OsString::capacity).sum::<usize>()
            + evidence.iter().map(Vec::capacity).sum::<usize>()
            + self.directory.capacity()
            + destination.capacity()
            + input
                .shell
                .as_ref()
                .map_or(self.policy.program.capacity(), |s| s.program.capacity())
            + proof.bytes()
            + 4096;
        admission.grow(size)?;
        Ok(Candidate::FamilyPlan(FamilyPlan {
            family: input.family,
            directory: self.directory.clone(),
            program: input
                .shell
                .map_or_else(|| self.policy.program.clone(), |s| s.program),
            args: input.args,
            destination,
            observation: input.observation,
            source,
            evidence,
            directory_stamp,
            parent_stamp,
            proof,
            _admission: admission.split(size),
        }))
    }

    fn family_source_evidence(
        &mut self,
        source: Option<&Arc<Sequence>>,
        admission: &mut Lease,
    ) -> Result<[Vec<u8>; 3], String> {
        if let Some(source) = source.filter(|s| s.refs.observation.status.is_some()) {
            self.stash_evidence(source, admission)
        } else {
            Ok([Vec::new(), Vec::new(), Vec::new()])
        }
    }

    fn prepare_family_directory(&mut self, input: &Preparation) -> Result<[u64; 8], String> {
        if input.family != Family::RunShellCommand {
            input.family.validate(&input.args)?;
        }
        if (input.family == Family::RunShellCommand) != input.shell.is_some() {
            return Err("Explicit shell preparation is unavailable".into());
        }
        self.directory = std::fs::canonicalize(&self.directory)
            .map_err(|e| format!("Resolve prepared command directory: {e}"))?;
        let directory_stamp =
            fingerprint(&std::fs::metadata(&self.directory).map_err(|e| e.to_string())?);
        if !self.directory.is_dir() {
            return Err("Prepared command requires a captured directory".into());
        }
        if !input.family.direct() {
            let commands = self.command(&["--list-cmds=main,others"], Phase::Checking, false)?;
            if !commands
                .split(|b| *b == b'\n')
                .any(|name| name == input.args[0].as_encoded_bytes())
            {
                return Err("Installed Git does not provide the selected command family".into());
            }
        }
        Ok(directory_stamp)
    }
}

impl Preparation {
    fn uses_source_directory(&self) -> bool {
        self.family.notes() || self.family.sparse() || self.family.patch() || self.family.direct()
    }
    fn destination(&self, runner: &Runner) -> Result<PathBuf, String> {
        if let Some(output) = &self.output {
            runner.family_destination(output.as_os_str())
        } else if self.uses_source_directory() {
            Ok(runner.directory.clone())
        } else {
            self.repository_destination(runner)
        }
    }
    fn repository_destination(&self, runner: &Runner) -> Result<PathBuf, String> {
        if self.family.subtree() {
            runner.family_relative_destination(&self.args[3])
        } else if self.family.submodule() {
            self.submodule_destination(runner)
        } else {
            runner.family_destination(self.args.last().ok_or("Destination is required")?)
        }
    }
    fn submodule_destination(&self, runner: &Runner) -> Result<PathBuf, String> {
        if self.family == Family::SubmoduleList {
            Ok(runner.directory.clone())
        } else {
            runner
                .family_relative_destination(self.args.last().ok_or("Submodule path is required")?)
        }
    }
    fn parent_stamp(&self, destination: &Path) -> Result<Option<[u64; 8]>, String> {
        if (!self.family.repository_action() && !self.family.direct())
            || self.family.output()
            || matches!(self.family, Family::SubmoduleAdd | Family::SubtreeAdd)
        {
            Ok(Some(Runner::family_destination_stamp(destination)?))
        } else {
            Ok(None)
        }
    }
    fn resolve_destination(&mut self, destination: &Path) {
        if !self.family.repository_action() && !self.family.direct() {
            *self.args.last_mut().expect("checked destination") = destination.as_os_str().into();
        }
    }
    fn source(
        &self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<Option<Arc<Sequence>>, String> {
        self.observation
            .as_ref()
            .map(|o| {
                let refs = runner.integration_refs(o, admission)?;
                let expected = refs
                    .observation
                    .identity
                    .worktree
                    .as_ref()
                    .unwrap_or(&refs.observation.identity.git_directory);
                if expected != &runner.directory {
                    return Err(
                        "Prepared directory differs from the captured repository root".into(),
                    );
                }
                runner.read_sequence(&refs, admission).map(Arc::new)
            })
            .transpose()
    }
    fn submodule_proof(
        &mut self,
        runner: &mut Runner,
        destination: &Path,
        source: Option<&Arc<Sequence>>,
        admission: &mut Lease,
    ) -> Result<Option<Submodule>, String> {
        if !self.family.submodule() {
            return Ok(None);
        }
        let source = source.ok_or("Submodule source context is unavailable")?;
        if source.family.is_some()
            || source
                .refs
                .observation
                .status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete)
        {
            return Err(
                "Submodule preparation requires complete status and no active sequence".into(),
            );
        }
        let state = runner.submodule_state(self.family, destination, source, admission)?;
        self.args.insert(0, "--literal-pathspecs".into());
        Ok(Some(state))
    }
    fn proof(
        &mut self,
        runner: &mut Runner,
        destination: &Path,
        source: Option<&Arc<Sequence>>,
        admission: &mut Lease,
    ) -> Result<FamilyProof, String> {
        let submodule = self.submodule_proof(runner, destination, source, admission)?;
        if self.family.direct() {
            Ok(FamilyProof::Direct(runner.prepare_direct(
                source,
                self.shell.as_ref(),
                &self.args,
                admission,
            )?))
        } else if let Some(state) = submodule {
            Ok(FamilyProof::Submodule(state))
        } else if self.family.subtree() {
            Ok(FamilyProof::Subtree(runner.prepare_subtree(
                self,
                destination,
                source.ok_or("Subtree source context is unavailable")?,
                admission,
            )?))
        } else {
            self.file_proof(runner, destination, source, admission)
        }
    }
    fn file_proof(
        &mut self,
        runner: &mut Runner,
        destination: &Path,
        source: Option<&Arc<Sequence>>,
        admission: &mut Lease,
    ) -> Result<FamilyProof, String> {
        if self.family.output() {
            Ok(FamilyProof::Output(runner.prepare_output(
                self,
                destination,
                source.ok_or("Output source context unavailable")?,
                admission,
            )?))
        } else if self.family.patch() {
            Ok(FamilyProof::Patch(runner.prepare_patch(
                self.family,
                &mut self.args,
                source.ok_or("Patch source context unavailable")?,
                admission,
            )?))
        } else {
            self.notes_or_sparse_proof(runner, source, admission)
        }
    }
    fn notes_or_sparse_proof(
        &mut self,
        runner: &mut Runner,
        source: Option<&Arc<Sequence>>,
        admission: &mut Lease,
    ) -> Result<FamilyProof, String> {
        if self.family.notes() {
            Ok(FamilyProof::Notes(runner.prepare_notes(
                self.family,
                &mut self.args,
                source.ok_or("Notes source context unavailable")?,
                admission,
            )?))
        } else if self.family.sparse() {
            Ok(FamilyProof::Sparse(runner.prepare_sparse(
                self.family,
                &self.args,
                source.ok_or("Sparse source context unavailable")?,
                admission,
            )?))
        } else {
            Ok(FamilyProof::Creation)
        }
    }
}
