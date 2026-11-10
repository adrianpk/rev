use super::*;
mod branch;
mod observation;
mod tag;
mod transport;
use crate::git::{BranchAction, RefKind, Reference, Refs, refs::Request};

impl Runner {
    fn check_ref_snapshot(&mut self, refs: &Refs, admission: &mut Lease) -> Result<(), String> {
        self.check_ref_origin(&refs.observation, admission)?;
        let current = self.read_refs(std::sync::Arc::clone(&refs.observation), admission)?;
        if !refs.same(&current) {
            return Err("Refs changed while request parameters or confirmation were open".into());
        }
        Ok(())
    }
    fn refresh_ref_result(&mut self, refs: &Refs, admission: &mut Lease) -> Result<Refs, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != refs.observation.identity {
            return Err("Repository identity changed after local ref action".into());
        }
        let id = self.progress.lock().expect("Git progress owner").operation;
        let mut observation = Observation {
            context: refs.observation.context,
            generation: id.0,
            identity,
            status,
            admission: admission.split(0),
        };
        observation.admission = admission.split(observation.bytes());
        self.read_refs(Arc::new(observation), admission)
    }

    pub(super) fn refs(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        match request {
            Request::PreparePush {
                refs,
                source,
                remote,
                ..
            } => self.prepare_push(refs, &source, remote.as_deref(), admission),
            Request::Push(intent) => self.push(&intent, admission),
            Request::Fetch { refs, remote, .. } => self.fetch(&refs, &remote, admission),
            Request::PrepareTag {
                refs,
                name,
                kind,
                target,
                ..
            } => self.prepare_tag(refs, name, kind, target, admission),
            Request::CreateTag { intent, text, .. } => self.create_tag(&intent, &text, admission),
            Request::DeleteTag { refs, name, .. } => self.delete_tag(&refs, &name, admission),
            Request::Observe(observation) => self.observe_refs(&observation, admission),
            Request::Branch { refs, action, .. } => self.branch(&refs, &action, admission),
        }
    }
    fn observe_refs(
        &mut self,
        observation: &Observation,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != observation.identity {
            return Err("Refs repository identity changed".into());
        }
        let id = self.progress.lock().expect("Git progress owner").operation;
        let mut current = Observation {
            context: observation.context,
            generation: id.0,
            identity,
            status,
            admission: admission.split(0),
        };
        current.admission = admission.split(current.bytes());
        let refs = self.read_refs(Arc::new(current), admission)?;
        Ok(Candidate::Refs {
            refs,
            changed: true,
        })
    }
    fn check_ref_origin(
        &mut self,
        expected: &Observation,
        admission: &mut Lease,
    ) -> Result<Option<Status>, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != expected.identity
            || status.as_ref().map(|s| &s.head) != expected.status.as_ref().map(|s| &s.head)
        {
            return Err("Refs repository or HEAD changed; refresh Status and Refs".into());
        }
        Ok(status)
    }
    pub(super) fn ref_head(
        &mut self,
        format: ObjectFormat,
    ) -> Result<(Vec<u8>, Option<ObjectId>), String> {
        let reference =
            self.command(&["symbolic-ref", "--quiet", "HEAD"], Phase::Verifying, true)?;
        let reference = reference.strip_suffix(b"\n").unwrap_or(&reference);
        if reference.len() > 4096 {
            return Err("HEAD ref exceeds metadata bound".into());
        }
        let object = self.command(
            &["rev-parse", "--verify", "--quiet", "HEAD"],
            Phase::Verifying,
            true,
        )?;
        let object = object.strip_suffix(b"\n").unwrap_or(&object);
        Ok((
            reference.to_vec(),
            if object.is_empty() {
                None
            } else {
                Some(ObjectId::parse(object, format)?)
            },
        ))
    }
    fn branch_name(&mut self, name: &str) -> Result<(), String> {
        let bytes = self.command(
            &["check-ref-format", "--branch", name],
            Phase::Checking,
            false,
        )?;
        if bytes.strip_suffix(b"\n") != Some(name.as_bytes()) {
            return Err("Branch name resolves to a different identity".into());
        }
        Ok(())
    }
}
