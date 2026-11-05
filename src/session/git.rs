use super::{Action, Session};
use crate::{
    command::{Command, GitCommand, Motion, Scroll},
    display::{Frame, git::label},
    git::{ContextId, Group, Head, Observation, OperationId, PathChange, Repositories, Status},
    input::Focus,
};
use crossterm::event::{Event, MouseButton, MouseEventKind};
use std::{path::PathBuf, sync::Arc};

mod commit;
mod comparisons;
mod conflicts;
pub(super) mod discard;
mod families;
mod history;
mod index;
mod lines;
use lines::Wrapped;
mod processes;
pub(super) mod refs;
pub(in crate::session) mod repository;
mod shutdown;

pub(super) struct Work {
    pub engine: Repositories,
    launch: PathBuf,
    visible: bool,
    state: State,
    painted: Option<Painted>,
    page: Page,
    process_origin: Option<Page>,
    inspection: comparisons::Inspection,
    index_request: Option<OperationId>,
    discard_request: Option<OperationId>,
    family_request: Option<OperationId>,
    family_preparing: Option<OperationId>,
    commit_request: Option<OperationId>,
    commit_preparing: Option<OperationId>,
    composer: Option<commit::Composer>,
    index_preparing: Option<index::Preparing>,
    marks: index::Marks,
    ancestry: history::Inspection,
    refs: refs::Inspection,
    conflicts: conflicts::Inspection,
    repository: repository::Inspection,
    worktrees: repository::worktrees::Inspection,
    process_selected: Option<OperationId>,
    process_top: usize,
    detail_top: usize,
    pub(super) exit_operations: Vec<OperationId>,
    pub(super) shutdown: Option<super::quit::Sources>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Status,
    History,
    Refs,
    Sequence,
    Conflict,
    Stash,
    Worktree,
    Revision,
    Reflog,
    Blame,
    RevisionFile,
    Diff,
    Processes,
    Process(OperationId),
}

struct Painted {
    context: Option<ContextId>,
    generation: Option<u64>,
    top: usize,
    collapsed: [bool; 4],
    size: crate::display::Size,
    focus: Focus,
    menu: bool,
    page: Page,
    processes: Vec<OperationId>,
    process_top: usize,
    comparison: Option<OperationId>,
    comparison_top: usize,
    history: Option<OperationId>,
    revision: Option<OperationId>,
    history_top: usize,
    revision_top: usize,
    inspected: (Option<OperationId>, usize),
    refs: Option<OperationId>,
    refs_top: usize,
    conflict: (Option<OperationId>, usize),
    stash: (Option<OperationId>, usize),
    worktree: (Option<OperationId>, usize),
    sequence_top: usize,
    expanded: bool,
    expanded_hunk: Option<crate::git::HunkId>,
}

#[derive(Default)]
struct State {
    context: Option<ContextId>,
    request: Option<(OperationId, bool)>,
    selected: Option<Target>,
    collapsed: [bool; 4],
    top: usize,
    menu: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum Target {
    Section(Group),
    Path(Group, PathBuf),
    Hunk(Group, PathBuf, crate::git::HunkId),
}

enum Item<'a> {
    Section(Group, usize),
    Path(Group, &'a PathChange),
    Detail(&'a crate::git::Diff, &'a crate::git::DiffLine),
}

impl Item<'_> {
    fn target(&self) -> Target {
        match self {
            Self::Section(group, _) => Target::Section(*group),
            Self::Path(group, path) => Target::Path(*group, path.path.clone()),
            Self::Detail(diff, line) => line.hunk.map_or_else(
                || Target::Path(diff.group, diff.change.path.clone()),
                |id| Target::Hunk(diff.group, diff.change.path.clone(), id),
            ),
        }
    }
    fn matches(&self, target: &Target) -> bool {
        match (self, target) {
            (Self::Section(group, _), Target::Section(other)) => group == other,
            (Self::Path(group, path), Target::Path(other, target)) => {
                group == other && path.path == *target
            }
            (Self::Detail(diff, line), Target::Hunk(group, path, id)) => {
                diff.group == *group
                    && diff.change.path == *path
                    && line.hunk == Some(*id)
                    && diff
                        .hunk(*id)
                        .is_some_and(|h| h.patch.start == line.bytes.start)
            }
            _ => false,
        }
    }
    fn selectable(&self) -> bool {
        match self {
            Self::Detail(diff, line) => line
                .hunk
                .and_then(|id| diff.hunk(id))
                .is_some_and(|h| h.patch.start == line.bytes.start),
            _ => true,
        }
    }
}

fn items<'a>(
    status: &'a Status,
    collapsed: &'a [bool; 4],
    detail: Option<&'a crate::git::Diff>,
    expanded_hunk: Option<crate::git::HunkId>,
) -> impl Iterator<Item = Item<'a>> + 'a {
    Group::ALL
        .into_iter()
        .enumerate()
        .flat_map(move |(index, group)| {
            let count = status.group(group).count();
            std::iter::once(Item::Section(group, count))
                .filter(move |_| count != 0)
                .chain(
                    status
                        .group(group)
                        .filter(move |_| !collapsed[index])
                        .flat_map(move |p| {
                            std::iter::once(Item::Path(group, p)).chain(
                                detail
                                    .filter(|d| d.group == group && d.change.path == p.path)
                                    .into_iter()
                                    .flat_map(move |d| {
                                        d.lines
                                            .iter()
                                            .filter(move |line| {
                                                line.hunk.is_none()
                                                    || line.hunk == expanded_hunk
                                                    || line
                                                        .hunk
                                                        .and_then(|id| d.hunk(id))
                                                        .is_some_and(|h| {
                                                            h.patch.start == line.bytes.start
                                                        })
                                            })
                                            .map(move |line| Item::Detail(d, line))
                                    }),
                            )
                        }),
                )
        })
}

