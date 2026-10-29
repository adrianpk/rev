use super::*;
use crate::org::{
    agenda::SourceScan,
    refile::{Subtree, Targets},
};
impl Work {
    pub(in crate::session) fn start_refile_scan(
        &mut self,
        stamp: Stamp,
        scan: SourceScan,
        rows: usize,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::RefileTargets(Targets::observe(scan.run(&bounded), rows, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.refile_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn start_refile_plan(
        &mut self,
        targets: Targets,
        subtree: Subtree,
        choice: (usize, bool),
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(subtree.stamp(), move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::RefilePlan(targets.plan(choice.0, subtree, choice.1, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.refile_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
impl Session {
    pub(super) fn poll_refile_job(&mut self) -> bool {
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.refile_running = false;
        self.org.deadline = None;
        self.org.source = None;
        match result {
            Ok((_, Observation::RefileTargets(result))) => self.finish_refile_targets(result),
            Ok((_, Observation::RefilePlan(result))) => self.finish_refile_plan(result),
            Err(JobError::Cancelled) => {
                self.refile = None;
            }
            Err(error) => self.fail_refile(&error.to_string()),
            _ => unreachable!("refile owns this worker observation"),
        }
        true
    }
}

impl Work {
    pub(in crate::session) fn start_archive(
        &mut self,
        work: crate::org::archive::Archive,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(work.stamp(), move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::RefilePlan(work.run(&bounded))
            })
            .map_err(|e| e.to_string())?;
        self.refile_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
}
