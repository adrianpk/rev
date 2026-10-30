use super::*;
impl Work {
    pub(in crate::session) fn start_preview(
        &mut self,
        work: crate::org::preview::Work,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(work.stamp(), move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Preview(work.run(&bounded))
            })
            .map_err(|e| e.to_string())?;
        self.preview_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_preview(&self) {
        if self.preview_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_preview(&mut self) -> bool {
        if !self
            .preview_request
            .as_ref()
            .is_some_and(|r| self.org_source_current(r))
        {
            self.org.cancel_preview();
        }
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.preview_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Preview(result))) => self.finish_preview(result),
            Err(e) => self.finish_preview(Err(e.to_string())),
            _ => unreachable!("preview owns job"),
        }
        true
    }
}