impl Work {
    pub fn new(
        snapshot: &crate::config::Snapshot,
        wake: Arc<crate::file::watch::Wake>,
    ) -> std::io::Result<Self> {
        let engine = Repositories::new(snapshot, Arc::new(move || wake.signal()));
        let ancestry = history::Inspection::new(&engine)?;
        let refs = refs::Inspection::new(&engine)?;
        let conflicts = conflicts::Inspection::new(&engine)?;
        Ok(Self {
            engine,
            launch: std::env::current_dir()?,
            visible: false,
            state: State::default(),
            painted: None,
            page: Page::Status,
            process_origin: None,
            inspection: comparisons::Inspection::default(),
            index_request: None,
            discard_request: None,
            family_request: None,
            family_preparing: None,
            commit_request: None,
            commit_preparing: None,
            composer: None,
            index_preparing: None,
            marks: index::Marks::default(),
            ancestry,
            refs,
            conflicts,
            repository: repository::Inspection::new(),
            worktrees: repository::worktrees::Inspection::new(),
            process_selected: None,
            process_top: 0,
            detail_top: 0,
            exit_operations: Vec::new(),
            shutdown: None,
        })
    }
    fn observation(&self) -> Option<&Observation> {
        if self.state.request.is_some_and(|(_, opening)| opening) {
            return None;
        }
        self.state
            .context
            .and_then(|id| self.engine.context(id))
            .map(|c| c.observation.as_ref())
    }
    fn status(&self) -> Option<&Status> {
        self.observation()?.status.as_ref()
    }
    fn items(&self) -> impl Iterator<Item = Item<'_>> {
        self.status().into_iter().flat_map(|s| {
            items(
                s,
                &self.state.collapsed,
                self.inspection
                    .detail
                    .as_deref()
                    .filter(|_| self.inspection.expanded),
                self.inspection.expanded_hunk,
            )
        })
    }
    fn reconcile(&mut self, extent: usize) {
        let Some(status) = self.status() else {
            return;
        };
        let selected = self
            .state
            .selected
            .as_ref()
            .and_then(|target| self.items().position(|i| i.matches(target)));
        let index = selected.unwrap_or(0);
        if selected.is_none() {
            let replacement = self
                .state
                .selected
                .as_ref()
                .and_then(|target| match target {
                    Target::Hunk(group, path, _)
                        if status.group(*group).any(|p| p.path == *path) =>
                    {
                        Some(Target::Path(*group, path.clone()))
                    }
                    _ => None,
                })
                .or_else(|| self.items().next().map(|i| i.target()));
            self.state.selected = replacement;
        }
        self.state.top = self
            .state
            .top
            .min(index)
            .max(index.saturating_sub(extent.saturating_sub(1)));
    }
}

impl Session {
    pub fn git_visible(&self) -> bool {
        self.git.visible
    }
    pub fn git_observation(&self) -> Option<&Observation> {
        self.git.observation()
    }

