use super::*;
impl Work {
    pub(in crate::session) fn start_execution(
        &mut self,
        stamp: Stamp,
        work: crate::org::execution::Work,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                Observation::Execution(work.run(cancel))
            })
            .map_err(|e| e.to_string())?;
        self.execution_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_execution(&self) {
        if self.execution_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_execution(&mut self) -> bool {
        if !self.execution_current() {
            self.org.cancel_execution();
        }
        let Some(result) = self.org.job.poll_effect() else {
            return false;
        };
        self.org.execution_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Execution(out))) => self.finish_execution(Ok(out)),
            Err(e) => self.finish_execution(Err(e.to_string())),
            _ => unreachable!("execution owns job"),
        }
        true
    }
}
