use super::*;

impl Runner {
    pub(super) fn branch(
        &mut self,
        refs: &Arc<Refs>,
        action: &BranchAction,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let status = self.check_ref_origin(&refs.observation, admission)?;
        let before = self.read_refs(std::sync::Arc::clone(&refs.observation), admission)?;
        if !refs.same(&before) {
            return Err("Refs changed while confirmation was open; refresh required".into());
        }
        let mut change = BranchChange::new(refs, action, before);
        change.run(self, status.as_ref(), admission)
    }
}

/// One exact branch action retains selection and its admitted before snapshot.
struct BranchChange<'a> {
    refs: &'a Arc<Refs>,
    action: &'a BranchAction,
    selected: Option<&'a Reference>,
    short: Option<OsString>,
    before: Refs,
}
impl<'a> BranchChange<'a> {
    fn new(refs: &'a Arc<Refs>, action: &'a BranchAction, before: Refs) -> Self {
        let selected = match &action {
            BranchAction::Create { .. } => None,
            BranchAction::Switch { name }
            | BranchAction::Rename { name, .. }
            | BranchAction::Delete { name } => refs.reference(name),
        };
        let short = selected.map(|r| OsString::from_vec(r.name[b"refs/heads/".len()..].to_vec()));
        Self {
            refs,
            action,
            selected,
            short,
            before,
        }
    }
    fn run(
        &mut self,
        runner: &mut Runner,
        status: Option<&Status>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let args = self.arguments(runner, status)?;
        self.check_launch(runner, admission)?;
        let (executed, facts) = runner.execute_change(&args, None);
        if runner.retained.is_some() {
            runner.restore_execution(facts, true);
            return Err("Branch cleanup remains owned; effects need inspection".into());
        }
        let result = self.observe(runner, &executed, admission);
        runner.restore_execution(facts, executed.is_err());
        if result.is_err() {
            runner.operation_failure =
                Some("Branch effects remain unverified; refresh required".into());
        }
        result
    }
    fn arguments(
        &mut self,
        runner: &mut Runner,
        status: Option<&Status>,
    ) -> Result<Vec<OsString>, String> {
        let action = self.action;
        match action {
            BranchAction::Create { name, target } => self.create_arguments(runner, name, *target),
            BranchAction::Switch { .. } => self.switch_arguments(status),
            BranchAction::Rename { new_name, .. } => self.rename_arguments(runner, new_name),
            BranchAction::Delete { name } => self.delete_arguments(runner, name),
        }
    }
    fn create_arguments(
        &self,
        runner: &mut Runner,
        name: &str,
        target: ObjectId,
    ) -> Result<Vec<OsString>, String> {
        let refs = self.refs;
        let mut args = Vec::new();

        runner.branch_name(name)?;
        if refs
            .reference(format!("refs/heads/{name}").as_bytes())
            .is_some()
        {
            return Err("Branch name already exists".into());
        }
        let actual = runner.command(
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{}^{{commit}}", target.hex()),
            ],
            Phase::Checking,
            false,
        )?;
        if actual.strip_suffix(b"\n") != Some(target.hex().as_bytes()) {
            return Err("Branch target is not the exact admitted commit".into());
        }
        args.extend([
            "branch".into(),
            "--no-track".into(),
            "--".into(),
            name.into(),
            target.hex().into(),
        ]);
        Ok(args)
    }
    fn switch_arguments(&mut self, status: Option<&Status>) -> Result<Vec<OsString>, String> {
        let mut args = Vec::new();

        let status = status.ok_or("Switch requires a working tree")?;
        if status.completeness != Completeness::Complete || !status.paths.is_empty() {
            return Err("Branch switch requires a complete clean index and working tree".into());
        }
        args.extend([
            "switch".into(),
            "--no-guess".into(),
            "--".into(),
            self.short.take().ok_or("Missing branch")?,
        ]);
        Ok(args)
    }
    fn rename_arguments(
        &mut self,
        runner: &mut Runner,
        new_name: &str,
    ) -> Result<Vec<OsString>, String> {
        let refs = self.refs;
        let mut args = Vec::new();

        runner.branch_name(new_name)?;
        if refs
            .reference(format!("refs/heads/{new_name}").as_bytes())
            .is_some()
        {
            return Err("Branch name already exists".into());
        }
        args.extend([
            "branch".into(),
            "--move".into(),
            "--".into(),
            self.short.take().ok_or("Missing branch")?,
            new_name.into(),
        ]);
        Ok(args)
    }
    fn delete_arguments(
        &mut self,
        runner: &mut Runner,
        name: &[u8],
    ) -> Result<Vec<OsString>, String> {
        let refs = self.refs;
        let mut args = Vec::new();

        if refs.head_ref == name {
            return Err("Cannot delete the current branch".into());
        }
        let object = self.selected.ok_or("Missing branch")?.object.hex();
        let head = refs
            .head
            .ok_or("Branch deletion requires a checked HEAD")?
            .hex();
        runner.command(
            &["merge-base", "--is-ancestor", &object, &head],
            Phase::Checking,
            false,
        )?;
        args.extend([
            "branch".into(),
            "--delete".into(),
            "--".into(),
            self.short.take().ok_or("Missing branch")?,
        ]);
        Ok(args)
    }
    fn check_launch(&self, runner: &mut Runner, admission: &mut Lease) -> Result<(), String> {
        let refs = self.refs;
        let action = self.action;
        let status = runner.check_ref_origin(&refs.observation, admission)?;
        if matches!(action, BranchAction::Switch { .. })
            && status
                .as_ref()
                .is_none_or(|s| s.completeness != Completeness::Complete || !s.paths.is_empty())
        {
            return Err("Branch switch disk state changed before launch".into());
        }
        let checked = runner.read_refs(std::sync::Arc::clone(&refs.observation), admission)?;
        if !refs.same(&checked) {
            return Err("Refs changed before branch execution".into());
        }
        Ok(())
    }
    fn observe(
        &self,
        runner: &mut Runner,
        executed: &Result<(), String>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let refs = self.refs;
        let before = &self.before;
        let (identity, status) = runner.observe(admission)?;
        let progress = *runner.progress.lock().expect("Git progress owner");
        let mut observation = Observation {
            context: refs.observation.context,
            generation: progress.operation.0,
            identity,
            status,
            admission: admission.split(0),
        };
        observation.admission = admission.split(observation.bytes());
        let observation = Arc::new(observation);
        let after = runner.read_refs(observation, admission)?;
        let valid =
            after.observation.identity == refs.observation.identity && self.postcondition(&after);
        if executed.is_ok() && valid {
            runner.effects = Effects::Verified;
        } else {
            runner.failure_verified = executed.is_err() && after.same(before);
            runner.operation_failure=Some("Branch child failed or exact postcondition differs; inspect actual diagnostics and refs".into());
        }
        Ok(Candidate::Refs {
            refs: after,
            changed: true,
        })
    }
    fn postcondition(&self, after: &Refs) -> bool {
        match self.action {
            BranchAction::Create { name, target } => self.created(after, name, *target),
            BranchAction::Switch { name } => self.switched(after, name),
            BranchAction::Rename { name, new_name } => self.renamed(after, name, new_name),
            BranchAction::Delete { name } => self.deleted(after, name),
        }
    }
    fn created(&self, after: &Refs, name: &str, target: ObjectId) -> bool {
        let refs = self.refs;

        after
            .reference(format!("refs/heads/{name}").as_bytes())
            .is_some_and(|r| r.object == target)
            && after.head_ref == refs.head_ref
            && after.head == refs.head
    }
    fn switched(&self, after: &Refs, name: &[u8]) -> bool {
        let selected = self.selected;

        after.head_ref == name && after.head == selected.map(|r| r.object)
    }
    fn renamed(&self, after: &Refs, name: &[u8], new_name: &str) -> bool {
        let refs = self.refs;
        let selected = self.selected;

        after.reference(name).is_none()
            && after
                .reference(format!("refs/heads/{new_name}").as_bytes())
                .is_some_and(|r| Some(r.object) == selected.map(|r| r.object))
            && after.head == refs.head
            && after.head_ref
                == if refs.head_ref == name {
                    format!("refs/heads/{new_name}").into_bytes()
                } else {
                    refs.head_ref.clone()
                }
    }
    fn deleted(&self, after: &Refs, name: &[u8]) -> bool {
        let refs = self.refs;

        after.reference(name).is_none()
            && after.head == refs.head
            && after.head_ref == refs.head_ref
    }
}
