use super::*;
use crate::git::{IntegrationMode, Outcome};

impl Work {
    fn sequence_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        let sequence = self
            .state
            .context
            .and_then(|id| self.engine.context(id))
            .and_then(|c| c.sequence.as_ref());
        (0..sequence.map_or(9, |s| 9 + s.targets.len() + s.conflicts.len()))
            .map(move |row| {
                let Some(s) = sequence else {
                    return if row == 0 {
                        "Git Operation: checking sequence state".into()
                    } else {
                        String::new()
                    };
                };
                match row {
                    0 => format!(
                        "Git Operation: {}",
                        s.family.map_or("none".into(), |f| format!("{f:?}"))
                    ),
                    1 => {
                        if s.family.is_some() {
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
                    _ if row - 9 < s.targets.len() => {
                        format!("Merge target: {}", s.targets[row - 9].hex())
                    }
                    _ => label(
                        s.conflicts[row - 9 - s.targets.len()]
                            .as_os_str()
                            .as_encoded_bytes(),
                    ),
                }
            })
            .flat_map(move |text| Wrapped {
                text,
                offset: Some(0),
                width: width.max(1),
            })
    }
    pub(in crate::session::git) fn sequence_row(&self, row: usize, width: usize) -> (String, bool) {
        (
            self.sequence_lines(width)
                .nth(self.refs.sequence_top + row)
                .unwrap_or_default(),
            false,
        )
    }
}
impl Session {
    pub(in crate::session::git) fn move_git_sequence(
        &mut self,
        distance: usize,
        backward: bool,
    ) -> Action {
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
                self.git.page = Page::Sequence;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking Git-owned sequence state and exact conflicts")
            }
            Err(e) => self.error(&e),
        }
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
        self.publish_git_refs(update);
        let record = self.git.engine.record(update.operation);
        let in_progress = record.is_some_and(|r| r.outcome == Outcome::InProgress);
        let observed = record.is_some_and(|r| r.kind == crate::git::OperationKind::Sequence);
        if update.published && (in_progress || observed) {
            self.git.page = Page::Sequence;
            self.git.refs.sequence_top = 0;
            self.git.refs.failed = false;
        }
        if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing {
            self.message(if in_progress {
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
