use super::*;
impl Work {
    pub(in crate::session) fn start_tangle(
        &mut self,
        stamp: Stamp,
        work: crate::org::tangle::JobWork,
        seconds: usize,
    ) -> Result<(), String> {
        let effect = work.effect();
        self.job
            .start(stamp, move |cancel| {
                let cancel = cancel.bounded(Instant::now() + Duration::from_secs(seconds as u64));
                Observation::Tangle(work.run(&cancel))
            })
            .map_err(|e| e.to_string())?;
        self.tangle_running = true;
        self.tangle_effect = effect;
        self.source = None;
        self.deadline = Some(Instant::now() + POLL);
        Ok(())
    }
    pub(in crate::session) fn cancel_tangle(&self) {
        if self.tangle_running {
            self.job.cancel();
        }
    }
}
impl Session {
    pub(super) fn poll_tangle(&mut self) -> bool {
        if !self.tangle_current() {
            self.org.cancel_tangle();
        }
        let observed = if self.org.tangle_effect {
            self.org.job.poll_effect()
        } else {
            self.org.job.poll_owned()
        };
        let Some(result) = observed else {
            return false;
        };
        self.org.tangle_running = false;
        self.org.tangle_effect = false;
        self.org.deadline = None;
        self.org.source = None;
        match result {
            Ok((_, Observation::Tangle(result))) => self.finish_tangle(result),
            Err(e) => self.finish_tangle(Err(e.to_string())),
            _ => unreachable!("tangle owns observation"),
        }
        true
    }
}
