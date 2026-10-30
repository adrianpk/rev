use super::*;
impl Work {
    pub(in crate::session) fn start_feed(
        &mut self,
        stamp: Stamp,
        work: crate::org::feed::Work,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| Observation::Feed(work.run(cancel)))
            .map_err(|e| e.to_string())?;
        self.feed_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_feed(&self) {
        if self.feed_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_feed(&mut self) -> bool {
        if !self.feed_current() {
            self.org.cancel_feed();
        }
        let Some(result) = self.org.job.poll_effect() else {
            return false;
        };
        self.org.feed_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Feed(result))) => self.finish_feed(result),
            Err(e) => self.finish_feed(Err(e.to_string())),
            _ => unreachable!("feed owns job"),
        }
        true
    }
}
