use super::super::Pending;
pub(in crate::session) mod tag;
use super::*;
use crate::{
    buffer::Stamp,
    git::{BranchAction, Completeness, RefKind, Refs, Update},
};

pub(super) struct Inspection {
    pub request: Option<OperationId>,
    pub tag_preparing: Option<OperationId>,
    selected: Option<Vec<u8>>,
    pub top: usize,
    failed: bool,
    _admission: crate::git::budget::Lease,
}
impl Inspection {
    pub(super) fn new(engine: &Repositories) -> std::io::Result<Self> {
        Ok(Self {
            request: None,
            tag_preparing: None,
            selected: None,
            top: 0,
            failed: false,
            _admission: engine
                .source_admission(65536)
                .map_err(std::io::Error::other)?,
        })
    }
}
#[derive(Clone, Copy)]
pub(in crate::session) enum Intent {
    Create,
    Switch,
    Rename,
    Delete,
}
pub(in crate::session) struct Prompt {
    refs: Arc<Refs>,
    intent: Intent,
    selected: Option<Vec<u8>>,
}
pub(in crate::session) struct Consent {
    refs: Arc<Refs>,
    action: RefAction,
    sources: Option<Sources>,
    pub message: String,
    review: String,
    top: usize,
    _admission: crate::git::budget::Lease,
}
enum RefAction {
    Branch(BranchAction),
    CreateTag(Arc<crate::git::TagIntent>),
    DeleteTag(Vec<u8>),
}

struct Sources {
    stamps: Vec<Stamp>,
    _admission: crate::git::budget::Lease,
}

impl Work {
    pub(super) fn refs(&self) -> Option<&Arc<Refs>> {
        self.state
            .context
            .and_then(|id| self.engine.context(id))?
            .refs
            .as_ref()
    }
    pub(super) fn refs_row(&self, row: usize) -> (String, bool) {
        let Some(refs) = self.refs() else {
            return (
                if row == 0 {
                    "Git Refs: checking repository".into()
                } else {
                    String::new()
                },
                false,
            );
        };
        if row == 0 {
            return (
                format!(
                    "HEAD: {} {}",
                    if refs.head_ref.is_empty() {
                        "detached".into()
                    } else {
                        label(&refs.head_ref)
                    },
                    refs.head.map_or("unborn".into(), |id| id.hex())
                ),
                false,
            );
        }
        if row == 1 {
            return (
                format!(
                    "{} refs / {} remotes / {}{}",
                    refs.entries.len(),
                    refs.remotes.len(),
                    if refs.completeness == Completeness::Complete {
                        "complete"
                    } else {
                        "TRUNCATED"
                    },
                    if self.refs.failed {
                        " / STALE: refresh failed; $ diagnostics"
                    } else {
                        ""
                    }
                ),
                false,
            );
        }
        if row == 2 {
            return (
                if self.refs.request.is_some() {
                    "Checking / executing / refreshing refs".into()
                } else {
                    "Exact full names and objects; local actions require review".into()
                },
                false,
            );
        }
        let index = self.refs.top + row - 3;
        if let Some(r) = refs.entries.get(index) {
            let selected = self.refs.selected.as_deref() == Some(r.name.as_slice());
            return (
                format!(
                    "{} {} {}{}{}{}{}",
                    if selected { ">" } else { " " },
                    label(&r.name),
                    r.object.hex(),
                    if r.object_type == b"tag" {
                        format!(
                            " tag -> {}",
                            r.peeled.map_or("unavailable".into(), |id| id.hex())
                        )
                    } else {
                        String::new()
                    },
                    if r.symbolic.is_empty() {
                        String::new()
                    } else {
                        format!(" symbolic -> {}", label(&r.symbolic))
                    },
                    if r.upstream.is_empty() {
                        String::new()
                    } else {
                        format!(" upstream {}", label(&r.upstream))
                    },
                    if r.tracking.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", label(&r.tracking))
                    }
                ),
                selected,
            );
        }
        (
            refs.remotes
                .get(index.saturating_sub(refs.entries.len()))
                .map_or(String::new(), |r| format!("  remote {}", label(r))),
            false,
        )
    }
    pub(super) fn refs_menu(&self, row: usize) -> (String, bool) {
        (
            [
                "Git Refs actions",
                "n  git-branch-create: exact selected commit / HEAD",
                "S  git-branch-switch: checked clean source state",
                "R  git-branch-rename: exact local branch",
                "D  git-branch-delete: merged, noncurrent local branch",
                "t / T  git-tag-create / git-tag-delete (confirmed local refs)",
                "g  git-refresh",
                "$  git-processes",
                "M-x / :  native commands",
                "q / C-g  return to Refs",
            ]
            .get(row)
            .unwrap_or(&"")
            .to_string(),
            false,
        )
    }
}
impl Session {
    pub(in crate::session) fn cancel_git_ref(&mut self) -> Option<Action> {
        if !matches!(
            self.pending,
            Some(Pending::GitRef(_) | Pending::GitBranchName(_) | Pending::GitTagPrompt(_))
        ) {
            return None;
        }
        self.pending = None;
        self.input.cancel();
        Some(self.message("Local ref action canceled"))
    }

