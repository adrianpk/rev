use super::*;
use crate::{
    buffer::Buffers,
    config::Org,
    org::{
        Cancellation,
        io::{CaptureRead, DestinationSource},
        time::Date,
    },
};
use std::time::{Duration, Instant};

/// Immutable work for the existing session-owned job. A caller may discard it
/// without cleanup writes; even missing file/heading/list/table plans are data.
pub struct Discovery {
    read: CaptureRead,
    template: CaptureTemplate,
    policy: Org,
    date: Date,
}
pub struct Resolved {
    pub template: CaptureTemplate,
    pub source: DestinationSource,
    pub target: Target,
}
impl Choice {
    pub fn discover(
        &self,
        buffers: &Buffers,
        policy: &Org,
        date: Date,
    ) -> Result<Discovery, String> {
        self.template.validate()?;
        let file = self
            .file
            .as_deref()
            .ok_or("Enter a capture destination file")?;
        let read = CaptureRead::new(buffers, file, &policy.limits)?;
        Ok(Discovery {
            read,
            template: self.template.clone(),
            policy: policy.clone(),
            date,
        })
    }
}
impl Discovery {
    pub fn run(self, cancel: &Cancellation) -> Result<Resolved, String> {
        let bounded = cancel
            .bounded(Instant::now() + Duration::from_secs(self.policy.limits.job_seconds as u64));
        let source = self.read.run(&bounded)?;
        let target = Target::resolve(&source, &self.template, &self.policy, self.date, &bounded)?;
        if bounded.is_cancelled() {
            return Err("Capture destination discovery cancelled or timed out".into());
        }
        Ok(Resolved {
            template: self.template,
            source,
            target,
        })
    }
}
