use super::*;
use crate::git::families::{Direct, FamilyProof, Shell};

impl Runner {
    pub(super) fn prepare_direct(
        &mut self,
        source: Option<&Arc<Sequence>>,
        shell: Option<&Shell>,
        args: &[OsString],
        admission: &mut Lease,
    ) -> Result<Direct, String> {
        let program = shell.map_or(self.policy.program.as_str(), |s| &s.program);
        let argv: Vec<_> = std::iter::once(OsString::from(program))
            .chain(
                shell
                    .is_none()
                    .then(|| OsString::from("--no-optional-locks")),
            )
            .chain(args.iter().cloned())
            .collect();
        if argv.len() > 64
            || program.is_empty()
            || argv
                .iter()
                .any(|a| a.len() > 4096 || a.as_encoded_bytes().contains(&0))
            || argv.iter().map(|a| a.len()).sum::<usize>() > 65536
        {
            return Err(
                "Direct executable/argv exceeds 64 tokens, 4096 bytes per token or 64 KiB total"
                    .into(),
            );
        }
        // Preparation observes only the captured source, never the arbitrary
        // command. A direct route can operate an existing Git-owned sequence.
        let configuration = source
            .map(|_| self.transport_config(admission))
            .transpose()?;
        if let Some(source) = source {
            self.checked_sequence(source, admission)?;
        }
        Ok(Direct {
            configuration,
            seconds: shell.map_or(self.policy.operation_timeout_seconds, |s| s.seconds),
            output_bytes: shell.map_or(self.policy.max_output_bytes, |s| s.output_bytes),
        })
    }
    pub(super) fn execute_direct(
        &mut self,
        plan: Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Direct(state) = &plan.proof else {
            return Err("Direct command evidence unavailable".into());
        };
        if let Some(configuration) = &state.configuration
            && self.transport_config(admission)? != *configuration
        {
            return Err("Direct source configuration changed during review".into());
        }
        let previous_limit = self.policy.max_output_bytes;
        self.policy.max_output_bytes = previous_limit.min(state.output_bytes);
        // Actual arbitrary output is inspectable through the existing escaped,
        // credential-redacting process presentation. Configuration probes stay
        // private and can never replace the accepted child's retained facts.
        self.private_output = false;
        let (executed, facts) = if plan.family == Family::RunShellCommand {
            let argv = std::iter::once(OsString::from(&plan.program))
                .chain(plan.args.iter().cloned())
                .collect();
            self.execute_change_argv(argv, None)
        } else {
            self.execute_change_output(&plan.args, None)
        };
        self.policy.max_output_bytes = previous_limit;
        let failed = executed.is_err();
        self.operation_failure = Some(format!(
            "Direct command exit {:?}; Git postconditions remain unknown; no automatic retry or rollback",
            facts.code()
        ));
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Direct command cleanup remains owned; possible effects require later inspection"
                    .into(),
            );
        }
        self.private_output = true;
        let result = if plan.observation.is_some() {
            self.family_observed_result(plan.clone(), None, admission)
        } else {
            Err("No previously checked repository observation was selected".into())
        };
        let result = match result {
            Ok(result) => Ok(result),
            Err(e) => (|| {
                self.operation_failure
                    .as_mut()
                    .expect("unknown direct outcome")
                    .push_str(&format!("; compatible source refresh unavailable: {e}"));
                let size = std::mem::size_of::<FamilyResult>() + 4096;
                admission.grow(size)?;
                Ok(Candidate::FamilyResult(FamilyResult {
                    plan,
                    created: None,
                    object: None,
                    source: None,
                    _admission: admission.split(size),
                }))
            })(),
        };
        self.restore_execution(facts, failed);
        self.private_output = false;
        result
    }
}
