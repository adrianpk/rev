use super::*;
use crate::git::{ApplyKind, ApplyPlan, History, RevisionChoice};

pub(in crate::session) struct Selection {
    history: Arc<History>,
    kind: ApplyKind,
    choices: Vec<RevisionChoice>,
    _admission: crate::git::budget::Lease,
}
impl Session {
    pub(in crate::session::git) fn prepare_git_application(&mut self, kind: ApplyKind) -> Action {
        let (history, selected) = match self.git.revision_target() {
            Ok(t) => t,
            Err(e) => return self.error(&e),
        };
        let targets = if self.git.ancestry.ordered.is_empty() {
            vec![selected]
        } else {
            if self.git.ancestry.mark_scope != Some((history.observation.context, history.root)) {
                return self.message("Ordered revision mark context changed; explicitly clear or select the set again");
            }
            self.git.ancestry.ordered.clone()
        };
        let admission = match self
            .git
            .engine
            .source_admission(targets.len() * std::mem::size_of::<RevisionChoice>() + 4096)
        {
            Ok(a) => a,
            Err(e) => return self.error(&e),
        };
        let choices = targets
            .into_iter()
            .map(|object| RevisionChoice {
                object,
                mainline: None,
            })
            .collect();
        self.advance_git_application(Selection {
            history,
            kind,
            choices,
            _admission: admission,
        })
    }
    fn advance_git_application(&mut self, selection: Selection) -> Action {
        let next = selection.choices.iter().find(|choice| {
            choice.mainline.is_none()
                && selection
                    .history
                    .revision(choice.object)
                    .is_some_and(|r| r.parents.len() > 1)
        });
        if let Some(choice) = next {
            let count = selection
                .history
                .revision(choice.object)
                .expect("admitted merge")
                .parents
                .len();
            let prompt = format!(
                "Mainline parent number for {} (1..{}): ",
                choice.object.hex(),
                count
            );
            if let Err(e) = self.input.argument(&prompt, "") {
                return self.error(&e.to_string());
            }
            self.pending = Some(Pending::GitSequenceMainline(Box::new(selection)));
            return self.message("Choose an explicit merge parent number; exact parent objects and order appear in the final review");
        }
        match self.git.engine.prepare_sequence(
            selection.history,
            selection.kind,
            &selection.choices,
        ) {
            Ok(id) => {
                self.git.refs.application_preparing = Some(id);
                self.git.painted = None;
                self.message("Preparing the exact ordered revisions and mainline choices without changing Git state")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session) fn git_sequence_mainline(
        &mut self,
        mut selection: Selection,
        text: &str,
    ) -> Action {
        let Some(choice) = selection.choices.iter_mut().find(|choice| {
            choice.mainline.is_none()
                && selection
                    .history
                    .revision(choice.object)
                    .is_some_and(|r| r.parents.len() > 1)
        }) else {
            return self.message("Merge parent prompt expired");
        };
        let count = selection
            .history
            .revision(choice.object)
            .expect("admitted merge")
            .parents
            .len();
        let Some(n) = text.parse::<usize>().ok().filter(|n| *n > 0 && *n <= count) else {
            return self
                .message("Mainline must be one explicit parent number from the captured merge");
        };
        choice.mainline = Some(n);
        self.advance_git_application(selection)
    }
    pub(in crate::session::git) fn publish_git_application_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.refs.application_preparing = None;
        if !self.git.visible
            || self.git.page != Page::History
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(plan) = update
            .context
            .filter(|_| update.published && update.context == self.git.state.context)
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.application_plan.clone())
        else {
            let detail = self
                .git
                .engine
                .record(update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("inspect git-processes");
            self.message(&format!("Ordered revision preparation failed: {detail}"));
            return;
        };
        self.confirm_git_application(plan);
    }
    fn confirm_git_application(&mut self, plan: Arc<ApplyPlan>) -> Action {
        let sources = match self.git_branch_sources(&plan.refs) {
            Ok(s) => s,
            Err(e) => return self.error(&e),
        };
        let size = 16384
            + plan
                .steps
                .iter()
                .map(|s| s.subject.len() * 4 + 512)
                .sum::<usize>();
        let _admission = match self.git.engine.source_admission(size * 2) {
            Ok(a) => a,
            Err(e) => return self.error(&e),
        };
        let mut review = format!(
            "Ordered revision review\nAction: {}\nOriginal current branch:\n{}\nOriginal HEAD:\n{}\nSource history root:\n{}\nOrdered targets: {} / 256\n",
            plan.kind.label(),
            label(&plan.refs.head_ref),
            plan.refs.head.map_or("unborn".into(), |id| id.hex()),
            plan.history
                .root
                .map_or("unavailable".into(), |id| id.hex()),
            plan.steps.len()
        );
        for (n, step) in plan.steps.iter().enumerate() {
            review.push_str(&format!(
                "{}: {} {}\n",
                n + 1,
                step.object.hex(),
                label(&step.subject)
            ));
            if let (Some(mainline), Some(parent)) = (step.mainline, step.parent) {
                review.push_str(&format!(
                    "Mainline parent {}:\n{}\n",
                    mainline,
                    parent.hex()
                ));
            }
        }
        review.push_str("One finite confirmed order; a stop requires explicit sequence control. Git retains hooks, authored messages and signing policy. Clean disk/index and unchanged unmodified visiting sources are required. y / C-Enter / C-c C-c accepts; n / C-g cancels.");
        let message = format!(
            "Confirm {} of these {} exact ordered revisions? y=accept, n=cancel",
            plan.kind.label(),
            plan.steps.len()
        );
        self.confirm_git_ref(
            plan.refs.clone(),
            RefAction::Application(plan),
            Some(sources),
            message,
            review,
        )
    }
}
