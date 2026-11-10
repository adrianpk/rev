use super::*;
use crate::git::bisect::Request;
mod execution;
mod observation;
use execution::Change;

impl Runner {
    pub(super) fn bisect(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let result = match request {
            Request::Prepare {
                refs, bad, good, ..
            } => self
                .prepare_bisect(refs, &bad, &good, admission)
                .map(Candidate::BisectPlan),
            Request::Start(plan) => Change {
                plan: &plan,
                control: None,
            }
            .run(self, admission),
            Request::Control { sequence, action } => {
                let plan = sequence
                    .bisect
                    .as_ref()
                    .and_then(|b| b.plan.clone())
                    .ok_or("Bisect original lineage is unavailable")?;
                Change {
                    plan: &plan,
                    control: Some((&sequence, action)),
                }
                .run(self, admission)
            }
        };
        if result.is_err() && self.effects == Effects::Possible {
            self.operation_failure.get_or_insert_with(||"Manual bisect effects remain unverified; refresh actual Git state; no automatic retry or inferred reset".into());
        }
        result
    }
    fn prepare_bisect(
        &mut self,
        refs: Arc<Refs>,
        bad: &str,
        good: &str,
        admission: &mut Lease,
    ) -> Result<BisectPlan, String> {
        self.check_bisect_preparation(&refs, admission)?;
        let bad = self.bisect_boundary(bad, refs.observation.identity.format)?;
        let good = self.bisect_boundary(good, refs.observation.identity.format)?;
        if bad == good || !self.bisect_ancestor(good, bad)? {
            return Err(
                "Bisect requires distinct full bad/good commits with good ancestral to bad".into(),
            );
        }
        self.check_integration_source(&refs, admission)?;
        admission.grow(std::mem::size_of::<BisectPlan>())?;
        Ok(BisectPlan {
            refs,
            bad,
            good,
            _admission: admission.split(std::mem::size_of::<BisectPlan>()),
        })
    }
    fn check_bisect_preparation(
        &mut self,
        refs: &Arc<Refs>,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if self.policy.max_refs < 2 {
            return Err("Bisect boundaries require admission for at least two refs".into());
        }
        let current = self.check_integration_source(refs, admission)?;
        if current.completeness != Completeness::Complete
            || current.head.is_none()
            || !current.head_ref.starts_with(b"refs/heads/")
            || self.read_sequence(&current, admission)?.family.is_some()
        {
            return Err(
                "Manual bisect starts from a checked existing branch without another sequence"
                    .into(),
            );
        }
        Ok(())
    }
    fn bisect_boundary(&mut self, text: &str, format: ObjectFormat) -> Result<ObjectId, String> {
        let bytes = self.command(
            &[
                "--no-replace-objects",
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{text}^{{commit}}"),
            ],
            Phase::Checking,
            false,
        )?;
        ObjectId::parse(bytes.strip_suffix(b"\n").unwrap_or(&bytes), format)
    }
    pub(in crate::git::work) fn bind_bisect(
        &mut self,
        sequence: &mut Sequence,
        plan: Arc<BisectPlan>,
    ) -> Result<(), String> {
        let start = sequence
            .files
            .iter()
            .find(|f| f.name == b"BISECT_START")
            .map(|f| f.bytes.strip_suffix(b"\n").unwrap_or(&f.bytes));
        let state = sequence
            .bisect
            .as_mut()
            .ok_or("Actual bisect has no compatible complete candidate state")?;
        if sequence.family != Some(SequenceFamily::Bisect)
            || start != plan.refs.head_ref.strip_prefix(b"refs/heads/")
            || !bisect_refs_match(&sequence.refs, &plan.refs)
            || !state.good.contains(&plan.good)
            || !self.bisect_ancestor(state.bad, plan.bad)?
        {
            return Err(
                "Actual bisect boundaries/original branch differ from retained exact lineage"
                    .into(),
            );
        }
        sequence._admission.grow(plan.refs.head_ref.len())?;
        sequence.original = plan.refs.head;
        sequence.branch = plan.refs.head_ref.clone();
        state.plan = Some(plan);
        Ok(())
    }
}

fn bisect_refs_match(actual: &Refs, original: &Refs) -> bool {
    actual.entries == original.entries
        && actual.remotes == original.remotes
        && actual.completeness == Completeness::Complete
}
