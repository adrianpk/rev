use super::*;
use crate::{command::OrgCommand, org::table::Snapshot};
impl Work {
    pub(in crate::session) fn start_table_file(
        &mut self,
        stamp: Stamp,
        work: crate::org::table::interchange::Work,
        seconds: usize,
    ) -> Result<(), String> {
        let effect = matches!(work, crate::org::table::interchange::Work::Publish(_));
        self.job
            .start(stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Table(work.run(&bounded))
            })
            .map_err(|e| e.to_string())?;
        self.table_running = true;
        self.table_effect = effect;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }

    pub(in crate::session) fn start_table(
        &mut self,
        snapshot: Snapshot,
        command: OrgCommand,
        argument: Option<String>,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(snapshot.stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Table(snapshot.run(command, argument, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.table_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_table(&mut self) {
        if self.table_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_table(&mut self) -> bool {
        if self
            .table_request
            .as_ref()
            .is_none_or(|r| !self.org_source_current(r))
        {
            self.org.job.cancel();
        }
        let observed = if self.org.table_effect {
            self.org.job.poll_effect()
        } else {
            self.org.job.poll_owned()
        };
        let Some(result) = observed else {
            return false;
        };
        self.org.table_running = false;
        self.org.table_effect = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Table(result))) => self.finish_table(result),
            Err(error) => self.finish_table(Err(error.to_string())),
            _ => unreachable!("table action owns worker observation"),
        }
        true
    }
}
