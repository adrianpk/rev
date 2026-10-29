use super::*;
use crate::org::publish::JobWork;
impl Work {
    pub(in crate::session) fn start_publishing(
        &mut self,
        stamp: Stamp,
        work: JobWork,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let cancel = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Publish(work.run(&cancel))
            })
            .map_err(|e| e.to_string())?;
        self.publish_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_publishing(&self) {
        if self.publish_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_publishing(&mut self) -> bool {
        if !self.publishing_current() {
            self.org.job.cancel();
        }
        let Some(result) = self.org.job.poll_effect() else {
            return false;
        };
        self.org.publish_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Publish(result))) => self.finish_publishing(result),
            Err(e) => self.finish_publishing(Err(e.to_string())),
            _ => unreachable!("publishing owns this observation"),
        }
        true
    }
}
