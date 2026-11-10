use super::*;

impl Runner {
    pub(super) fn execute_creation(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let (executed, facts) = self.execute_change(&plan.args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Prepared Git cleanup remains owned; no destination deletion or retry was attempted".into());
        }
        let result = executed.and_then(|()| self.verify_creation(plan, admission));
        if let Err(e) = &result {
            self.operation_failure = Some(format!(
                "Prepared command failed or effects remain unverified: {e}; inspect the exact destination; no automatic cleanup or retry"
            ));
        }
        self.restore_execution(facts, result.is_err());
        result
    }
    fn verify_creation(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.directory = plan.destination.clone();
        let (created, status) = self.observe(admission)?;
        check_created_identity(&plan, &created, status.as_ref())?;
        if plan.family != Family::Init {
            self.check_clone_result(&plan, created.format)?;
        }
        self.directory = plan.directory.clone();
        let source = self.creation_source(&plan, admission)?;
        let size = std::mem::size_of::<FamilyResult>() + created.bytes() + 4096;
        admission.grow(size)?;
        self.effects = Effects::Verified;
        Ok(Candidate::FamilyResult(FamilyResult {
            plan,
            created: Some(created),
            object: None,
            source,
            _admission: admission.split(size),
        }))
    }

    fn check_clone_result(
        &mut self,
        plan: &FamilyPlan,
        format: ObjectFormat,
    ) -> Result<(), String> {
        let head = self.command(
            &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"],
            Phase::Verifying,
            true,
        )?;
        // Empty remotes legitimately have no HEAD commit. Existing
        // objects still require a complete format-checked identity.
        if !head.is_empty() {
            ObjectId::parse(
                head.strip_suffix(b"\n")
                    .ok_or("Clone HEAD has no newline")?,
                format,
            )?;
        }
        self.check_clone_options(plan)
    }

    fn check_clone_options(&mut self, plan: &FamilyPlan) -> Result<(), String> {
        if plan.family == Family::CloneShallow
            && self.command(
                &["rev-parse", "--is-shallow-repository"],
                Phase::Verifying,
                false,
            )? != b"true\n"
        {
            return Err("Clone did not create the requested shallow repository".into());
        }
        if plan.family == Family::CloneMirror
            && self.command(
                &["config", "--bool", "remote.origin.mirror"],
                Phase::Verifying,
                false,
            )? != b"true\n"
        {
            return Err("Clone did not retain mirror configuration".into());
        }
        Ok(())
    }

    fn creation_source(
        &mut self,
        plan: &FamilyPlan,
        admission: &mut Lease,
    ) -> Result<Option<Observation>, String> {
        let source = if let Some(expected) = &plan.observation {
            let (identity, status) = self.observe(admission)?;
            if identity != expected.identity {
                return Err("Source repository identity changed after the command".into());
            }
            let mut source = Observation {
                context: expected.context,
                generation: self.progress.lock().expect("Git progress").operation.0,
                identity,
                status,
                admission: admission.split(0),
            };
            let size = source.bytes();
            admission.grow(size)?;
            source.admission = admission.split(size);
            Some(source)
        } else {
            None
        };
        Ok(source)
    }
}

fn check_created_identity(
    plan: &FamilyPlan,
    created: &Identity,
    status: Option<&Status>,
) -> Result<(), String> {
    let bare = matches!(plan.family, Family::CloneBare | Family::CloneMirror);
    if created.worktree.is_none() != bare
        || (!bare && created.worktree.as_ref() != Some(&plan.destination))
    {
        return Err(
            "Created repository identity does not match the reviewed destination and kind".into(),
        );
    }
    if plan.family == Family::Init && !status.is_some_and(|s| matches!(s.head, Head::Unborn { .. }))
    {
        return Err("Initialized destination is not a verified unborn repository".into());
    }
    Ok(())
}