    pub(super) fn open_git_refs(&mut self) -> Action {
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Open Git Status before selecting Refs");
        };
        match self.git.engine.refs(context) {
            Ok(id) => {
                self.git.refs.request = Some(id);
                self.git.page = Page::Refs;
                self.git.state.menu = false;
                self.git.painted = None;
                self.message("Checking exact Git refs and configured remote names")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn select_git_ref(&mut self, index: usize) -> Action {
        let Some(refs) = self.git.refs() else {
            return Action::Wait;
        };
        let Some(r) = refs.entries.get(index) else {
            return Action::Wait;
        };
        self.git.refs.selected = Some(r.name.clone());
        self.git.refs.top = self
            .git
            .refs
            .top
            .min(index)
            .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
        Action::Redraw
    }
    pub(super) fn move_git_refs(&mut self, distance: usize, backward: bool) -> Action {
        let Some(refs) = self.git.refs() else {
            return Action::Wait;
        };
        let index = refs
            .entries
            .iter()
            .position(|r| Some(&r.name) == self.git.refs.selected.as_ref())
            .unwrap_or(0);
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min(refs.entries.len().saturating_sub(1))
        };
        self.select_git_ref(next)
    }
    pub(super) fn prompt_git_branch(&mut self, intent: Intent) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select a current Refs observation first");
        };
        if let Err(e) = self.git.engine.check_refs(&refs) {
            return self.error(&e);
        }
        let prompt = Prompt {
            refs,
            intent,
            selected: self.git.refs.selected.clone(),
        };
        if matches!(intent, Intent::Create | Intent::Rename) {
            if let Err(e) = self.input.argument("New local branch name: ", "") {
                return self.error(&e.to_string());
            }
            self.pending = Some(Pending::GitBranchName(Box::new(prompt)));
            Action::Redraw
        } else {
            self.git_branch_name(prompt, "")
        }
    }
    pub(in crate::session) fn git_branch_name(&mut self, prompt: Prompt, value: &str) -> Action {
        if matches!(prompt.intent, Intent::Create | Intent::Rename)
            && (value.is_empty() || value.len() > 4096 || value.contains('\0'))
        {
            return self.message("Branch name requires 1..=4096 non-NUL bytes");
        }
        let selected = prompt
            .selected
            .as_ref()
            .and_then(|n| prompt.refs.reference(n));
        let action = match prompt.intent {
            Intent::Create => {
                let Some(target) = selected.map_or(prompt.refs.head, |r| r.commit()) else {
                    return self.message("Branch creation requires an exact commit target");
                };
                if value.is_empty() || value.len() > 4096 || value.contains('\0') {
                    return self.message("Branch name requires 1..=4096 non-NUL bytes");
                }
                BranchAction::Create {
                    name: value.into(),
                    target,
                }
            }
            intent => {
                let Some(selected) =
                    selected.filter(|r| r.kind == RefKind::Branch && r.symbolic.is_empty())
                else {
                    return self.message("Select one exact local branch first");
                };
                match intent {
                    Intent::Switch => BranchAction::Switch {
                        name: selected.name.clone(),
                    },
                    Intent::Rename => BranchAction::Rename {
                        name: selected.name.clone(),
                        new_name: value.into(),
                    },
                    Intent::Delete => BranchAction::Delete {
                        name: selected.name.clone(),
                    },
                    Intent::Create => unreachable!(),
                }
            }
        };
        if let Err(e) = self.git.engine.check_refs(&prompt.refs) {
            return self.error(&e);
        }
        let sources = if matches!(action, BranchAction::Switch { .. }) {
            match self.git_branch_sources(&prompt.refs) {
                Ok(s) => Some(s),
                Err(e) => return self.error(&e),
            }
        } else {
            None
        };
        let message = match &action {
            BranchAction::Create { name, target } => format!(
                "Create local branch {} at {}? y=accept, n=cancel",
                label(name.as_bytes()),
                target.hex()
            ),
            BranchAction::Switch { name } => format!(
                "Switch to {} at {}? y=accept, n=cancel",
                label(name),
                prompt
                    .refs
                    .reference(name)
                    .expect("checked branch")
                    .object
                    .hex()
            ),
            BranchAction::Rename { name, new_name } => format!(
                "Rename {} at {} to {}? y=accept, n=cancel",
                label(name),
                prompt
                    .refs
                    .reference(name)
                    .expect("checked branch")
                    .object
                    .hex(),
                label(new_name.as_bytes())
            ),
            BranchAction::Delete { name } => format!(
                "Delete local branch {} at {} without force? y=accept, n=cancel",
                label(name),
                prompt
                    .refs
                    .reference(name)
                    .expect("checked branch")
                    .object
                    .hex()
            ),
        };
        let target = match &action {
            BranchAction::Create { target, .. } => *target,
            BranchAction::Switch { name }
            | BranchAction::Rename { name, .. }
            | BranchAction::Delete { name } => {
                prompt.refs.reference(name).expect("checked branch").object
            }
        };
        let names = match &action {
            BranchAction::Create { name, .. } => {
                format!("Create local branch:\n{}", label(name.as_bytes()))
            }
            BranchAction::Switch { name } => format!("Switch to local branch:\n{}", label(name)),
            BranchAction::Rename { name, new_name } => format!(
                "Rename local branch:\n{}\nNew name:\n{}",
                label(name),
                label(new_name.as_bytes())
            ),
            BranchAction::Delete { name } => {
                format!("Delete local branch without force:\n{}", label(name))
            }
        };
        let review = format!(
            "Local branch review\nExact target:\n{}\n{}\nChecked HEAD:\n{}\nUp/Down or PageUp/PageDown scroll. y accepts; n or C-g cancels.",
            target.hex(),
            names,
            prompt.refs.head.map_or("unborn".into(), |id| id.hex())
        );
        self.confirm_git_ref(
            prompt.refs,
            RefAction::Branch(action),
            sources,
            message,
            review,
        )
    }
    fn confirm_git_ref(
        &mut self,
        refs: Arc<Refs>,
        action: RefAction,
        sources: Option<Sources>,
        message: String,
        review: String,
    ) -> Action {
        let admission = match self
            .git
            .engine
            .source_admission(message.capacity() + 2 * review.capacity() + 16384)
        {
            Ok(a) => a,
            Err(e) => return self.error(&e),
        };
        if let Err(e) = self.input.decision("Confirm local ref action? [y/n]") {
            return self.error(&e.to_string());
        }
        self.git.painted = None;
        self.pending = Some(Pending::GitRef(Box::new(Consent {
            refs,
            action,
            sources,
            message: message.clone(),
            review,
            top: 0,
            _admission: admission,
        })));
        self.message(&message)
    }
    fn git_branch_sources(&self, refs: &Refs) -> Result<Sources, String> {
        let root = refs
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Branch switch requires a working tree")?;
        let count = self.buffers.iter().count();
        let admission = self
            .git
            .engine
            .source_admission(count * std::mem::size_of::<Stamp>())?;
        let mut stamps = Vec::with_capacity(count);
        for buffer in self.buffers.iter() {
            let related = buffer.file().is_some_and(|f| {
                f.path().starts_with(root)
                    || std::fs::canonicalize(f.path()).is_ok_and(|p| p.starts_with(root))
            });
            if related {
                if buffer.is_modified() {
                    return Err("Branch switch rejected: modified repository-backed buffer has unsaved text".into());
                }
                stamps.push(buffer.stamp());
            }
        }
        Ok(Sources {
            stamps,
            _admission: admission,
        })
    }
    pub(in crate::session) fn git_ref_answer(
        &mut self,
        consent: Consent,
        accepted: bool,
    ) -> Action {
        self.input.cancel();
        if !accepted {
            return self.message("Local ref action canceled");
        }
        if self.git.state.context != Some(consent.refs.observation.context) {
            return self.message("Branch context changed; confirmation invalidated");
        }
        if matches!(
            consent.action,
            RefAction::Branch(BranchAction::Switch { .. })
        ) {
            match self.git_branch_sources(&consent.refs) {
                Ok(s)
                    if consent
                        .sources
                        .as_ref()
                        .is_some_and(|sources| sources.stamps == s.stamps) => {}
                Ok(_) => {
                    return self.message(
                        "Repository-backed source owners changed; confirmation invalidated",
                    );
                }
                Err(e) => return self.error(&e),
            }
        }
        let result = match consent.action {
            RefAction::Branch(action) => self.git.engine.branch(consent.refs, action),
            RefAction::CreateTag(intent) => self.git.engine.create_tag(intent, ""),
            RefAction::DeleteTag(name) => self.git.engine.delete_tag(consent.refs, &name),
        };
        match result {
            Ok(id) => {
                self.git.refs.request = Some(id);
                self.message("Checking confirmed local ref request")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn publish_git_refs(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            self.git.refs.failed = true;
            return;
        }
        self.git.refs.request = None;
        self.git.refs.failed = !update.published
            || self
                .git
                .engine
                .record(update.operation)
                .is_none_or(|r| r.outcome != crate::git::Outcome::Verified);
        if update.published {
            self.git.state.context = update.context;
            let selected = self.git.refs().and_then(|r| {
                self.git
                    .refs
                    .selected
                    .as_ref()
                    .filter(|n| r.reference(n).is_some())
                    .cloned()
                    .or_else(|| r.entries.first().map(|r| r.name.clone()))
            });
            self.git.refs.selected = selected;
            self.move_git_refs(0, false);
        }
        self.observe_files();
        self.git.painted = None;
        if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing {
            if self.git.refs.failed
                && self.git.engine.record(update.operation).is_some_and(|r| {
                    matches!(
                        r.kind,
                        crate::git::OperationKind::TagCreate | crate::git::OperationKind::TagDelete
                    )
                })
            {
                let record = self
                    .git
                    .engine
                    .record(update.operation)
                    .expect("tag record");
                let diagnostic = crate::display::git::DiagnosticLines::new(&record.stderr)
                    .next()
                    .unwrap_or_else(|| {
                        record
                            .diagnostic
                            .clone()
                            .unwrap_or("inspect git-processes".into())
                    });
                self.message(&format!(
                    "Tag operation failed; annotation retained: {diagnostic}"
                ));
                return;
            }
            self.message(if self.git.refs.failed {
                "Refs operation failed; confirmed facts retained; inspect git-processes"
            } else {
                "Refs and Status checked; source reload follows file policy"
            });
        }
    }
}

impl Session {
    pub(super) fn git_ref_review_input(&mut self, event: &Event) -> Option<Action> {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        let extent = usize::from(self.size.height.saturating_sub(2).max(1));
        let Some(Pending::GitRef(consent)) = &mut self.pending else {
            return None;
        };
        let movement = match event {
            Event::Key(k) if k.kind != KeyEventKind::Release => match (k.code, k.modifiers) {
                (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    Some((1, true))
                }
                (KeyCode::Down, KeyModifiers::NONE)
                | (KeyCode::Char('n'), KeyModifiers::CONTROL) => Some((1, false)),
                (KeyCode::PageUp, KeyModifiers::NONE) => Some((extent, true)),
                (KeyCode::PageDown, KeyModifiers::NONE) => Some((extent, false)),
                _ => None,
            },
            Event::Mouse(m) if self.mouse_enabled => match m.kind {
                MouseEventKind::ScrollUp => Some((3, true)),
                MouseEventKind::ScrollDown => Some((3, false)),
                _ => None,
            },
            _ => None,
        }?;
        let width = usize::from(self.size.width.max(1));
        let count = consent
            .review
            .lines()
            .flat_map(|line| Wrapped {
                text: line.into(),
                offset: Some(0),
                width,
            })
            .count();
        consent.top = if movement.1 {
            consent.top.saturating_sub(movement.0)
        } else {
            consent
                .top
                .saturating_add(movement.0)
                .min(count.saturating_sub(1))
        };
        Some(Action::Redraw)
    }
    pub(super) fn git_ref_review_frame(&mut self) -> Result<Frame, crate::view::Error> {
        let Some(Pending::GitRef(consent)) = &self.pending else {
            unreachable!("branch review");
        };
        let width = usize::from(self.size.width.max(1));
        let mut rows = consent
            .review
            .lines()
            .flat_map(|line| Wrapped {
                text: line.into(),
                offset: Some(0),
                width,
            })
            .skip(consent.top);
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let mut frame = Frame::git(
            self.buffer()?,
            self.size,
            self.buffers.palette(),
            &self.echo,
            " Git Local Ref Review  y accept  n / C-g cancel  Up/Down scroll",
            admission,
            |_| (rows.next().unwrap_or_default(), false),
        )?;
        self.prompt_frame(&mut frame, None)?;
        Ok(frame)
    }
}
