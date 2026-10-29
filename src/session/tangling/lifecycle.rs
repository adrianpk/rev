use super::*;
impl Session {
    pub(super) fn cancel_tangle(&mut self) -> Action {
        if let Some(Pending::Tangle(Prompt::Replace(file))) = self.pending.take() {
            self.tangling_results
                .push(file.failed("Tangle cancelled before replacement"));
            if let Some(s) = &mut self.tangling {
                s.active = None;
            }
        }
        self.input.cancel();
        if self.org.tangle_running {
            if let Some(s) = &mut self.tangling {
                s.cancelled = true;
            }
            self.org.cancel_tangle();
            return self.message("Tangle cancellation requested; awaiting actual file outcome");
        }
        self.end_tangle("Tangle cancelled");
        Action::Redraw
    }
    pub(super) fn end_tangle(&mut self, detail: &str) {
        if let Some(mut state) = self.tangling.take() {
            if let Some(target) = state.active.take() {
                self.tangling_results.push(tangle::Outcome {
                    target,
                    publication: Publication::NotPublished,
                    detail: detail.into(),
                });
            }
            if let Some(plan) = state.plan {
                for file in plan.files {
                    self.tangling_results.push(file.failed(detail));
                }
            }
        }
        self.tangle_summary(detail);
    }
    pub(super) fn tangle_summary(&mut self, detail: &str) -> Action {
        let complete = self
            .tangling_results
            .iter()
            .filter(|r| r.publication == Publication::Complete)
            .count();
        self.message(&format!(
            "{detail}: {complete}/{} files complete; org-babel-tangle-results shows each outcome",
            self.tangling_results.len()
        ))
    }
    pub(super) fn tangle_result_prompt(&mut self) -> Action {
        if self.tangling_results.is_empty() {
            return self.message("No tangle outcomes");
        }
        let label = format!(
            "Tangle result (1..{}; empty summary): ",
            self.tangling_results.len()
        );
        match self.input.argument(&label, "") {
            Ok(()) => {
                self.pending = Some(Pending::Tangle(Prompt::Result));
                Action::Redraw
            }
            Err(e) => self.error(&e.to_string()),
        }
    }
    pub(super) fn tangle_result(&mut self, value: &str) -> Action {
        if value.is_empty() {
            return self.tangle_summary("Tangle outcomes");
        }
        let result = value
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| self.tangling_results.get(n));
        let Some(result) = result else {
            return self.error("Choose a recorded tangle result number");
        };
        self.message(&format!(
            "Tangle {:?}: {}; {}",
            result.publication,
            result.target.display(),
            result.detail
        ))
    }
}
