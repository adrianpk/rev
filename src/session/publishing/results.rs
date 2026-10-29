use super::*;
impl Session {
    /// Retains one bounded project operation, including every partial failure.
    pub fn publishing_results(&self) -> &[publish::Outcome] {
        &self.publishing_results
    }
    pub(in crate::session) fn end_publishing(&mut self, detail: &str) {
        if let Some(mut state) = self.publishing.take() {
            if let Some((source, target)) = state.active.take() {
                self.publishing_results.push(publish::Outcome::failed(
                    source,
                    target,
                    detail.into(),
                ));
            }
            self.cancel_project_items(state.plan, detail);
        }
        self.show_publishing_summary(detail);
    }
    fn cancel_project_items(&mut self, plan: Option<Plan>, detail: &str) {
        let Some(plan) = plan else {
            return;
        };
        for item in plan.items {
            let outcome = match item {
                Item::Ready(p) => p.failed(detail),
                Item::Issue(issue) => issue,
            };
            self.publishing_results.push(outcome);
        }
    }
    pub(super) fn show_publishing_summary(&mut self, detail: &str) {
        let complete = self
            .publishing_results
            .iter()
            .filter(|r| r.publication == Publication::Complete)
            .count();
        let total = self.publishing_results.len();
        self.message(&format!(
            "{detail}: {complete}/{total} files complete; org-publish-results shows each outcome"
        ));
    }
    pub(in crate::session) fn publishing_result_prompt(&mut self) -> Action {
        if self.publishing.is_some() {
            return self.message("Project publication pending; C-g cancels remaining files");
        }
        if self.publishing_results.is_empty() {
            return self.message("No project publishing results");
        }
        let label = format!(
            "Publishing result (1..{}; empty summary): ",
            self.publishing_results.len()
        );
        match self.publishing_prompt(Prompt::Result, &label) {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session) fn publishing_result(&mut self, draft: &str) -> Action {
        if draft.is_empty() {
            self.show_publishing_summary("Project outcomes");
            return Action::Redraw;
        }
        let result = draft
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| self.publishing_results.get(n));
        let Some(result) = result else {
            return self.error("Choose a recorded publishing result number");
        };
        self.message(&format!(
            "Project {:?}: {} -> {}; {}",
            result.publication,
            result.source.display(),
            result.target.display(),
            result.detail
        ))
    }
}
