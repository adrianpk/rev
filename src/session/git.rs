use super::{Action, Session};
use crate::{
    command::{Command, GitCommand, Motion, Scroll},
    display::{Frame, git::label},
    git::{ContextId, Group, Head, Observation, OperationId, PathChange, Repositories, Status},
    input::Focus,
};
use crossterm::event::{Event, MouseButton, MouseEventKind};
use std::{path::PathBuf, sync::Arc};

mod comparisons;
mod index;
mod processes;
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
    index_preparing: Option<index::Preparing>,
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
        Ok(Self {
            engine: Repositories::new(snapshot, Arc::new(move || wake.signal())),
            launch: std::env::current_dir()?,
            visible: false,
            state: State::default(),
            painted: None,
            page: Page::Status,
            process_origin: None,
            inspection: comparisons::Inspection::default(),
            index_request: None,
            index_preparing: None,
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
        if let Command::Git(command) = command {
            if self.input.focus() != Focus::Editing {
                return Some(self.message("A prompt owns input focus"));
            }
            return Some(match command {
                GitCommand::Status => self.open_git(),
                GitCommand::Quit => self.quit_git(),
                GitCommand::Refresh => self.refresh_git(),
                GitCommand::ToggleSection => self.toggle_git_section(),
                GitCommand::Diff => self.open_git_diff(comparisons::Mode::Detail),
                GitCommand::VisitFile => self.visit_git_file(),
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
        if !self.git.visible {
            return self.message("No Git presentation is active");
        }
        if self.git.state.menu {
            self.git.state.menu = false;
            return Action::Redraw;
        }
        match self.git.page {
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
        let Some(context) = self.git.state.context.filter(|_| self.git.visible) else {
            return self.message("Git Status is still checking its repository context");
        };
        if self.git.state.request.is_some() || self.git.index_request.is_some() {
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
                        painted.context == self.git.state.context
                            && painted.generation == self.git.observation().map(|o| o.generation)
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
                            && mouse.row >= 3
                            && mouse.row < self.size.height.saturating_sub(2)
                            && mouse.column < self.size.width
                            && !self.git.state.menu =>
                    {
                        let index = usize::from(mouse.row - 3);
                        if self.git.page == Page::Processes {
                            self.select_git_process(self.git.process_top + index)
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
                Page::Status => {
                    " Git Status  Tab expand  Return visit  g refresh  $ processes  ? actions  q return"
                }
                Page::Diff => " Git Diff  g refresh  Return visit source  $ processes  q Status",
                Page::Processes => " Git Processes  Return inspect  C-g cancel selected  q return",
                Page::Process(_) => {
                    " Git Process  Up/Down scroll  C-g cancel this operation  q return"
                }
            },
            admission,
            |row| {
                if self.git.state.menu {
                    return (
                        [
                            "Git actions",
                            "g  git-refresh",
                            "Tab  git-toggle-section",
                            "Return  git-visit-file",
                            "s / u  git-stage / git-unstage",
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
                                        "{} {}",
                                        if selected { ">" } else { " " },
                                        comparisons::line_label(diff, line)
                                    ),
                                    selected,
                                )
                            },
                        )
                    };
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
                            "{}{}{}{}",
                            if self.git.state.request.is_some() || self.git.index_request.is_some()
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
                                "TRUNCATED: path admission reached"
                            } else {
                                ""
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
                let marker = if selected { ">" } else { " " };
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