    pub(super) fn git_command(&mut self, command: Command) -> Option<Action> {
        if let Some(action) = self.commit_command(command) {
            return Some(action);
        }
        if let Command::Git(command) = command {
            if self.input.focus() != Focus::Editing {
                return Some(self.message("A prompt owns input focus"));
            }
            return Some(match command {
                GitCommand::Merge => {
                    self.confirm_git_integration(crate::git::IntegrationMode::Merge)
                }
                GitCommand::MergeFastForward => {
                    self.confirm_git_integration(crate::git::IntegrationMode::FastForward)
                }
                GitCommand::MergeNoFastForward => {
                    self.confirm_git_integration(crate::git::IntegrationMode::NoFastForward)
                }
                GitCommand::Rebase => {
                    self.confirm_git_integration(crate::git::IntegrationMode::Rebase)
                }
                GitCommand::BisectStart => self.prompt_git_bisect(),
                GitCommand::BisectGood => self.request_git_bisect(crate::git::BisectAction::Good),
                GitCommand::BisectBad => self.request_git_bisect(crate::git::BisectAction::Bad),
                GitCommand::BisectSkip => self.request_git_bisect(crate::git::BisectAction::Skip),
                GitCommand::BisectReset => self.request_git_bisect(crate::git::BisectAction::Reset),
                GitCommand::ConflictUseBase => self.prepare_git_resolution(
                    crate::git::ResolutionAction::Use(crate::git::ConflictRole::Base),
                ),
                GitCommand::ConflictUseCurrent => self.prepare_git_resolution(
                    crate::git::ResolutionAction::Use(crate::git::ConflictRole::Current),
                ),
                GitCommand::ConflictUseApplied => self.prepare_git_resolution(
                    crate::git::ResolutionAction::Use(crate::git::ConflictRole::Applied),
                ),
                GitCommand::ConflictDelete => {
                    self.prepare_git_resolution(crate::git::ResolutionAction::Delete)
                }
                GitCommand::ConflictStage => {
                    self.prepare_git_resolution(crate::git::ResolutionAction::Stage)
                }
                GitCommand::WorktreeList => self.open_git_worktrees(),
                GitCommand::WorktreeVisit => self.visit_git_worktree(),
                GitCommand::WorktreeCreate => {
                    self.prompt_git_worktree(repository::worktrees::Intent::Create)
                }
                GitCommand::WorktreeLock => {
                    self.prompt_git_worktree(repository::worktrees::Intent::Lock)
                }
                GitCommand::WorktreeUnlock => {
                    self.prompt_git_worktree(repository::worktrees::Intent::Unlock)
                }
                GitCommand::WorktreePrune => {
                    self.prompt_git_worktree(repository::worktrees::Intent::Prune)
                }
                GitCommand::WorktreeRemove => {
                    self.prompt_git_worktree(repository::worktrees::Intent::Remove)
                }
                GitCommand::StashList => self.open_git_stashes(),
                GitCommand::StashShow => self.show_git_stash(),
                GitCommand::StashCreate => self.prompt_git_stash(repository::Intent::Create),
                GitCommand::StashApply => self.prompt_git_stash(repository::Intent::Apply),
                GitCommand::StashPop => self.prompt_git_stash(repository::Intent::Pop),
                GitCommand::StashBranch => self.prompt_git_stash(repository::Intent::Branch),
                GitCommand::StashDrop => self.prompt_git_stash(repository::Intent::Drop),
                GitCommand::Conflicts => self.open_git_sequence(),
                GitCommand::ConflictExplain => self.explain_git_conflict(),
                GitCommand::ConflictVisit => self.open_git_conflict(),
                GitCommand::OperationStatus => self.open_git_sequence(),
                GitCommand::RebaseAutosquash => self.prepare_git_autosquash(),
                GitCommand::MergeContinue => self.request_git_control(
                    crate::git::SequenceControl::Continue,
                    Some(crate::git::SequenceFamily::Merge),
                ),
                GitCommand::MergeAbort => self.request_git_control(
                    crate::git::SequenceControl::Abort,
                    Some(crate::git::SequenceFamily::Merge),
                ),
                GitCommand::RebaseContinue => self.request_git_control(
                    crate::git::SequenceControl::Continue,
                    Some(crate::git::SequenceFamily::Rebase),
                ),
                GitCommand::RebaseSkip => self.request_git_control(
                    crate::git::SequenceControl::Skip,
                    Some(crate::git::SequenceFamily::Rebase),
                ),
                GitCommand::RebaseAbort => self.request_git_control(
                    crate::git::SequenceControl::Abort,
                    Some(crate::git::SequenceFamily::Rebase),
                ),
                GitCommand::CherryPickContinue => self.request_git_control(
                    crate::git::SequenceControl::Continue,
                    Some(crate::git::SequenceFamily::CherryPick),
                ),
                GitCommand::CherryPickSkip => self.request_git_control(
                    crate::git::SequenceControl::Skip,
                    Some(crate::git::SequenceFamily::CherryPick),
                ),
                GitCommand::CherryPickAbort => self.request_git_control(
                    crate::git::SequenceControl::Abort,
                    Some(crate::git::SequenceFamily::CherryPick),
                ),
                GitCommand::RevertContinue => self.request_git_control(
                    crate::git::SequenceControl::Continue,
                    Some(crate::git::SequenceFamily::Revert),
                ),
                GitCommand::RevertSkip => self.request_git_control(
                    crate::git::SequenceControl::Skip,
                    Some(crate::git::SequenceFamily::Revert),
                ),
                GitCommand::RevertAbort => self.request_git_control(
                    crate::git::SequenceControl::Abort,
                    Some(crate::git::SequenceFamily::Revert),
                ),
                GitCommand::OperationContinue => {
                    self.request_git_control(crate::git::SequenceControl::Continue, None)
                }
                GitCommand::OperationSkip => {
                    self.request_git_control(crate::git::SequenceControl::Skip, None)
                }
                GitCommand::OperationAbort => {
                    self.request_git_control(crate::git::SequenceControl::Abort, None)
                }
                GitCommand::TagCreate => self.prompt_git_tag(),
                GitCommand::TagDelete => self.confirm_git_tag_delete(),
                GitCommand::Push => self.prepare_git_push(),
                GitCommand::Fetch => self.fetch_git_remote(),
                GitCommand::Refs => self.open_git_refs(),
                GitCommand::BranchCreate => self.prompt_git_branch(refs::Intent::Create),
                GitCommand::BranchSwitch => self.prompt_git_branch(refs::Intent::Switch),
                GitCommand::BranchRename => self.prompt_git_branch(refs::Intent::Rename),
                GitCommand::BranchDelete => self.prompt_git_branch(refs::Intent::Delete),
                GitCommand::ShowRange => self.show_git_range(),
                GitCommand::CherryPick => {
                    self.prepare_git_application(crate::git::ApplyKind::CherryPick)
                }
                GitCommand::Revert => self.prepare_git_application(crate::git::ApplyKind::Revert),
                GitCommand::SequenceMark => self.mark_git_revision(true, false, false),
                GitCommand::SequenceUnmark => self.mark_git_revision(true, true, false),
                GitCommand::Reflog => self.open_git_reflog(),
                GitCommand::Blame => self.open_git_blame(),
                GitCommand::Log => self.open_git_history(),
                GitCommand::Show => self.show_git_revision(),
                GitCommand::LogNextPage => self.continue_git_history(true),
                GitCommand::Clone => self.compose_git_family(crate::git::Family::Clone),
                GitCommand::CloneShallow => {
                    self.compose_git_family(crate::git::Family::CloneShallow)
                }
                GitCommand::CloneBare => self.compose_git_family(crate::git::Family::CloneBare),
                GitCommand::CloneMirror => self.compose_git_family(crate::git::Family::CloneMirror),
                GitCommand::Init => self.compose_git_family(crate::git::Family::Init),
                GitCommand::Ignore => self.visit_git_ignore(false),
                GitCommand::IgnoreExclude => self.visit_git_ignore(true),
                GitCommand::SubmoduleAdd => {
                    self.compose_git_family(crate::git::Family::SubmoduleAdd)
                }
                GitCommand::SubmoduleList => {
                    self.compose_git_family(crate::git::Family::SubmoduleList)
                }
                GitCommand::SubmoduleUpdate => {
                    self.compose_git_family(crate::git::Family::SubmoduleUpdate)
                }
                GitCommand::SubmoduleSync => {
                    self.compose_git_family(crate::git::Family::SubmoduleSync)
                }
                GitCommand::SubmoduleDeinit => {
                    self.compose_git_family(crate::git::Family::SubmoduleDeinit)
                }

                GitCommand::SubtreeAdd => self.compose_git_family(crate::git::Family::SubtreeAdd),
                GitCommand::SubtreeMerge => {
                    self.compose_git_family(crate::git::Family::SubtreeMerge)
                }
                GitCommand::SubtreePull => self.compose_git_family(crate::git::Family::SubtreePull),
                GitCommand::SubtreePush => self.compose_git_family(crate::git::Family::SubtreePush),
                GitCommand::SubtreeSplit => {
                    self.compose_git_family(crate::git::Family::SubtreeSplit)
                }
                GitCommand::NotesAdd => self.compose_git_family(crate::git::Family::NotesAdd),
                GitCommand::NotesRemove => self.compose_git_family(crate::git::Family::NotesRemove),
                GitCommand::NotesPrune => self.compose_git_family(crate::git::Family::NotesPrune),
                GitCommand::SparseEnable => {
                    self.compose_git_family(crate::git::Family::SparseEnable)
                }
                GitCommand::SparseDisable => {
                    self.compose_git_family(crate::git::Family::SparseDisable)
                }
                GitCommand::SparseReapply => {
                    self.compose_git_family(crate::git::Family::SparseReapply)
                }
                GitCommand::SparseSet => self.compose_git_family(crate::git::Family::SparseSet),
                GitCommand::SparseAdd => self.compose_git_family(crate::git::Family::SparseAdd),
                GitCommand::FormatPatch => self.compose_git_family(crate::git::Family::FormatPatch),
                GitCommand::SaveDiff => self.compose_git_family(crate::git::Family::SaveDiff),
                GitCommand::ApplyPatch => self.compose_git_family(crate::git::Family::ApplyPatch),
                GitCommand::ApplyMailbox => {
                    self.compose_git_family(crate::git::Family::ApplyMailbox)
                }
                GitCommand::RunCommand => self.compose_git_family(crate::git::Family::RunCommand),
                GitCommand::RunShellCommand => {
                    self.compose_git_family(crate::git::Family::RunShellCommand)
                }
                GitCommand::RequestPull => self.compose_git_family(crate::git::Family::RequestPull),
                GitCommand::Status => self.open_git(),
                GitCommand::Commit => self.compose_git_commit(),
                GitCommand::CommitAmend => self.prepare_git_amend(),
                GitCommand::CommitFixup => self.prompt_git_fixup(),
                GitCommand::CommitSubmit | GitCommand::CommitCancel => {
                    self.message("No commit composer is active")
                }
                GitCommand::Quit => self.quit_git(),
                GitCommand::Refresh => self.refresh_git(),
                GitCommand::ToggleSection => self.toggle_git_section(),
                GitCommand::Diff => self.open_git_diff(comparisons::Mode::Detail),
                GitCommand::VisitFile => self.visit_git_file(),
                GitCommand::Discard => self.discard_git(),
                GitCommand::Mark if self.git.page == Page::Refs => self.mark_git_push(false),
                GitCommand::Mark if self.git.page == Page::History => {
                    self.mark_git_revision(false, false, false)
                }
                GitCommand::Mark => self.mark_git(),
                GitCommand::Unmark if self.git.page == Page::Refs => self.mark_git_push(true),
                GitCommand::Unmark if self.git.page == Page::History => {
                    self.mark_git_revision(false, true, false)
                }
                GitCommand::Unmark => self.unmark_git(false),
                GitCommand::UnmarkAll if self.git.page == Page::History => {
                    self.mark_git_revision(false, true, true)
                }
                GitCommand::UnmarkAll => self.unmark_git(true),
                GitCommand::Stage => self.change_git_index(crate::git::IndexDirection::Stage),
                GitCommand::Unstage => self.change_git_index(crate::git::IndexDirection::Unstage),
                GitCommand::Dispatch => {
                    if !self.git.visible {
                        self.message("git-dispatch requires a Git presentation")
                    } else {
                        self.git.state.menu = true;
                        Action::Redraw
                    }
                }
                GitCommand::Processes => self.open_git_processes(),
                GitCommand::ProcessShow => self.show_git_process(),
                GitCommand::ProcessCancel => self.cancel_git_process(),
            });
        }
        if !self.git.visible {
            return None;
        }
        if self.git.state.menu && matches!(command, Command::Scroll(_) | Command::Motion(_)) {
            return Some(Action::Wait);
        }
        Some(match command {
            Command::Quit => self.quit(),
            Command::ExecuteExtended => self.extended(),
            Command::KeyboardQuit => {
                if let Some(action) = self.cancel_git_discard() {
                    return Some(action);
                }
                if self.cancel_quit() {
                    return Some(self.message("Quit canceled; Git work retained"));
                }
                self.pending = None;
                self.input.cancel();
                self.git.state.menu = false;
                self.message("Git input canceled; owned work retained")
            }
            Command::Scroll(Scroll::NextLine) => self.move_git(1, false),
            Command::Scroll(Scroll::PreviousLine) => self.move_git(1, true),
            Command::Scroll(Scroll::PageForward) => self.move_git(self.git_extent(), false),
            Command::Scroll(Scroll::PageBackward) => self.move_git(self.git_extent(), true),
            Command::Motion(Motion::BufferStart) => self.move_git(usize::MAX, true),
            Command::Motion(Motion::BufferEnd) => self.move_git(usize::MAX, false),
            _ => self.message("Git presentation is read-only; ? lists available actions"),
        })
    }

