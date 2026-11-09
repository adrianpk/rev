use super::*;
use crate::git::{IntegrationMode, Outcome, SequenceControl, SequenceFamily};

impl Work {
    fn sequence_lines(
        &self,
        width: usize,
    ) -> impl Iterator<Item = (String, Option<&std::path::Path>)> + '_ {
        let sequence = self
            .state
            .context
            .and_then(|id| self.engine.context(id))
            .and_then(|c| c.sequence.as_ref());
        (0..sequence.map_or(10, |s| {
            10 + s.targets.len()
                + s.conflicts.len()
                + s.bisect
                    .as_ref()
                    .map_or(0, |b| 3 + b.good.len() + b.skipped.len())
        }))
            .map(move |row| {
                let Some(s) = sequence else {
                    return (if row == 0 {
                        "Git Operation: checking sequence state".into()
                    } else {
                        String::new()
                    }, None);
                };
                let target = row.checked_sub(10 + s.targets.len()).and_then(|i| s.conflicts.get(i)).map(PathBuf::as_path);
                (match row {
                    0 => format!(
                        "Git Operation: {}",
                        s.family.map_or("none".into(), |f| format!("{f:?}"))
                    ),
                    1 => {
                        if s.family == Some(SequenceFamily::Bisect) {
                            self.bisect_summary()
                                .unwrap_or("Manual bisect state unavailable")
                                .into()
                        } else if s.family.is_some() {
                            "In progress; Git owns state; process cancellation does not abort it"
                                .into()
                        } else {
                            "No active Git-owned sequence".into()
                        }
                    }
                    2 => format!("Current HEAD ref: {}", label(&s.refs.head_ref)),
                    3 => format!(
                        "Current HEAD object: {}",
                        s.refs.head.map_or("unborn".into(), |id| id.hex())
                    ),
                    4 => format!(
                        "Original object: {}",
                        s.original.map_or("unavailable".into(), |id| id.hex())
                    ),
                    5 => format!(
                        "Applied/current revision: {}",
                        s.current.map_or("unavailable".into(), |id| id.hex())
                    ),
                    6 => format!(
                        "Onto: {}",
                        s.onto.map_or("unavailable".into(), |id| id.hex())
                    ),
                    7 => format!(
                        "Step {} / {}",
                        s.step.map_or("unavailable".into(), |n| n.to_string()),
                        s.total.map_or("unavailable".into(), |n| n.to_string())
                    ),
                    8 => format!("Exact conflict paths: {}", s.conflicts.len()),
                    9 if self.engine.progress().any(|p| {
                        p.context.is_none()
                            || p.context
                                .and_then(|id| self.engine.context(id))
                                .is_some_and(|c| {
                                    c.observation.identity.common_directory
                                        == s.refs.observation.identity.common_directory
                                })
                    }) =>
                    {
                        "Checking Git owner; sequence controls wait for owned work".into()
                    }
                    9 if s.family == Some(SequenceFamily::Bisect) => format!(
                        "Admitted manual controls: {}",
                        [
                            crate::git::BisectAction::Good,
                            crate::git::BisectAction::Bad,
                            crate::git::BisectAction::Skip,
                            crate::git::BisectAction::Reset
                        ]
                        .into_iter()
                        .filter(|a| s.bisect_admission(*a).is_ok())
                        .map(crate::git::BisectAction::label)
                        .collect::<Vec<_>>()
                        .join(", ")
                    ),
                    9 => format!(
                        "Admitted controls: {}",
                        [
                            SequenceControl::Continue,
                            SequenceControl::Skip,
                            SequenceControl::Abort
                        ]
                        .into_iter()
                        .filter(|action| s.control_admission(*action, None).is_ok())
                        .map(SequenceControl::label)
                        .collect::<Vec<_>>()
                        .join(", ")
                    ),
                    _ if row - 10 < s.targets.len() => {
                        format!("Merge target: {}", s.targets[row - 10].hex())
                    }
                    _ if row - 10 - s.targets.len() < s.conflicts.len() => label(
                        s.conflicts[row - 10 - s.targets.len()]
                            .as_os_str()
                            .as_encoded_bytes(),
                    ),
                    _ => {
                        let b = s.bisect.as_ref().expect("admitted bisect row");
                        let index = row - 10 - s.targets.len() - s.conflicts.len();
                        match index {
                            0 => format!("Known bad: {}", b.bad.hex()),
                            1 => format!(
                                "Unresolved range: {} commits / {} skipped",
                                b.remaining,
                                b.skipped.len()
                            ),
                            2 => format!("Manual result: {}", b.result.label()),
                            _ if index - 3 < b.good.len() => {
                                format!("Known good: {}", b.good[index - 3].hex())
                            }
                            _ => format!(
                                "Skipped candidate: {}",
                                b.skipped[index - 3 - b.good.len()].hex()
                            ),
                        }
                    }
                }, target)
            })
            .flat_map(move |(text, target)| Wrapped {
                text: if target.is_some() { format!("{} {}", if target == self.conflict_path() { ">" } else { " " }, text) } else { text },
                offset: Some(0),
                width: width.max(1),
            }.map(move |text| (text, target)))
    }
    pub(in crate::session::git) fn sequence_row(&self, row: usize, width: usize) -> (String, bool) {
        self.sequence_lines(width)
            .nth(self.refs.sequence_top + row)
            .map_or_else(
                || (String::new(), false),
                |(text, path)| (text, path.is_some() && path == self.conflict_path()),
            )
    }
}
impl Session {
    pub(in crate::session::git) fn move_git_sequence(
        &mut self,
        distance: usize,
        backward: bool,
    ) -> Action {
        let width = usize::from(self.size.width);
        if let Some(sequence) = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.sequence.as_ref())
            && !sequence.conflicts.is_empty()
        {
            let at = self
                .git
                .conflict_path()
                .and_then(|p| sequence.conflicts.iter().position(|c| c == p))
                .unwrap_or(0);
            let next = if backward {
                at.saturating_sub(distance)
            } else {
                at.saturating_add(distance)
                    .min(sequence.conflicts.len() - 1)
            };
            self.git.conflicts.selected = Some(sequence.conflicts[next].clone());
            let selected_row = self
                .git
                .sequence_lines(width)
                .position(|(_, p)| p.is_some() && p == self.git.conflict_path());
            if let Some(row) = selected_row {
                self.git.refs.sequence_top =
                    row.saturating_sub(self.git_extent().saturating_sub(1));
            }
            self.git.painted = None;
            return Action::Redraw;
        }
        let count = self
            .git
            .sequence_lines(usize::from(self.size.width))
            .count();
        self.git.refs.sequence_top = if backward {
            self.git.refs.sequence_top.saturating_sub(distance)
        } else {
            self.git
                .refs
                .sequence_top
                .saturating_add(distance)
                .min(count.saturating_sub(1))
        };
        self.git.painted = None;
        Action::Redraw
    }
    pub(in crate::session::git) fn select_git_sequence(&mut self, row: usize) -> Action {
        let target = self
            .git
            .sequence_lines(usize::from(self.size.width))
            .nth(row)
            .and_then(|(_, p)| p.map(std::path::Path::to_path_buf));
        if let Some(path) = target {
            self.git.conflicts.selected = Some(path);
            self.git.painted = None;
        }
        Action::Redraw
    }
    pub(in crate::session::git) fn confirm_git_integration(
        &mut self,
        mode: IntegrationMode,
    ) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select a current exact integration target in Refs");
        };
        if let Err(e) = self.git.engine.check_refs(&refs) {
            return self.error(&e);
        }
        let Some(reference) = self
            .git
            .refs
            .selected
            .as_ref()
            .and_then(|n| refs.reference(n))
        else {
            return self.message("Select one exact ref as integration target");
        };
        let Some(target) = reference.commit() else {
            return self.message("Integration target must reference one exact commit");
        };
        let sources = match self.git_branch_sources(&refs) {
            Ok(s) => s,
            Err(e) => return self.error(&format!("Integration source admission: {e}")),
        };
        let message = format!(
            "Confirm {} with {} at {}? y=accept, n=cancel",
            mode.label(),
            label(&reference.name),
            target.hex()
        );
        let review = format!(
            "Checked integration review\nPolicy: {}\nCurrent branch:\n{}\nCurrent HEAD:\n{}\nSelected full ref:\n{}\nExact target commit:\n{}\nComplete clean disk/index and unchanged unmodified visiting sources are required.\nNo implicit autostash. y accepts; n or C-g cancels.",
            mode.label(),
            label(&refs.head_ref),
            refs.head.map_or("unborn".into(), |id| id.hex()),
            label(&reference.name),
            target.hex()
        );
        let name = reference.name.clone();
        self.confirm_git_ref(
            refs,
            RefAction::Integration { target: name, mode },
            Some(sources),
            message,
            review,
        )
    }
    pub(in crate::session::git) fn open_git_sequence(&mut self) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before inspecting Git-owned sequence state");
        };
        match self.git.engine.sequence(context) {
            Ok(id) => {
                self.git.refs.integration_request = Some(id);
                self.git.refs.control_preparing = None;
                self.git.page = Page::Sequence;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking Git-owned sequence state and exact conflicts")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn request_git_control(
        &mut self,
        action: SequenceControl,
        family: Option<SequenceFamily>,
    ) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before requesting sequence control");
        };
        match self.git.engine.sequence(context) {
            Ok(id) => {
                self.git.refs.integration_request = Some(id);
                self.git.refs.control_preparing = Some((action, family));
                self.git.page = Page::Sequence;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking the actual Git-owned sequence before control review")
            }
            Err(e) => self.error(&e),
        }
    }
    fn confirm_git_control(
        &mut self,
        action: SequenceControl,
        family: Option<SequenceFamily>,
    ) -> Action {
        let Some(sequence) = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.sequence.clone())
        else {
            return self.message("No fresh Git sequence observation is available");
        };
        if let Err(e) = sequence.control_admission(action, family) {
            return self.error(&e);
        }
        let sources = match self.git_branch_sources(&sequence.refs) {
            Ok(s) => s,
            Err(e) => return self.error(&e),
        };
        let message = format!(
            "Confirm {} of actual {:?} sequence? y=accept, n=cancel",
            action.label(),
            sequence.family
        );
        let review = format!(
            "Checked sequence control review\nActual family: {:?}\nAction: {}\nCurrent HEAD ref: {}\nCurrent HEAD: {}\nOriginal object: {}\nOriginal branch: {}\nOnto: {}\nCurrent revision: {}\nStep {:?} / {:?}\nConflicts: {}\nExact Git metadata, refs, index, working tree and visiting source owners must remain unchanged.\nCancellation does not abort. y / C-Enter / C-c C-c accept; n / C-g cancel.",
            sequence.family,
            action.label(),
            label(&sequence.refs.head_ref),
            sequence.refs.head.map_or("unborn".into(), |id| id.hex()),
            sequence
                .original
                .map_or("unavailable".into(), |id| id.hex()),
            label(&sequence.branch),
            sequence.onto.map_or("unavailable".into(), |id| id.hex()),
            sequence.current.map_or("unavailable".into(), |id| id.hex()),
            sequence.step,
            sequence.total,
            sequence.conflicts.len()
        );
        self.confirm_git_ref(
            std::sync::Arc::clone(&sequence.refs),
            RefAction::Control {
                sequence,
                action,
                family,
            },
            Some(sources),
            message,
            review,
        )
    }
    pub(in crate::session::git) fn prepare_git_autosquash(&mut self) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select one exact autosquash target in current Refs");
        };
        let Some(target) = self.git.refs.selected.clone() else {
            return self.message("Select one exact autosquash target ref");
        };
        if let Err(e) = self.git_branch_sources(&refs) {
            return self.error(&e);
        }
        match self.git.engine.prepare_autosquash(refs, &target) {
            Ok(id) => {
                self.git.refs.autosquash_preparing = Some(id);
                self.git.painted = None;
                self.message("Preparing the finite autosquash plan without starting a Git sequence")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn publish_git_autosquash_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.refs.autosquash_preparing = None;
        if !self.git.visible
            || self.git.page != Page::Refs
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let Some(plan) = update
            .context
            .filter(|_| update.published && self.git.state.context == update.context)
            .and_then(|id| self.git.engine.context(id))
            .and_then(|c| c.autosquash.clone())
        else {
            let detail = self
                .git
                .engine
                .record(update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("inspect git-processes");
            self.message(&format!("Autosquash preparation failed: {detail}"));
            return;
        };
        let sources = match self.git_branch_sources(&plan.refs) {
            Ok(s) => s,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        let mut review = format!(
            "Reviewed autosquash plan\nCurrent branch: {}\nOriginal HEAD: {}\nSelected full target ref: {}\nExact target: {}\nSource merge base: {}\nOrdered actions ({} / 256 admitted):\n",
            label(&plan.refs.head_ref),
            plan.refs.head.map_or("unborn".into(), |id| id.hex()),
            label(&plan.target_name),
            plan.target.hex(),
            plan.base.hex(),
            plan.steps.len()
        );
        // The plan is already admitted. Reserve presentation before allocating
        // its bounded full-object/subject review; the consent retains it next.
        let size = review.len()
            + plan
                .steps
                .iter()
                .map(|s| s.subject.len() + 128)
                .sum::<usize>();
        let _admission = match self.git.engine.source_admission(size * 2) {
            Ok(a) => a,
            Err(e) => {
                self.error(&e);
                return;
            }
        };
        for step in &plan.steps {
            review.push_str(&format!(
                "{} {} {}\n",
                step.action.label(),
                step.object.hex(),
                step.subject
            ));
        }
        review.push_str("Git keeps hooks/signing authority. No implicit autostash or executable todo actions. y / C-Enter / C-c C-c accept; n / C-g cancel.");
        self.confirm_git_ref(
            std::sync::Arc::clone(&plan.refs),
            RefAction::Autosquash(plan),
            Some(sources),
            "Confirm this exact ordered autosquash plan? y=accept, n=cancel".into(),
            review,
        );
    }
    pub(in crate::session::git) fn publish_git_integration(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            self.git.refs.failed = true;
            return;
        }
        self.git.refs.integration_request = None;
        let pending_control = self.git.refs.control_preparing.take();
        if pending_control.is_some() {
            self.git.refs.failed = !update.published;
            if update.published {
                self.git.state.context = update.context;
            }
            self.observe_files();
            self.git.painted = None;
        } else {
            self.publish_git_refs(update);
        }
        let record = self.git.engine.record(update.operation);
        let in_progress = record.is_some_and(|r| r.outcome == Outcome::InProgress);
        let completed =
            record.is_some_and(|r| matches!(r.outcome, Outcome::Skipped | Outcome::Aborted));
        let aborted = record.is_some_and(|r| r.outcome == Outcome::Aborted);
        if update.published && completed {
            self.git.refs.failed = false;
        }
        let bisect = record.is_some_and(|r| r.kind == crate::git::OperationKind::Bisect);
        let observed = record.is_some_and(|r| r.kind == crate::git::OperationKind::Sequence);
        if update.published && (in_progress || observed || bisect) {
            self.git.page = Page::Sequence;
            self.git.refs.sequence_top = 0;
            self.git.refs.failed = false;
        }
        if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing {
            if let Some((action, family)) = pending_control
                .filter(|_| update.published && self.git.state.context == update.context)
            {
                self.confirm_git_control(action, family);
                return;
            }
            self.message(if bisect && update.published && !self.git.refs.failed {
                self.git
                    .bisect_summary()
                    .unwrap_or("Manual bisect state checked")
            } else if aborted {
                "Git sequence aborted; original branch and object verified"
            } else if completed {
                "Git sequence skipped; resulting branch and lineage verified"
            } else if in_progress {
                "Git integration stopped in progress; inspect the exact sequence and conflicts"
            } else if self.git.refs.failed {
                "Integration failed or effects remain uncertain; inspect git-processes"
            } else if observed {
                "Git-owned sequence state checked"
            } else {
                "Integration verified; source reload follows file policy"
            });
        }
    }
}
