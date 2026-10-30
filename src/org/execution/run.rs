use super::*;
use crate::{
    config::{ProgramStep, SourceDelivery},
    org::io::process::Program,
};
use std::time::{Duration, Instant};
impl Work {
    pub fn run(self, cancel: &Cancellation) -> Outcome {
        let cancel = cancel.bounded(Instant::now() + Duration::from_secs(self.seconds as u64));
        let mut out = Outcome {
            definition: self.definition.clone(),
            program: self.profile.name.clone(),
            result: self.profile.result,
            state: State::Succeeded,
            status: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            detail: String::new(),
        };
        if let Err(e) = self.perform(&cancel, &mut out) {
            out.state = failed(&cancel, &e);
            out.detail = e;
        }
        out
    }
    fn perform(&self, cancel: &Cancellation, out: &mut Outcome) -> Result<(), String> {
        let body = self.body(cancel)?;
        let mut scratch = scratch::Scratch::new()?;
        let result = self.execute(&scratch, &body, cancel, out);
        let cleanup = scratch.clean();
        cleanup?;
        result
    }
    fn body(&self, cancel: &Cancellation) -> Result<String, String> {
        check(cancel)?;
        let text = if headers::expand(&self.definition)? {
            let blocks = self.scan.clone().all(cancel)?;
            let index = blocks
                .iter()
                .position(|b| b.body == self.definition.body)
                .ok_or("Selected execution block is excluded from source")?;
            crate::org::tangle::expand::Expansion::new(&blocks, cancel)?.execute(index)?
        } else {
            self.definition.draft.clone()
        };
        if text.len() > self.expansion_bytes {
            return Err("Source expansion byte limit reached".into());
        }
        Ok(text)
    }
    fn execute(
        &self,
        scratch: &scratch::Scratch,
        body: &str,
        cancel: &Cancellation,
        out: &mut Outcome,
    ) -> Result<(), String> {
        scratch.input(body, cancel)?;
        for step in &self.profile.steps {
            self.step(step, scratch, (None, cancel), out)?;
        }
        let input = if self.profile.delivery == SourceDelivery::Stdin {
            Some(scratch.stdin()?)
        } else {
            None
        };
        let step = ProgramStep {
            executable: self.profile.executable.clone(),
            argv: self.profile.argv.clone(),
        };
        self.step(&step, scratch, (input, cancel), out)
    }
    fn step(
        &self,
        step: &ProgramStep,
        scratch: &scratch::Scratch,
        io: (Option<std::fs::File>, &Cancellation),
        out: &mut Outcome,
    ) -> Result<(), String> {
        let (input, cancel) = io;
        let remaining = self
            .output_bytes
            .saturating_sub(out.stdout.len() + out.stderr.len());
        if remaining == 0 {
            return Err("Source combined step output limit reached".into());
        }
        let result = self.program(step, scratch, remaining)?.run_input(
            input,
            Some(scratch.root()),
            cancel,
        )?;
        out.status = result.status.code();
        out.stdout.extend(result.stdout);
        out.stderr.extend(result.stderr);
        if !result.status.success() {
            return Err(format!("Source program failed ({})", result.status));
        }
        Ok(())
    }
    fn program(
        &self,
        step: &ProgramStep,
        scratch: &scratch::Scratch,
        remaining: usize,
    ) -> Result<Program, String> {
        let mut argv = vec![scratch.substitute(&step.executable)?];
        for token in &step.argv {
            argv.push(scratch.substitute(token)?);
        }
        Ok(Program {
            argv,
            directory: self
                .directory
                .clone()
                .unwrap_or_else(|| scratch.root().into()),
            seconds: self.seconds,
            output_bytes: remaining,
        })
    }
}
fn failed(cancel: &Cancellation, detail: &str) -> State {
    if cancel.expired() || detail.contains("timed out") {
        State::TimedOut
    } else if cancel.is_cancelled() {
        State::Cancelled
    } else {
        State::Failed
    }
}
