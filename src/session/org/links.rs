use super::*;
use crate::{
    command::OrgCommand,
    org::links::{Resolve, Snapshot},
};
impl Work {
    pub(in crate::session) fn start_links(
        &mut self,
        snapshot: Snapshot,
        command: OrgCommand,
        wrap: bool,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(snapshot.stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Links(snapshot.observe(command, wrap, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.links_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn start_link_navigation(
        &mut self,
        stamp: Stamp,
        work: Resolve,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::LinkNavigation(work.run(&bounded))
            })
            .map_err(|e| e.to_string())?;
        self.links_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
impl Session {
    pub(super) fn poll_links(&mut self) -> bool {
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.links_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Links(result))) => self.finish_links(result),
            Ok((_, Observation::LinkNavigation(result))) => self.finish_link_navigation(result),
            Err(JobError::Cancelled) => self.links.request = None,
            Err(error) => {
                self.links.request = None;
                self.error(&error.to_string());
            }
            _ => unreachable!("link owns worker observation"),
        }
        true
    }
}