    fn open_git(&mut self) -> Action {
        if self.git.visible {
            self.git.page = Page::Status;
            self.git.state.menu = false;
            if self.git.state.context.is_none() && self.git.state.request.is_none() {
                self.git.visible = false;
                return self.open_git();
            }
            return self.refresh_git();
        }
        if self.layout_active() {
            return self
                .message("Source layout is in progress; retry Git entry after it completes");
        }
        let directory = self
            .buffer()
            .ok()
            .and_then(|b| b.file())
            .and_then(|f| f.path().parent())
            .or_else(|| self.tree.chosen_root())
            .unwrap_or(&self.git.launch)
            .to_owned();
        match self.git.engine.open(&directory) {
            Ok(operation) => {
                self.git.state.request = Some((operation, true));
                self.git.state.menu = false;
                self.git.visible = true;
                self.git.page = Page::Status;
                self.git.painted = None;
                self.cancel_pointer_capture();
                self.cancel_terminal_pointer();
                self.input.set_git(true);
                self.message("Checking Git repository")
            }
            Err(error) => self.error(&error),
        }
    }
    fn quit_git(&mut self) -> Action {
        if !self.git.state.menu
            && (self.quit_git_ref_detail()
                || self.quit_git_stash_detail()
                || self.quit_git_worktree_detail())
        {
            return Action::Redraw;
        }

        if !self.git.visible {
            return self.message("No Git presentation is active");
        }
        if self.git.state.menu {
            self.git.state.menu = false;
            return Action::Redraw;
        }
        match self.git.page {
            Page::Worktree | Page::Stash | Page::Refs | Page::History | Page::Sequence => {
                self.git.page = Page::Status;
                return Action::Redraw;
            }
            Page::Conflict => {
                self.git.page = Page::Sequence;
                return Action::Redraw;
            }
            Page::Revision => {
                self.git.page = self.git.ancestry.revision_origin;
                return Action::Redraw;
            }
            Page::RevisionFile => {
                self.git.page = self.git.ancestry.file_origin;
                return Action::Redraw;
            }
            Page::Reflog | Page::Blame => {
                self.git.page = Page::Status;
                return Action::Redraw;
            }
            Page::Process(_) => {
                self.git.page = Page::Processes;
                return Action::Redraw;
            }
            Page::Diff => {
                self.git.page = Page::Status;
                self.git.inspection.return_to_status();
                return Action::Redraw;
            }
            Page::Processes if self.git.process_origin.is_some() => {
                self.git.page = self.git.process_origin.take().expect("Git process origin");
                return Action::Redraw;
            }
            _ => {}
        }
        self.git.visible = false;
        self.git.painted = None;
        self.input.set_git(false);
        self.sync_input_context();
        self.message(if self.git.engine.busy() {
            "Returned from Git; owned work continues"
        } else {
            "Returned from Git"
        })
    }
    fn refresh_git(&mut self) -> Action {
        if self.git.visible && self.git.page == Page::Worktree {
            return self.open_git_worktrees();
        }
        if self.git.visible && self.git.page == Page::Stash {
            return self.open_git_stashes();
        }
        if self.git.visible && self.git.page == Page::Conflict {
            self.git.page = Page::Sequence;
            return self.open_git_sequence();
        }
        if self.git.visible && self.git.page == Page::Sequence {
            return self.open_git_sequence();
        }
        if self.git.visible && self.git.page == Page::Refs {
            return self.open_git_refs();
        }
        if self.git.visible
            && matches!(
                self.git.page,
                Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
            )
        {
            return self.refresh_git_inspection();
        }
        if self.git.visible && matches!(self.git.page, Page::History | Page::Revision) {
            return self.continue_git_history(false);
        }
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Git Status is still checking its repository context");
        };
        if self.git.state.request.is_some()
            || (self.git.index_request.is_some() || self.git.discard_request.is_some())
        {
            return self.message("Git observation is already active");
        }
        match self.git.engine.refresh(context) {
            Ok(operation) => {
                self.git.inspection.after_refresh = self.git.inspection.resume(self.git.page);
                self.git.state.request = Some((operation, false));
                self.git.state.menu = false;
                self.message("Refreshing Git Status")
            }
            Err(error) => self.error(&error),
        }
    }
    fn git_extent(&self) -> usize {
        usize::from(self.size.height.saturating_sub(5).max(1))
    }
    fn move_git(&mut self, distance: usize, backward: bool) -> Action {
        match self.git.page {
            Page::Worktree => return self.move_git_worktrees(distance, backward),
            Page::Stash => return self.move_git_stashes(distance, backward),
            Page::Refs => return self.move_git_refs(distance, backward),
            Page::Sequence => return self.move_git_sequence(distance, backward),
            Page::Conflict => return self.move_git_conflict(distance, backward),
            Page::History | Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile => {
                return self.move_git_history(distance, backward);
            }
            Page::Processes => return self.move_git_process(distance, backward),
            Page::Process(id) => return self.scroll_git_process(id, distance, backward),
            Page::Diff => return self.move_git_diff(distance, backward),
            Page::Status => {}
        }
        if self.git.status().is_none() {
            return self.message("Git Status is unavailable or checking context");
        }
        let selected = self.git.state.selected.as_ref();
        let position = self
            .git
            .items()
            .filter(Item::selectable)
            .position(|i| selected.is_some_and(|t| i.matches(t)))
            .unwrap_or(0);
        let count = self.git.items().filter(Item::selectable).count();
        let next = if backward {
            position.saturating_sub(distance)
        } else {
            position
                .saturating_add(distance)
                .min(count.saturating_sub(1))
        };
        let target = self
            .git
            .items()
            .filter(Item::selectable)
            .nth(next)
            .map(|i| i.target());
        self.git.state.selected = target;
        self.git.reconcile(self.git_extent());
        Action::Redraw
    }
    fn select_git_row(&mut self, index: usize) -> Action {
        let target = self.git.items().nth(index).map(|i| i.target());
        if target.is_some() {
            self.git.state.selected = target;
            self.git.reconcile(self.git_extent());
        }
        Action::Redraw
    }
    fn toggle_git_section(&mut self) -> Action {
        if self.git.page == Page::Worktree {
            return self.toggle_git_worktree_detail();
        }
        if self.git.page == Page::Stash {
            return self.show_git_stash();
        }
        if self.git.page == Page::Refs {
            return self.toggle_git_ref_detail();
        }

        if self.git.visible
            && matches!(
                self.git.page,
                Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
            )
        {
            return self.toggle_git_inspection();
        }
        self.git.state.menu = false;
        if matches!(self.git.page, Page::Processes | Page::Process(_)) {
            return self.show_git_process();
        }
        if self.git.page == Page::Diff {
            return self.message("Diff detail is read-only; q returns to Status expansion");
        }
        let group = match self.git.state.selected.as_ref() {
            Some(Target::Section(group)) => *group,
            Some(Target::Path(group, path)) => {
                if self.git.inspection.expanded
                    && self
                        .git
                        .inspection
                        .detail
                        .as_ref()
                        .is_some_and(|d| d.group == *group && d.change.path == *path)
                {
                    self.git.inspection.expanded = false;
                    return Action::Redraw;
                }
                return self.open_git_diff(comparisons::Mode::Inline);
            }
            Some(Target::Hunk(_, _, id)) => {
                self.git.inspection.expanded_hunk =
                    if self.git.inspection.expanded_hunk == Some(*id) {
                        None
                    } else {
                        Some(*id)
                    };
                return Action::Redraw;
            }
            None => return self.message("No current Git section is selected"),
        };
        let index = Group::ALL
            .iter()
            .position(|g| *g == group)
            .expect("known Git group");
        self.git.state.collapsed[index] = !self.git.state.collapsed[index];
        self.git.reconcile(self.git_extent());
        Action::Redraw
    }
    pub(super) fn expire_git(&mut self) -> Action {
        let updates = self.git.engine.poll();
        if updates.is_empty() {
            return Action::Wait;
        }
        for update in updates {
            if matches!(
                update.kind,
                crate::git::OperationKind::FamilyPrepare | crate::git::OperationKind::Family
            ) {
                self.publish_git_family(&update);
                continue;
            }
            if self.git.worktrees.request == Some(update.operation)
                || self.git.worktrees.preparing == Some(update.operation)
            {
                self.publish_git_worktrees(&update);
                continue;
            }
            if self.git.repository.request == Some(update.operation)
                || self.git.repository.preparing == Some(update.operation)
            {
                self.publish_git_repository(&update);
                continue;
            }
            if self
                .git
                .refs
                .bisect_preparing
                .is_some_and(|(id, _)| id == update.operation)
            {
                self.publish_git_bisect_preparation(&update);
                continue;
            }
            if self.git.refs.application_preparing == Some(update.operation) {
                self.publish_git_application_preparation(&update);
                continue;
            }
            if self.git.refs.autosquash_preparing == Some(update.operation) {
                self.publish_git_autosquash_preparation(&update);
                continue;
            }
            if self.git.conflicts.explaining == Some(update.operation) {
                self.publish_git_explanation(&update);
                continue;
            }
            if self.git.conflicts.preparing == Some(update.operation) {
                self.publish_git_resolution_preparation(&update);
                continue;
            }
            if self.git.conflicts.resolving == Some(update.operation) {
                self.publish_git_resolution(&update);
                continue;
            }
            if self.git.conflicts.request == Some(update.operation) {
                self.publish_git_conflict(&update);
                continue;
            }
            if self.git.refs.integration_request == Some(update.operation) {
                self.publish_git_integration(&update);
                continue;
            }
            if self.git.refs.push_preparing == Some(update.operation) {
                self.publish_git_push_preparation(&update);
                continue;
            }
            if self.git.refs.tag_preparing == Some(update.operation) {
                self.publish_git_tag_preparation(&update);
                continue;
            }
            if self.git.refs.request == Some(update.operation) {
                self.publish_git_refs(&update);
                continue;
            }
            if self
                .git
                .ancestry
                .request
                .is_some_and(|(id, _)| id == update.operation)
            {
                self.publish_git_history(&update);
                continue;
            }
            if self.git.commit_preparing == Some(update.operation) {
                self.publish_git_preparation(&update);
                continue;
            }
            if self.git.commit_request == Some(update.operation) {
                self.publish_git_commit(&update);
                continue;
            }
            if self.git.discard_request == Some(update.operation) {
                self.publish_git_discard(&update);
                continue;
            }
            if self.git.index_request == Some(update.operation) {
                self.publish_git_index(&update);
                continue;
            }
            if self
                .git
                .inspection
                .request
                .as_ref()
                .is_some_and(|r| r.operation == update.operation)
            {
                self.publish_git_diff(&update);
                self.finish_git_index_preparation(&update);
                continue;
            }
            if self
                .git
                .state
                .request
                .is_none_or(|(request, _)| request != update.operation)
            {
                continue;
            }
            let opening = self.git.state.request.take().expect("Git request").1;
            if update.published {
                if self.git.state.context != update.context {
                    self.git.state.selected = None;
                    self.git.state.collapsed = [false; 4];
                    self.git.state.top = 0;
                    self.git.inspection = comparisons::Inspection::default();
                }
                self.git.state.context = update.context;
                self.git.reconcile(self.git_extent());
                if let Some(resume) = self.git.inspection.after_refresh.take() {
                    self.request_git_diff(resume);
                    continue;
                }
                if self.git.visible
                    && self.input.focus() == Focus::Editing
                    && self.pending.is_none()
                {
                    self.message("Git Status refreshed");
                }
            } else if opening && self.git.visible && self.git.page == Page::Status {
                self.git.visible = false;
                self.input.set_git(false);
                self.sync_input_context();
                let diagnostic = self
                    .git
                    .engine
                    .records()
                    .iter()
                    .find(|r| r.id == update.operation)
                    .and_then(|r| r.diagnostic.as_deref())
                    .unwrap_or("Repository observation unavailable");
                let error = format!("Git unavailable: {}", label(diagnostic.as_bytes()));
                return self.error(&error);
            } else if self.git.visible
                && self.input.focus() == Focus::Editing
                && self.pending.is_none()
            {
                self.git.inspection.after_refresh = None;
                self.message("Git refresh failed; confirmed observation retained");
            }
        }
        Action::Redraw
    }
    pub(super) fn git_input(&mut self, event: &Event) -> Option<Action> {
        self.input
            .set_git_worktree(self.git.visible && self.git.page == Page::Worktree);
        self.input
            .set_git_stash(self.git.visible && self.git.page == Page::Stash);
        self.input
            .set_git_conflict(self.git.visible && self.git.page == Page::Conflict);
        self.input
            .set_git_sequence(self.git.visible && self.git.page == Page::Sequence);
        self.input
            .set_git_refs(self.git.visible && self.git.page == Page::Refs);
        self.input
            .set_git_history(self.git.visible && self.git.page == Page::History);
        self.input.set_git_inspect(
            self.git.visible
                && matches!(
                    self.git.page,
                    Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
                ),
        );
        if let Some(action) = self.git_ref_review_input(event) {
            return Some(action);
        }
        if self.git.composer.is_some() && matches!(event, Event::Mouse(_)) {
            return Some(Action::Wait);
        }
        if !self.git.visible || self.input.focus() != Focus::Editing {
            return None;
        }
        if matches!(self.git.page, Page::Processes | Page::Process(_))
            && !self.git.state.menu
            && self.pending.is_none()
            && self.input.prefix().is_none()
            && crate::input::Input::interrupt(event)
        {
            return Some(self.cancel_git_process());
        }
        match event {
            Event::Paste(_) => Some(self.message("Git presentation is read-only")),
            Event::Mouse(mouse) => {
                if !self.mouse_enabled || self.git.state.menu {
                    return Some(Action::Wait);
                }
                if mouse.column >= self.size.width
                    || mouse.row >= self.size.height.saturating_sub(2)
                {
                    return Some(Action::Wait);
                }

                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
                    && !self.git.painted.as_ref().is_some_and(|painted| {
                        painted.refs == self.git.refs().map(|r| r.id)
                            && painted.refs_top == self.git.refs.top
                            && painted.conflict
                                == (
                                    self.git.conflicts.detail.as_ref().map(|c| c.id),
                                    self.git.conflicts.top,
                                )
                            && painted.sequence_top == self.git.refs.sequence_top
                            && painted.context == self.git.state.context
                            && painted.generation == self.git.observation().map(|o| o.generation)
                            && painted.worktree == self.git.worktree_token()
                            && painted.stash == self.git.stash_token()
                            && painted.inspected == self.git.inspection_token()
                            && painted.history == self.git.history().map(|h| h.id)
                            && painted.revision == self.git.revision().map(|d| d.id)
                            && painted.history_top == self.git.ancestry.top
                            && painted.revision_top == self.git.ancestry.detail_top
                            && painted.top == self.git.state.top
                            && painted.collapsed == self.git.state.collapsed
                            && painted.size == self.size
                            && painted.focus == self.input.focus()
                            && painted.menu == self.git.state.menu
                            && painted.page == self.git.page
                            && painted.processes == self.git.process_ids().collect::<Vec<_>>()
                            && painted.process_top == self.git.process_top
                            && painted.comparison
                                == self.git.inspection.detail.as_ref().map(|d| d.id)
                            && painted.comparison_top == self.git.inspection.top
                            && painted.expanded == self.git.inspection.expanded
                            && painted.expanded_hunk == self.git.inspection.expanded_hunk
                    })
                {
                    return Some(
                        self.message("Git view changed; redraw before selecting a target"),
                    );
                }
                Some(match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        self.move_git(self.buffers.snapshot().settings().mouse.scroll_lines, true)
                    }
                    MouseEventKind::ScrollDown => {
                        self.move_git(self.buffers.snapshot().settings().mouse.scroll_lines, false)
                    }
                    MouseEventKind::Down(MouseButton::Left)
                        if mouse.modifiers.is_empty()
                            && (mouse.row >= 3 || self.git.page == Page::Sequence)
                            && mouse.row < self.size.height.saturating_sub(2)
                            && mouse.column < self.size.width
                            && !self.git.state.menu =>
                    {
                        let index = usize::from(mouse.row - 3);
                        if self.git.page == Page::Sequence {
                            self.select_git_sequence(
                                self.git.refs.sequence_top + usize::from(mouse.row),
                            )
                        } else if self.git.page == Page::Worktree {
                            self.select_git_worktree(self.git.worktrees.top + index)
                        } else if self.git.page == Page::Stash {
                            self.select_git_stash(self.git.repository.top + index)
                        } else if self.git.page == Page::Refs {
                            self.select_git_ref(self.git.refs.top + index)
                        } else if self.git.page == Page::Processes {
                            self.select_git_process(self.git.process_top + index)
                        } else if matches!(
                            self.git.page,
                            Page::History
                                | Page::Revision
                                | Page::Reflog
                                | Page::Blame
                                | Page::RevisionFile
                        ) {
                            self.select_git_history(index)
                        } else if self.git.page == Page::Status {
                            self.select_git_row(self.git.state.top + index)
                        } else if self.git.page == Page::Diff {
                            self.select_git_diff(self.git.inspection.top + index)
                        } else {
                            Action::Wait
                        }
                    }
                    _ => Action::Wait,
                })
            }
            _ => None,
        }
    }
    pub(super) fn git_frame(&mut self) -> Result<Frame, crate::view::Error> {
        if matches!(self.pending, Some(super::Pending::GitRef(_))) {
            return self.git_ref_review_frame();
        }
        if self.git_commit_reviewing() {
            return self.git_commit_review_frame();
        }
        self.git.reconcile(self.git_extent());
        let popup = self.input.palette().and_then(|palette| {
            crate::panel::Composition::navigation(self.size, palette.requested_rows())
        });
        self.input.reveal_commands(
            popup
                .as_ref()
                .and_then(|c| c.auxiliary())
                .map_or(0, |p| p.content().height()),
        );
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let context = self
            .git
            .state
            .context
            .and_then(|id| self.git.engine.context(id));
        let observation = self.git.observation();
        let status = observation.and_then(|o| o.status.as_ref());
        let mut rows = self.git.items().skip(self.git.state.top);
        let mut comparison_rows = self
            .git
            .inspection
            .detail
            .as_ref()
            .into_iter()
            .flat_map(|d| d.lines.iter().map(move |line| (d.as_ref(), line)))
            .skip(self.git.inspection.top);
        let mut process_rows = self.git.process_rows().skip(match self.git.page {
            Page::Process(_) => self.git.detail_top,
            _ => self.git.process_top,
        });
        let mut frame = Frame::git(
            self.buffer()?,
            self.size,
            self.buffers.palette(),
            &self.echo,
            match self.git.page {
                Page::Worktree => {
                    " Git Worktrees  c create  v Status visit  L lock  U unlock  P prune  k remove  q Status"
                }
                Page::Status => {
                    " Git Status  c commit  Tab expand  Return visit  g refresh  $ processes  ? actions  q return"
                }
                Page::Conflict => {
                    " Git Conflict  Return editable result  Up/Down scroll  g sequence refresh  q Operation"
                }
                Page::Sequence => {
                    " Git Operation  g refresh  $ processes  Up/Down scroll  q Status"
                }
                Page::Stash => {
                    " Git Stashes  c create  v inspect  a apply  p pop  b branch  k drop  g refresh  q Status"
                }
                Page::Refs => {
                    " Git Refs  n branch  S switch  R rename  D delete  t/T tags  ? actions  g refresh  q Status"
                }
                Page::History => {
                    " Git History  Tab revision  * base  Return range  a/U ordered marks  f next page  q Status"
                }
                Page::Revision => {
                    " Git Revision  Return parent/file  v parent review  g refresh  q return"
                }
                Page::Reflog => " Git Reflog  Return / v revision  g refresh  q Status",
                Page::Blame => {
                    " Git Blame  Return historical source  v revision  g refresh  q Status"
                }
                Page::RevisionFile => {
                    " Git Historical File  Up/Down scroll  git-visit-file current source  q return"
                }
                Page::Diff => " Git Diff  g refresh  Return visit source  $ processes  q Status",
                Page::Processes => " Git Processes  Return inspect  C-g cancel selected  q return",
                Page::Process(_) => {
                    " Git Process  Up/Down scroll  C-g cancel this operation  q return"
                }
            },
            admission,
            |row| {
                if self.git.state.menu && self.git.page == Page::Refs {
                    return self.git.refs_menu(row);
                }
                if self.git.state.menu {
                    return (
                        [
                            "Git actions",
                            "c  git-commit (native composer)",
                            "g  git-refresh",
                            "Tab  git-toggle-section",
                            "Return  git-visit-file",
                            "s / u  git-stage / git-unstage",
                            "* / - / 0  mark / unmark / clear",
                            "k / x d  git-discard (confirmed)",
                            "M-x git-diff  selected comparison",
                            "$  git-processes",
                            "M-x / :  native commands",
                            "q / C-g  return to Status",
                        ]
                        .get(row)
                        .unwrap_or(&"")
                        .to_string(),
                        false,
                    );
                }
                if self.git.page == Page::Worktree {
                    return self
                        .git
                        .worktree_row(row, usize::from(self.size.width.max(1)));
                }
                if self.git.page == Page::Stash {
                    return self.git.stash_row(row, usize::from(self.size.width.max(1)));
                }
                if self.git.page == Page::Refs {
                    return self.git.refs_row(row, usize::from(self.size.width.max(1)));
                }
                if self.git.page == Page::Conflict {
                    return self.git.conflict_row(row, usize::from(self.size.width));
                }
                if self.git.page == Page::Sequence {
                    return self.git.sequence_row(row, usize::from(self.size.width));
                }
                if self.git.page == Page::Diff {
                    return if row < 3 {
                        (self.git.diff_header(row), false)
                    } else {
                        comparison_rows.next().map_or_else(
                            || (String::new(), false),
                            |(diff, line)| {
                                let selected = line.hunk.is_some()
                                    && line.hunk == self.git.inspection.selected;
                                (
                                    format!(
                                        "{}{} {}",
                                        if selected { ">" } else { " " },
                                        if line.hunk.is_some_and(|id| self.git.marks.contains(
                                            &Target::Hunk(diff.group, diff.change.path.clone(), id)
                                        )) {
                                            "*"
                                        } else {
                                            " "
                                        },
                                        comparisons::line_label(diff, line)
                                    ),
                                    selected,
                                )
                            },
                        )
                    };
                }
                if self.git.page == Page::History {
                    return if row < 3 {
                        (self.git.history_header(row), false)
                    } else {
                        self.git
                            .history()
                            .and_then(|h| h.revisions().nth(self.git.ancestry.top + row - 3))
                            .map_or_else(
                                || (String::new(), false),
                                |r| {
                                    let selected = Some(r.object) == self.git.ancestry.selected;
                                    (
                                        format!(
                                            "{} {}{} {} {}",
                                            if selected { ">" } else { " " },
                                            self.git.revision_mark_label(r.object),
                                            r.object.hex(),
                                            label(&r.authored),
                                            label(&r.subject)
                                        ),
                                        selected,
                                    )
                                },
                            )
                    };
                }
                if matches!(
                    self.git.page,
                    Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
                ) {
                    return self.git.inspection_row(row, self.size.width);
                }
                if self.git.page != Page::Status {
                    return if row < 3 {
                        (self.git.process_header(row), false)
                    } else {
                        process_rows.next().unwrap_or_default()
                    };
                }
                if row == 0 {
                    return (header(observation), false);
                }
                if row == 1 {
                    return (
                        status
                            .and_then(|s| {
                                s.upstream.as_ref().map(|u| {
                                    format!(
                                        "Upstream: {}  ahead {}  behind {}",
                                        label(u),
                                        s.ahead.map_or("unavailable".into(), |n| n.to_string()),
                                        s.behind.map_or("unavailable".into(), |n| n.to_string())
                                    )
                                })
                            })
                            .unwrap_or_default(),
                        false,
                    );
                }
                if row == 2 {
                    return (
                        format!(
                            "{}{}{}{}{}",
                            if self.git.state.request.is_some()
                                || (self.git.index_request.is_some()
                                    || self.git.discard_request.is_some())
                            {
                                "Checking / refreshing  "
                            } else {
                                ""
                            },
                            if context.is_some_and(|c| c.failure.is_some()) {
                                "STALE: refresh failed; confirmed observation retained  "
                            } else {
                                ""
                            },
                            if self.git.inspection.expanded {
                                self.git.diff_status()
                            } else {
                                String::new()
                            },
                            if status.is_some_and(
                                |s| s.completeness != crate::git::Completeness::Complete
                            ) {
                                "TRUNCATED: path admission reached  "
                            } else {
                                ""
                            },
                            if self.git.marks.targets.is_empty() {
                                String::new()
                            } else {
                                format!(
                                    "  {} marked{}; 0 clears",
                                    self.git.marks.targets.len(),
                                    if self
                                        .git
                                        .marks
                                        .targets
                                        .iter()
                                        .all(|t| self.git.engine.detail_current(t.diff()))
                                    {
                                        ""
                                    } else {
                                        " / STALE"
                                    }
                                )
                            }
                        ),
                        false,
                    );
                }
                let Some(status) = status else {
                    return (
                        if row == 3 {
                            if observation.is_some() {
                                "Bare repository: working-tree Status unavailable"
                            } else {
                                "Checking repository context"
                            }
                        } else {
                            ""
                        }
                        .to_owned(),
                        false,
                    );
                };
                if status.paths.is_empty() {
                    return (
                        if row == 3 { "Working tree clean" } else { "" }.to_owned(),
                        false,
                    );
                }
                let item = rows.next();
                let selected = item.as_ref().is_some_and(|i| {
                    self.git
                        .state
                        .selected
                        .as_ref()
                        .is_some_and(|t| i.matches(t))
                });
                let marked = item
                    .as_ref()
                    .is_some_and(|i| self.git.marks.contains(&i.target()));
                let marker = format!(
                    "{}{}",
                    if selected { ">" } else { " " },
                    if marked { "*" } else { " " }
                );
                let text = match item {
                    Some(Item::Section(group, count)) => {
                        format!("{marker} {} ({count})", group.label())
                    }
                    Some(Item::Path(_, path)) => format!(
                        "{marker}   {}",
                        label(path.path.as_os_str().as_encoded_bytes())
                    ),
                    Some(Item::Detail(diff, line)) => {
                        format!("{marker}     {}", comparisons::line_label(diff, line))
                    }
                    None => String::new(),
                };
                (text, selected)
            },
        )?;
        if let Some(popup) = &popup {
            frame.git_popup(popup)?;
        }
        frame.message_kind(self.echo_kind);
        self.prompt_frame(&mut frame, popup.as_ref())?;
        drop(rows);
        drop(comparison_rows);
        drop(process_rows);
        self.git.painted = Some(Painted {
            worktree: self.git.worktree_token(),
            stash: self.git.stash_token(),
            context: self.git.state.context,
            generation: self.git.observation().map(|o| o.generation),
            top: self.git.state.top,
            collapsed: self.git.state.collapsed,
            size: self.size,
            focus: self.input.focus(),
            menu: self.git.state.menu,
            page: self.git.page,
            processes: self.git.process_ids().collect(),
            process_top: self.git.process_top,
            comparison: self.git.inspection.detail.as_ref().map(|d| d.id),
            comparison_top: self.git.inspection.top,
            history: self.git.history().map(|h| h.id),
            revision: self.git.revision().map(|d| d.id),
            history_top: self.git.ancestry.top,
            revision_top: self.git.ancestry.detail_top,
            inspected: self.git.inspection_token(),
            refs: self.git.refs().map(|r| r.id),
            refs_top: self.git.refs.top,
            conflict: (
                self.git.conflicts.detail.as_ref().map(|c| c.id),
                self.git.conflicts.top,
            ),
            sequence_top: self.git.refs.sequence_top,
            expanded: self.git.inspection.expanded,
            expanded_hunk: self.git.inspection.expanded_hunk,
        });
        Ok(frame)
    }
}

fn header(observation: Option<&Observation>) -> String {
    let Some(observation) = observation else {
        return "Git: checking repository".into();
    };
    let identity = &observation.identity;
    let root = identity
        .worktree
        .as_deref()
        .unwrap_or(&identity.git_directory);
    let root = root.file_name().unwrap_or(root.as_os_str());
    let head = match observation.status.as_ref().map(|s| &s.head) {
        Some(Head::Unborn { branch }) => format!("{} (unborn)", label(branch)),
        Some(Head::Branch { name, object }) => format!("{} {}", label(name), &object.hex()[..12]),
        Some(Head::Detached { object }) => format!("detached {}", &object.hex()[..12]),
        None => "bare".into(),
    };
    format!("Git: {}  {head}", label(root.as_encoded_bytes()))
}
