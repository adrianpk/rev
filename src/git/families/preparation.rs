use super::*;

/// Exact command input is validated before repository authority is captured.
pub(super) struct Preparation {
    pub family: Family,
    pub args: Vec<OsString>,
    pub output: Option<PathBuf>,
}
impl Preparation {
    fn validate(&self, directory: &Path) -> Result<(), String> {
        self.validate_output()?;
        if !directory.is_absolute() || directory.as_os_str().len() > 4096 {
            return Err("Prepared directory requires an absolute path within 4096 bytes".into());
        }
        if self.args.len() > 64
            || self.args.iter().any(|a| a.len() > 4096)
            || self.args.iter().map(|a| a.len()).sum::<usize>() > 65536
        {
            return Err(
                "Prepared input exceeds 64 arguments, 4096 bytes per argument or 64 KiB total"
                    .into(),
            );
        }
        self.family.validate(&self.args)?;
        Ok(())
    }
    fn validate_output(&self) -> Result<(), String> {
        if self.family.output() != self.output.is_some()
            || self.output.as_ref().is_some_and(|p| {
                p.as_os_str().is_empty()
                    || p.as_os_str().len() > 4096
                    || p.as_os_str().as_encoded_bytes().contains(&0)
            })
        {
            return Err("Output families require one explicit output path within 4096 bytes; other families take literal argv only".into());
        }
        Ok(())
    }
}
impl Repositories {
    pub(super) fn prepare_family_file(
        &mut self,
        directory: &Path,
        context: Option<ContextId>,
        input: Preparation,
    ) -> Result<OperationId, String> {
        self.family_idle()?;
        input.validate(directory)?;
        let Preparation {
            family,
            args,
            output,
        } = input;
        let (args, shell) = self.family_arguments(family, args)?;
        let observation = self.family_observation(context, family)?;
        let size = args.iter().map(OsString::capacity).sum::<usize>()
            + output.as_ref().map_or(0, PathBuf::capacity)
            + shell.as_ref().map_or(0, |s| s.program.capacity())
            + 8192;
        let admission = self.budget.reserve(size)?;
        self.family_plan = None;
        self.family_result = None;
        self.start(
            directory.into(),
            context,
            work::Request::Family(Request::Prepare {
                family,
                args,
                observation,
                output,
                shell,
                _admission: admission,
            }),
        )
    }

    fn family_arguments(
        &self,
        family: Family,
        args: Vec<OsString>,
    ) -> Result<(Vec<OsString>, Option<Shell>), String> {
        if family != Family::RunShellCommand {
            return Ok((args, None));
        }
        let command = args[0]
            .to_str()
            .ok_or("Shell command requires UTF-8 input")?;
        let mut argv = crate::shell::arguments(&self.shell_policy, &self.shell_startup, command)?;
        let program = argv.remove(0);
        Ok((
            argv.into_iter().map(OsString::from).collect(),
            Some(Shell {
                program,
                seconds: self
                    .shell_policy
                    .timeout_seconds
                    .min(self.policy.operation_timeout_seconds),
                output_bytes: self
                    .shell_policy
                    .max_output_bytes
                    .min(self.policy.max_output_bytes),
            }),
        ))
    }
    fn family_observation(
        &self,
        context: Option<ContextId>,
        family: Family,
    ) -> Result<Option<Arc<Observation>>, String> {
        let observation = context
            .map(|id| {
                self.context(id)
                    .map(|c| std::sync::Arc::clone(&c.observation))
                    .ok_or("Prepared source context expired")
            })
            .transpose()?;
        if family.repository_action()
            && observation.as_ref().is_none_or(|o| {
                !(family.notes() || family.output()) && o.identity.worktree.is_none()
            })
        {
            return Err(
                "Repository family preparation requires a checked working-tree context".into(),
            );
        }
        Ok(observation)
    }
}
