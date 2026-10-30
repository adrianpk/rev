use super::*;
impl Work {
    pub(in crate::session) fn start_block(
        &mut self,
        stamp: Stamp,
        scan: crate::org::block::Scan,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let cancel = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::SourceBlock(scan.run(&cancel))
            })
            .map_err(|e| e.to_string())?;
        self.block_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn start_execution_input(
        &mut self,
        stamp: Stamp,
        prepare: crate::org::execution::Prepare,
        seconds: usize,
    ) -> Result<(), String> {
        self.job
            .start(stamp, move |cancel| {
                let cancel = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::ExecutionInput(prepare.run(&cancel))
            })
            .map_err(|e| e.to_string())?;
        self.block_running = true;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_block(&self) {
        if self.block_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_block(&mut self) -> bool {
        if self
            .block_request
            .as_ref()
            .is_none_or(|r| !self.org_source_current(r))
        {
            self.org.cancel_block();
        }
        let Some(result) = self.org.job.poll_owned() else {
            return false;
        };
        self.org.block_running = false;
        self.org.source = None;
        self.org.deadline = None;
        match result {
            Ok((_, Observation::ExecutionInput(result))) => self.finish_execution_input(result),
            Ok((_, Observation::SourceBlock(result))) => self.finish_block(result),
            Err(e) => self.finish_block(Err(e.to_string())),
            _ => unreachable!("source block owns observation"),
        }
        true
    }
}
