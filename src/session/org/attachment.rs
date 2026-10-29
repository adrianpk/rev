use super::*;
use crate::org::attachment::{Context, Listing, Method, Plan};
impl Work {
    pub(in crate::session) fn start_attachment_listing(
        &mut self,
        stamp: Stamp,
        context: Context,
    ) -> Result<(), String> {
        let seconds = context.policy.limits.job_seconds;
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::AttachmentListing(Listing::read(&context, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.attachment_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn start_attachment_plan(
        &mut self,
        stamp: Stamp,
        context: Context,
        path: String,
        method: Method,
    ) -> Result<(), String> {
        let seconds = context.policy.limits.job_seconds;
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::AttachmentPlan(Plan::prepare(context, &path, method, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.attachment_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn start_attachment_effect(
        &mut self,
        stamp: Stamp,
        plan: Plan,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::AttachmentEffect(plan.run(&bounded))
            })
            .map_err(|e| e.to_string())?;
        self.attachment_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
impl Session {
    pub(super) fn poll_attachment(&mut self) -> bool {
        if self.attachment_source_changed() {
            self.org.job.cancel();
        }
        let Some(result) = self.org.job.poll_effect() else {
            return false;
        };
        self.org.attachment_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::AttachmentEffect(result))) => self.finish_attachment_effect(result),
            Ok((_, Observation::AttachmentPlan(result))) => self.finish_attachment_plan(result),
            Ok((_, Observation::AttachmentListing(result))) => {
                self.finish_attachment_listing(result)
            }
            Err(e) => {
                self.attachment = None;
                self.error(&e.to_string());
            }
            _ => unreachable!("attachment owns worker"),
        }
        true
    }
}
