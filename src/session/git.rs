use super::{Action, Session};
use crate::{
    command::{Command, GitCommand, Motion, Scroll},
    display::{
        Frame,
        git::{Line as GitLine, Role as GitRole, label},
    },
    git::{ContextId, Group, Head, Observation, OperationId, PathChange, Repositories, Status},
    input::Focus,
};
use crossterm::event::{Event, MouseButton, MouseEventKind};
use std::{path::PathBuf, sync::Arc};

mod commands;
mod commit;
mod comparisons;
mod conflicts;
pub(super) mod discard;
mod families;
mod history;
mod index;
mod input;
mod lines;
mod updates;
use lines::Wrapped;
mod presentation;
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
            return Some(self.dispatch_git_command(command));
        }
        if !self.git.visible {
            return None;
        }
        if self.git.state.menu && matches!(command, Command::Scroll(_) | Command::Motion(_)) {
            return Some(Action::Wait);
        }
        Some(self.git_presentation_command(command))
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
        if let Some(action) = self.quit_git_page() {
            return action;
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
    fn quit_git_page(&mut self) -> Option<Action> {
        if [
            Page::Worktree,
            Page::Stash,
            Page::Refs,
            Page::History,
            Page::Sequence,
            Page::Reflog,
            Page::Blame,
        ]
        .contains(&self.git.page)
        {
            self.git.page = Page::Status;
            return Some(Action::Redraw);
        }
        match self.git.page {
            Page::Conflict => {
                self.git.page = Page::Sequence;
                return Some(Action::Redraw);
            }
            Page::Revision => {
                self.git.page = self.git.ancestry.revision_origin;
                return Some(Action::Redraw);
            }
            Page::RevisionFile => {
                self.git.page = self.git.ancestry.file_origin;
                return Some(Action::Redraw);
            }
            Page::Process(_) => {
                self.git.page = Page::Processes;
                return Some(Action::Redraw);
            }
            Page::Diff => {
                self.git.page = Page::Status;
                self.git.inspection.return_to_status();
                return Some(Action::Redraw);
            }
            Page::Processes if self.git.process_origin.is_some() => {
                self.git.page = self.git.process_origin.take().expect("Git process origin");
                return Some(Action::Redraw);
            }
            _ => {}
        }
        None
    }

    fn refresh_git(&mut self) -> Action {
        if self.git.visible
            && let Some(action) = self.refresh_git_page()
        {
            return action;
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
    fn refresh_git_page(&mut self) -> Option<Action> {
        if self.git.page == Page::Worktree {
            return Some(self.open_git_worktrees());
        }
        if self.git.page == Page::Stash {
            return Some(self.open_git_stashes());
        }
        if self.git.page == Page::Conflict {
            self.git.page = Page::Sequence;
            return Some(self.open_git_sequence());
        }
        if self.git.page == Page::Sequence {
            return Some(self.open_git_sequence());
        }
        if self.git.page == Page::Refs {
            return Some(self.open_git_refs());
        }
        if matches!(
            self.git.page,
            Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
        ) {
            return Some(self.refresh_git_inspection());
        }
        if matches!(self.git.page, Page::History | Page::Revision) {
            return Some(self.continue_git_history(false));
        }
        None
    }

    fn git_extent(&self) -> usize {
        usize::from(self.size.height.saturating_sub(5).max(1))
    }
    fn move_git(&mut self, distance: usize, backward: bool) -> Action {
        match self.git.page {
            Page::Worktree => self.move_git_worktrees(distance, backward),
            Page::Stash => self.move_git_stashes(distance, backward),
            Page::Refs => self.move_git_refs(distance, backward),
            Page::Sequence => self.move_git_sequence(distance, backward),
            Page::Conflict => self.move_git_conflict(distance, backward),
            _ => self.move_git_detail(distance, backward),
        }
    }
    fn move_git_detail(&mut self, distance: usize, backward: bool) -> Action {
        match self.git.page {
            Page::History | Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile => {
                self.move_git_history(distance, backward)
            }
            Page::Processes => self.move_git_process(distance, backward),
            Page::Process(id) => self.scroll_git_process(id, distance, backward),
            Page::Diff => self.move_git_diff(distance, backward),
            Page::Status => self.move_git_status(distance, backward),
            _ => unreachable!("page navigation dispatched by its owner"),
        }
    }
    fn move_git_status(&mut self, distance: usize, backward: bool) -> Action {
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
        if let Some(action) = self.toggle_git_page() {
            return action;
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
                return self.toggle_git_hunk(*id);
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
    fn toggle_git_hunk(&mut self, id: crate::git::HunkId) -> Action {
        self.git.inspection.expanded_hunk = if self.git.inspection.expanded_hunk == Some(id) {
            None
        } else {
            Some(id)
        };
        Action::Redraw
    }

    fn toggle_git_page(&mut self) -> Option<Action> {
        if self.git.page == Page::Worktree {
            return Some(self.toggle_git_worktree_detail());
        }
        if self.git.page == Page::Stash {
            return Some(self.show_git_stash());
        }
        if self.git.page == Page::Refs {
            return Some(self.toggle_git_ref_detail());
        }

        if self.git.visible
            && matches!(
                self.git.page,
                Page::Revision | Page::Reflog | Page::Blame | Page::RevisionFile
            )
        {
            return Some(self.toggle_git_inspection());
        }
        None
    }

    pub(super) fn expire_git(&mut self) -> Action {
        let updates = self.git.engine.poll();
        if updates.is_empty() {
            return Action::Wait;
        }
        for update in updates {
            if self.route_git_update(&update) {
                continue;
            }
            if let Some(action) = self.publish_git_status(&update) {
                return action;
            }
        }
        Action::Redraw
    }
}
