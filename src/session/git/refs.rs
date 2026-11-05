use super::super::Pending;
pub(in crate::session) mod bisect;
mod integration;
pub(in crate::session) mod sequences;
pub(in crate::session) mod tag;
mod transport;
use super::*;
use crate::{
    buffer::Stamp,
    git::{BranchAction, Completeness, RefKind, Refs, Update},
};

pub(super) struct Inspection {
    pub request: Option<OperationId>,
    pub tag_preparing: Option<OperationId>,
    pub push_preparing: Option<OperationId>,
    pub integration_request: Option<OperationId>,
    pub autosquash_preparing: Option<OperationId>,
    pub application_preparing: Option<OperationId>,
    pub bisect_preparing: Option<(OperationId, Option<crate::git::BisectAction>)>,
    control_preparing: Option<(
        crate::git::SequenceControl,
        Option<crate::git::SequenceFamily>,
    )>,
    pub(super) sequence_top: usize,
    push_mark: Option<transport::PushMark>,
    pub(super) selected: Option<Vec<u8>>,
    remote_selected: Option<Vec<u8>>,
    detail: Option<usize>,
    pub top: usize,
    pub(super) failed: bool,
    _admission: crate::git::budget::Lease,
}
impl Inspection {
    pub(super) fn new(engine: &Repositories) -> std::io::Result<Self> {
        Ok(Self {
            request: None,
            tag_preparing: None,
            push_preparing: None,
            integration_request: None,
            autosquash_preparing: None,
            application_preparing: None,
            bisect_preparing: None,
            control_preparing: None,
            sequence_top: 0,
            push_mark: None,
            selected: None,
            remote_selected: None,
            detail: None,
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
    submit_prefix: bool,
    _admission: crate::git::budget::Lease,
}
pub(super) enum RefAction {
    Worktree(Arc<crate::git::WorktreePlan>),
    Stash(Arc<crate::git::StashPlan>),
    Resolution(Arc<crate::git::Resolution>),
    Branch(BranchAction),
    CreateTag(Arc<crate::git::TagIntent>),
    DeleteTag(Vec<u8>),
    Push(Arc<crate::git::PushIntent>),
    Autosquash(Arc<crate::git::AutosquashPlan>),
    Application(Arc<crate::git::ApplyPlan>),
    BisectStart(Arc<crate::git::BisectPlan>),
    BisectControl(Arc<crate::git::Sequence>, crate::git::BisectAction),
    Control {
        sequence: Arc<crate::git::Sequence>,
        action: crate::git::SequenceControl,
        family: Option<crate::git::SequenceFamily>,
    },
    Integration {
        target: Vec<u8>,
        mode: crate::git::IntegrationMode,
    },
}

pub(super) struct Sources {
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
    pub(super) fn refs_row(&self, row: usize, width: usize) -> (String, bool) {
        if let Some(top) = self.refs.detail {
            return (
                self.ref_detail_lines(width)
                    .nth(top + row)
                    .unwrap_or_default(),
                false,
            );
        }

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
                if self.refs.request.is_some()
                    || self.refs.tag_preparing.is_some()
                    || self.refs.push_preparing.is_some()
                    || self.ancestry.request.is_some()
                {
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
                    "{}{} {} {}{}{}{}{}",
                    if selected { ">" } else { " " },
                    if self
                        .refs
                        .push_mark
                        .as_ref()
                        .is_some_and(|mark| mark.context == refs.observation.context
                            && mark.name == r.name
                            && mark.object == r.object)
                    {
                        "*"
                    } else {
                        " "
                    },
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
        refs.remotes
            .get(index.saturating_sub(refs.entries.len()))
            .map_or((String::new(), false), |r| {
                let selected = self.refs.remote_selected.as_ref() == Some(r);
                (
                    format!("{} remote {}", if selected { ">" } else { " " }, label(r)),
                    selected,
                )
            })
    }
    fn ref_detail_lines(&self, width: usize) -> impl Iterator<Item = String> + '_ {
        (0..19)
            .map(move |row| {
                let Some(refs) = self.refs() else {
                    return String::new();
                };
                let reference = self.refs.selected.as_ref().and_then(|n| refs.reference(n));
                match row {
                    0 => "Git Ref Detail".into(),
                    1 => reference.map_or("Configured remote name".into(), |r| {
                        format!("Kind: {:?}", r.kind)
                    }),
                    2 => "Full exact name:".into(),
                    3 => reference
                        .map(|r| label(&r.name))
                        .or_else(|| self.refs.remote_selected.as_ref().map(|n| label(n)))
                        .unwrap_or_default(),
                    4 => "Full ref object:".into(),
                    5 => reference.map_or("unavailable for configured remote".into(), |r| {
                        r.object.hex()
                    }),
                    6 => reference.map_or(String::new(), |r| {
                        format!("Object type: {}", label(&r.object_type))
                    }),
                    7 => "Symbolic target:".into(),
                    8 => reference.map_or(String::new(), |r| label(&r.symbolic)),
                    9 => "Peeled target:".into(),
                    10 => reference
                        .and_then(|r| r.peeled)
                        .map_or("direct object / unavailable".into(), |id| id.hex()),
                    11 => reference.map_or(String::new(), |r| {
                        format!("Peeled type: {}", label(&r.peeled_type))
                    }),
                    12 => "Full upstream ref:".into(),
                    13 => reference.map_or(String::new(), |r| label(&r.upstream)),
                    14 => reference.map_or(String::new(), |r| {
                        format!(
                            "Tracking: {}",
                            if r.tracking.is_empty() {
                                "unavailable or equal".into()
                            } else {
                                label(&r.tracking)
                            }
                        )
                    }),
                    15 => format!(
                        "HEAD ref: {}",
                        if refs.head_ref.is_empty() {
                            "detached".into()
                        } else {
                            label(&refs.head_ref)
                        }
                    ),
                    16 => refs.head.map_or("HEAD unborn".into(), |id| id.hex()),
                    17 => format!("Observation: {:?}", refs.completeness),
                    _ => "Up/Down scroll; Tab or q returns to Refs".into(),
                }
            })
            .flat_map(move |text| Wrapped {
                text,
                offset: Some(0),
                width: width.max(1),
            })
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
                "= / i / I  confirmed fast-forward / normal / required merge",
                "e  ordinary rebase onto one exact selected ref",
                "* / -  mark / clear one exact push source branch",
                "p / M p  git-push: review one configured upstream branch",
                "& / M f  git-fetch: selected configured remote, without integration",
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
            Some(
                Pending::GitRef(_)
                    | Pending::GitBranchName(_)
                    | Pending::GitTagPrompt(_)
                    | Pending::GitWorktree(_)
                    | Pending::GitRepository(_)
            )
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
    pub(super) fn fetch_git_remote(&mut self) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Open current Refs and select a configured remote before fetch");
        };
        let Some(remote) = self.git.refs.remote_selected.clone() else {
            return self.message("Select one exact configured remote row before fetch");
        };
        match self.git.engine.fetch(refs, &remote) {
            Ok(id) => {
                self.git.refs.request = Some(id);
                self.git.painted = None;
                self.message("Fetching the selected configured remote; integration remains a separate choice")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(super) fn select_git_ref(&mut self, index: usize) -> Action {
        if self.git.refs.detail.is_some() {
            return Action::Wait;
        }
        let Some(refs) = self.git.refs() else {
            return Action::Wait;
        };
        if let Some(reference) = refs.entries.get(index) {
            let name = reference.name.clone();
            self.git.refs.selected = Some(name);
            self.git.refs.remote_selected = None;
        } else if let Some(remote) = refs.remotes.get(index.saturating_sub(refs.entries.len())) {
            let name = remote.clone();
            self.git.refs.selected = None;
            self.git.refs.remote_selected = Some(name);
        } else {
            return Action::Wait;
        }
        self.git.refs.top = self
            .git
            .refs
            .top
            .min(index)
            .max(index.saturating_sub(self.git_extent().saturating_sub(1)));
        Action::Redraw
    }
    pub(super) fn toggle_git_ref_detail(&mut self) -> Action {
        self.git.refs.detail = if self.git.refs.detail.is_some() {
            None
        } else {
            Some(0)
        };
        self.git.painted = None;
        Action::Redraw
    }
    pub(super) fn quit_git_ref_detail(&mut self) -> bool {
        if self.git.page == Page::Refs && self.git.refs.detail.take().is_some() {
            self.git.painted = None;
            true
        } else {
            false
        }
    }
    pub(super) fn move_git_refs(&mut self, distance: usize, backward: bool) -> Action {
        if let Some(top) = self.git.refs.detail {
            let count = self
                .git
                .ref_detail_lines(usize::from(self.size.width.max(1)))
                .count();
            self.git.refs.detail = Some(if backward {
                top.saturating_sub(distance)
            } else {
                top.saturating_add(distance).min(count.saturating_sub(1))
            });
            self.git.painted = None;
            return Action::Redraw;
        }
        let Some(refs) = self.git.refs() else {
            return Action::Wait;
        };
        let index = refs
            .entries
            .iter()
            .position(|r| Some(&r.name) == self.git.refs.selected.as_ref())
            .or_else(|| {
                refs.remotes
                    .iter()
                    .position(|n| Some(n) == self.git.refs.remote_selected.as_ref())
                    .map(|i| i + refs.entries.len())
            })
            .unwrap_or(0);
        let next = if backward {
            index.saturating_sub(distance)
        } else {
            index
                .saturating_add(distance)
                .min((refs.entries.len() + refs.remotes.len()).saturating_sub(1))
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
    pub(super) fn confirm_git_ref(
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
        let prompt = if matches!(action, RefAction::Worktree(_)) {
            "Confirm checked worktree action? [y/n]"
        } else if matches!(action, RefAction::Stash(_)) {
            "Confirm checked stash action? [y/n]"
        } else if matches!(action, RefAction::Resolution(_)) {
            "Confirm exact conflict resolution? [y/n]"
        } else if matches!(action, RefAction::Push(_)) {
            "Confirm reviewed upstream push? [y/n]"
        } else {
            if matches!(
                action,
                RefAction::Integration { .. }
                    | RefAction::Autosquash(_)
                    | RefAction::Application(_)
                    | RefAction::BisectStart(_)
                    | RefAction::BisectControl(..)
                    | RefAction::Control { .. }
            ) {
                "Confirm checked integration? [y/n]"
            } else {
                "Confirm local ref action? [y/n]"
            }
        };
        if let Err(e) = self.input.decision(prompt) {
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
            submit_prefix: false,
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
    pub(super) fn git_worktree_sources(
        &self,
        plan: &crate::git::WorktreePlan,
    ) -> Result<Sources, String> {
        let admission = self.git.engine.source_admission(256)?;
        if matches!(
            plan.action,
            crate::git::WorktreeAction::Lock { .. } | crate::git::WorktreeAction::Unlock(_)
        ) {
            return Ok(Sources {
                stamps: Vec::new(),
                _admission: admission,
            });
        }
        for root in &plan.paths {
            for buffer in self.buffers.iter() {
                let related = buffer.file().is_some_and(|f| {
                    f.path().starts_with(root)
                        || std::fs::canonicalize(f.path()).is_ok_and(|p| p.starts_with(root))
                });
                if related {
                    return Err(if buffer.is_modified() {
                        "Worktree is protected by an unsaved visiting buffer"
                    } else {
                        "Worktree is protected by a retained visiting buffer"
                    }
                    .into());
                }
            }
            if self.tree.chosen_root().is_some_and(|p| p.starts_with(root)) {
                return Err("Worktree is protected by the native file-tree root".into());
            }
            if self.terminals.retains_directory(root)
                || self.shell.retains_directory(root)
                || self.ai.retains_directory(root)
                || self.lsp.retains_directory(root)
            {
                return Err(
                    "Worktree is protected by a retained native process or language-server owner"
                        .into(),
                );
            }
        }
        Ok(Sources {
            stamps: Vec::new(),
            _admission: admission,
        })
    }
    pub(super) fn git_stash_sources(
        &self,
        plan: &crate::git::StashPlan,
    ) -> Result<Sources, String> {
        let root = plan
            .source
            .refs
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Stash worktree unavailable")?;
        let count = self.buffers.iter().count();
        let admission = self
            .git
            .engine
            .source_admission(count * std::mem::size_of::<Stamp>())?;
        let mut stamps = Vec::with_capacity(count);
        for buffer in self.buffers.iter() {
            let related = buffer.file().is_some_and(|f| {
                plan.paths.iter().any(|p| {
                    let target = root.join(p);
                    f.path() == target
                        || std::fs::canonicalize(&target)
                            .is_ok_and(|p| std::fs::canonicalize(f.path()).is_ok_and(|q| p == q))
                })
            });
            if related {
                if buffer.is_modified() {
                    return Err(
                        "Stash action rejected: affected visiting buffer has unsaved text".into(),
                    );
                }
                stamps.push(buffer.stamp());
            }
        }
        Ok(Sources {
            stamps,
            _admission: admission,
        })
    }
    pub(super) fn git_resolution_sources(
        &self,
        conflict: &crate::git::Conflict,
    ) -> Result<Sources, String> {
        let root = conflict
            .sequence
            .refs
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Conflict has no worktree source owner")?;
        let target = root.join(&conflict.change.path);
        let canonical = std::fs::canonicalize(&target).ok();
        let count = self.buffers.iter().count();
        let admission = self
            .git
            .engine
            .source_admission(count * std::mem::size_of::<Stamp>())?;
        let mut stamps = Vec::with_capacity(count);
        for buffer in self.buffers.iter() {
            let related = buffer.file().is_some_and(|f| {
                f.path() == target
                    || canonical.as_ref().is_some_and(|target| {
                        std::fs::canonicalize(f.path()).is_ok_and(|p| p == *target)
                    })
            });
            if related {
                if buffer.is_modified() {
                    return Err(
                        "Conflict resolution rejected: the visiting result buffer has unsaved text"
                            .into(),
                    );
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
            return self.message(if matches!(consent.action, RefAction::Worktree(_)) {
                "Worktree action canceled"
            } else if matches!(consent.action, RefAction::Stash(_)) {
                "Stash action canceled"
            } else if matches!(consent.action, RefAction::Resolution(_)) {
                "Conflict resolution canceled"
            } else {
                "Local ref action canceled"
            });
        }
        if self.git.state.context != Some(consent.refs.observation.context) {
            return self.message("Branch context changed; confirmation invalidated");
        }
        if matches!(
            consent.action,
            RefAction::Worktree(_)
                | RefAction::Stash(_)
                | RefAction::Branch(BranchAction::Switch { .. })
                | RefAction::Resolution(_)
                | RefAction::Integration { .. }
                | RefAction::Autosquash(_)
                | RefAction::Application(_)
                | RefAction::BisectStart(_)
                | RefAction::BisectControl(..)
                | RefAction::Control { .. }
        ) {
            let sources = match &consent.action {
                RefAction::Worktree(plan) => self.git_worktree_sources(plan),
                RefAction::Stash(plan) => self.git_stash_sources(plan),
                RefAction::Resolution(plan) => self.git_resolution_sources(&plan.conflict),
                _ => self.git_branch_sources(&consent.refs),
            };
            match sources {
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
        let worktree = matches!(consent.action, RefAction::Worktree(_));
        let stash = matches!(consent.action, RefAction::Stash(_));
        let resolution = matches!(consent.action, RefAction::Resolution(_));
        let integration = matches!(
            consent.action,
            RefAction::Integration { .. }
                | RefAction::Autosquash(_)
                | RefAction::Application(_)
                | RefAction::BisectStart(_)
                | RefAction::BisectControl(..)
                | RefAction::Control { .. }
        );
        let result = match consent.action {
            RefAction::Worktree(plan) => self.git.engine.change_worktree(plan),
            RefAction::Stash(plan) => self.git.engine.change_stash(plan),
            RefAction::Resolution(plan) => self.git.engine.resolve_conflict(plan),
            RefAction::Branch(action) => self.git.engine.branch(consent.refs, action),
            RefAction::CreateTag(intent) => self.git.engine.create_tag(intent, ""),
            RefAction::DeleteTag(name) => self.git.engine.delete_tag(consent.refs, &name),
            RefAction::Push(intent) => self.git.engine.push(intent),
            RefAction::BisectStart(plan) => self.git.engine.start_bisect(plan),
            RefAction::BisectControl(sequence, action) => {
                self.git.engine.classify_bisect(sequence, action)
            }
            RefAction::Application(plan) => self.git.engine.apply_sequence(plan),
            RefAction::Autosquash(plan) => self.git.engine.autosquash(plan),
            RefAction::Control {
                sequence,
                action,
                family,
            } => self.git.engine.control_sequence(sequence, action, family),
            RefAction::Integration { target, mode } => {
                self.git.engine.integrate(consent.refs, &target, mode)
            }
        };
        match result {
            Ok(id) => {
                if worktree {
                    self.git.worktrees.request = Some(id);
                } else if stash {
                    self.git.repository.request = Some(id);
                } else if resolution {
                    self.git.conflicts.resolving = Some(id);
                } else if integration {
                    self.git.refs.integration_request = Some(id);
                } else {
                    self.git.refs.request = Some(id);
                }
                self.message(if worktree {
                    "Checking confirmed worktree paths, identities and source owners"
                } else if stash {
                    "Checking confirmed stash object and saved sources"
                } else if resolution {
                    "Checking the confirmed conflict path, saved result and exact stage fingerprint"
                } else if integration {
                    "Checking confirmed integration policy and exact ref target"
                } else {
                    "Checking confirmed local ref request"
                })
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
            if self
                .git
                .refs
                .remote_selected
                .as_ref()
                .is_none_or(|name| self.git.refs().is_none_or(|r| !r.remotes.contains(name)))
            {
                self.git.refs.remote_selected = None;
            }
            if self.git.refs.selected.is_some() && selected != self.git.refs.selected {
                self.git.refs.detail = None;
            }
            self.git.refs.selected = if self.git.refs.remote_selected.is_some() {
                None
            } else {
                selected
            };
            if self.git.refs.detail.is_none() {
                self.move_git_refs(0, false);
            }
        }
        self.observe_files();
        self.git.painted = None;
        if update.published
            && let Some(history) = self.git.history().cloned()
            && let Some(observation) = update
                .context
                .and_then(|id| self.git.engine.context(id))
                .map(|c| c.observation.clone())
        {
            let preserve_root = self.git.engine.record(update.operation).is_some_and(|r| {
                matches!(
                    r.kind,
                    crate::git::OperationKind::Application
                        | crate::git::OperationKind::ApplicationControl
                        | crate::git::OperationKind::Bisect
                )
            });
            match self
                .git
                .engine
                .refresh_history_at(history, observation, preserve_root)
            {
                Ok(id) => {
                    let page = if update
                        .context
                        .and_then(|id| self.git.engine.context(id))
                        .and_then(|c| c.sequence.as_ref())
                        .is_some_and(|s| s.family.is_some())
                    {
                        Page::Sequence
                    } else {
                        Page::History
                    };
                    self.git.ancestry.request = Some((id, page));
                }
                Err(_) => self.git.ancestry.failed = true,
            }
        }

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
            if self
                .git
                .engine
                .record(update.operation)
                .is_some_and(|r| r.kind == crate::git::OperationKind::Fetch)
            {
                self.message(if self.git.refs.failed {
                    "Fetch failed or tracking effects remain uncertain; inspect git-processes; raw transport output withheld"
                } else {
                    "Fetch completed; Refs and Status checked; branch not integrated"
                });
                return;
            }
            if self
                .git
                .engine
                .record(update.operation)
                .is_some_and(|r| r.kind == crate::git::OperationKind::Push)
            {
                self.message(if self.git.refs.failed {
                    "Push failed or publication remains unresolved; inspect git-processes; no automatic retry"
                } else {
                    "Push verified: exact reviewed object observed at its configured upstream destination"
                });
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
        let accept = if let Some(Pending::GitRef(consent)) = &mut self.pending {
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press
                    && key.modifiers == KeyModifiers::CONTROL
                    && key.code == KeyCode::Enter
                {
                    true
                } else if key.kind == KeyEventKind::Press
                    && key.modifiers == KeyModifiers::CONTROL
                    && key.code == KeyCode::Char('c')
                {
                    if !consent.submit_prefix {
                        consent.submit_prefix = true;
                        return Some(Action::Redraw);
                    }
                    true
                } else {
                    if key.kind == KeyEventKind::Press {
                        consent.submit_prefix = false;
                    }
                    false
                }
            } else {
                false
            }
        } else {
            false
        };
        if accept {
            let Some(Pending::GitRef(consent)) = self.pending.take() else {
                unreachable!("checked Git review");
            };
            return Some(self.git_ref_answer(*consent, true));
        }
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
            if matches!(consent.action, RefAction::Resolution(_)) {
                " Git Conflict Review  y / C-Enter / C-c C-c accept  n / C-g cancel"
            } else if matches!(
                consent.action,
                RefAction::Integration { .. }
                    | RefAction::Autosquash(_)
                    | RefAction::Application(_)
                    | RefAction::BisectStart(_)
                    | RefAction::BisectControl(..)
                    | RefAction::Control { .. }
            ) {
                " Git Integration Review  y / C-Enter / C-c C-c accept  C-g cancel"
            } else if matches!(consent.action, RefAction::Push(_)) {
                " Git Push Review  y accept  n / C-g cancel  Up/Down scroll"
            } else {
                " Git Local Ref Review  y accept  n / C-g cancel  Up/Down scroll"
            },
            admission,
            |_| (rows.next().unwrap_or_default(), false),
        )?;
        self.prompt_frame(&mut frame, None)?;
        Ok(frame)
    }
}
