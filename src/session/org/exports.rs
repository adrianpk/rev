use super::*;
use crate::org::export::output::Work as ExportWork;
impl Work {
    pub(in crate::session) fn start_export(
        &mut self,
        stamp: Stamp,
        work: ExportWork,
        seconds: usize,
    ) -> Result<(), String> {
        let effect = work.effect();
        self.job
            .start(stamp, move |cancel| {
                let cancel = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Export(work.run(&cancel))
            })
            .map_err(|e| e.to_string())?;
        self.export_running = true;
        self.export_effect = effect;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_export(&self) {
        if self.export_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_export(&mut self) -> bool {
        if self
            .export_request
            .as_ref()
            .is_none_or(|r| !self.org_source_current(r))
        {
            self.org.cancel_export();
        }
        let observed = if self.org.export_effect {
            self.org.job.poll_effect()
        } else {
            self.org.job.poll_owned()
        };
        let Some(result) = observed else {
            return false;
        };
        self.org.export_running = false;
        self.org.export_effect = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::Export(result))) => self.finish_export(result),
            Err(e) => self.finish_export(Err(e.to_string())),
            _ => unreachable!("export owns this observation"),
        }
        true
    }
}
