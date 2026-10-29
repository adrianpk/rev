use super::*;
use crate::{
    command::OrgCommand,
    org::document::{Argument, Snapshot},
};
impl Work {
    pub(in crate::session) fn start_document(
        &mut self,
        snapshot: Snapshot,
        command: OrgCommand,
        argument: Argument,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(snapshot.stamp, move |cancel| {
                let bounded = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Document(snapshot.run(command, argument, &bounded))
            })
            .map_err(|e| e.to_string())?;
        self.document_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_document(&mut self) {
        if self.document_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_document(&mut self) -> bool {
        if self
            .document_request
            .as_ref()
            .is_none_or(|r| !self.org_source_current(r))
        {
            self.org.job.cancel();
        }
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.document_running = false;
        self.org.deadline = None;
        self.org.source = None;
        match result {
            Ok((_, Observation::Document(result))) => self.finish_document(result),
            Err(error) => self.finish_document(Err(error.to_string())),
            _ => unreachable!("document action owns worker observation"),
        }
        true
    }
}
