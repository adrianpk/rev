mod agenda;
mod agenda_actions;
mod agenda_custom;
mod agenda_queries;
mod agenda_sources;
mod ai;
mod arguments;
mod assets;
mod bookmarks;
mod buffers;
mod capture;
mod command_usage;
mod completion;
mod contexts;
mod git;
mod lsp;
mod plugins;
mod shell;
mod status;
mod terminals;
pub use crate::completion::{
    Candidate, Completion, CompletionFallback, CompletionSource, Kind as CompletionKind,
};
mod refile;
pub use refile::{ResultEntry as RefileResult, Status as RefileStatus};
mod attachment;
mod document;
mod edit;
mod editing;
mod events;
mod exports;
mod extended;
mod file;
mod goto;
mod gutter;
mod habits;
mod history;
mod jumps;
mod kill;
mod layout;
mod linear;
mod links;
mod publishing;
mod tables;
mod wrap;
pub use attachment::{AttachmentResult, HeadingStatus};
mod execution;
mod feeds;
mod previews;
pub use feeds::FeedOutcome;
mod macros;
mod modal;
mod mode;
mod motion;
mod navigation;
mod observation;
mod org;
mod outline;
mod paste;
mod path;
mod pointer;
mod prompt;
mod quit;
mod rectangle;
mod region;
mod reload;
mod repeat;
mod replace;
mod save;
mod search;
mod source_batch;
mod source_edit;
mod startup;
mod tangling;
mod timers;
mod tree;
mod visual;
mod whitespace;
mod windows;
mod yank;

use std::io::{self, Write};
use std::path::Path;

use crossterm::event::KeyCode;

use crate::buffer::{self, Buffer, BufferId, Buffers, Confirmation, Kill};
use crate::command::Command;
use crate::display::{Frame, Message, Size};
use crate::input::{Input, Key, Outcome};
use crate::terminal::Terminal;
use crate::view::{self, View};
use crate::window::Windows;

pub(crate) fn run(terminal: &mut Terminal<impl Write>, path: Option<&Path>) -> io::Result<()> {
    initial(path)?.run_terminal(terminal)
}

fn initial(path: Option<&Path>) -> io::Result<Session> {
    let (width, height) = crossterm::terminal::size()?;
    Session::startup(path, Size { width, height })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Exit,
    Redraw,
    Wait,
}

/// Owns the live editing session. Native events and named commands use the same
/// handlers; all text mutations publish through the checked buffer edit owner.
pub struct Session {
    context: crate::context::Context,
    startup_screen: Option<startup::Screen>,
    context_generation: crate::context::Generation,
    context_prepared: Option<Box<contexts::Replacement>>,
    directory_prompt: bool,
    project_directory_prompt: bool,
    workspace_directory_prompt: bool,
    context_storage: contexts::projects::Persistence,
    context_record: Option<contexts::record::Record>,
    context_membership: Option<Box<contexts::workspaces::Membership>>,
    context_management: Option<Box<contexts::management::Plan>>,
    context_root_identity: Option<(u64, u64)>,
    context_unavailable: Vec<crate::context::FileEntry>,
    context_positions: std::collections::BTreeMap<BufferId, contexts::positions::Position>,
    observation: observation::Work,
    reload: reload::Work,
    ai: ai::Work,
    shell: shell::Work,
    git: git::Work,
    terminals: terminals::Work,
    plugins: plugins::Work,
    lsp: crate::lsp::Client,
    installation: crate::lsp::install::Manager,
    installation_notice: String,
    completion: Option<Completion>,
    completion_generation: u64,
    completion_request: Option<u64>,
    completion_state: Option<crate::input::EditingState>,
    completion_source: Option<CompletionSource>,
    completion_work: crate::completion::job::Work,
    completion_resolve: Option<completion::resolve::Resolving>,
    completion_recent: Vec<BufferId>,
    completion_policy: crate::config::Completion,
    bookmark_visit: Option<bookmarks::Navigation>,
    bookmark_storage: bookmarks::Persistence,
    bookmark_list: Option<bookmarks::List>,
    macro_draft: Option<macros::definition::Draft>,
    macro_menu: Option<macros::menu::Menu>,
    bookmark_iteration: Option<bookmarks::Iteration>,
    buffers: Buffers,
    org: org::Work,
    capture: Option<capture::State>,
    block_request: Option<outline::Request>,
    source_edit: Option<source_edit::Draft>,
    tangling: Option<tangling::State>,
    execution: Option<execution::Selection>,
    execution_result: Option<crate::org::execution::Outcome>,
    feeds: Option<feeds::State>,
    preview_request: Option<outline::Request>,
    feed_results: Vec<FeedOutcome>,
    tangling_results: Vec<crate::org::tangle::Outcome>,
    refile: Option<refile::State>,
    refile_result: Vec<refile::ResultEntry>,
    refile_label: &'static str,
    links: links::State,
    document_request: Option<outline::Request>,
    table_request: Option<outline::Request>,
    export_request: Option<outline::Request>,
    publishing: Option<publishing::State>,
    publishing_results: Vec<crate::org::publish::Outcome>,
    attachment: Option<attachment::State>,
    attachment_results: std::collections::VecDeque<AttachmentResult>,
    windows: Windows,
    tree: crate::tree::Tree,
    kills: crate::kill::KillRing,
    rectangle: Option<std::sync::Arc<crate::rectangle::Rectangle>>,
    visual_source: Option<crate::buffer::Stamp>,
    one_normal: Option<jumps::Continuation>,
    repeat: repeat::State,
    macros: crate::macros::State,
    yank: Option<yank::Capture>,
    selection: Option<pointer::Selection>,
    mouse_enabled: bool,
    input: Input,
    last_find: Option<navigation::find::State>,
    last_search: Option<search::Confirmed>,
    size: Size,
    layout_pending: bool,
    layout: layout::Work,
    echo: String,
    echo_kind: Message,
    pending: Option<Pending>,
    command_origin: Option<extended::Origin>,
    task_clock: Option<fn() -> Result<String, String>>,
    org_clock: Option<outline::timestamps::Clock>,
    active_clock: Option<outline::clocks::Active>,
    retained_clock: Option<outline::clocks::Active>,
    clock_store: Option<crate::org::io::ClockStore>,
    clock_recovery_pin: Option<crate::org::io::ClockState>,
    recent_clocks: std::collections::VecDeque<crate::buffer::edit::Point>,
    calendar_date: Option<crate::org::time::Date>,
    timestamp_repeat: Option<outline::timestamps::Repeat>,
    timer: crate::org::time::Timer,
    timer_clock: Option<fn() -> std::time::Instant>,
    timer_notice: bool,
    habit_agenda: Option<habits::State>,
    agenda_sources: Option<crate::org::agenda::SourceSet>,
    agenda_run: Option<agenda_actions::Run>,
    agenda_results: Vec<crate::org::agenda::ActionResult>,
    agenda_marks: agenda_actions::Marks,
    agenda_results_view: Option<(crate::window::ViewId, usize)>,
    calendar_agenda: Option<agenda::State>,
    query_agenda: Option<agenda_queries::State>,
    agenda_query_plan: Option<crate::org::agenda::AgendaQuery>,
    agenda_custom_plan: Option<crate::config::AgendaView>,
}

enum AfterSave {
    Stay,
    Close(Kill),
    Quit(quit::Quit),
}

enum Pending {
    Plugin(crate::command::PluginCommand),
    MacroStepKey,
    MacroMenuPrompt(Box<macros::menu::Prompt>),
    MacroMenuConsent(Box<macros::menu::Consent>),
    MacroEditSelect,
    MacroDefinitionKey,
    MacroLibrary(Box<macros::library::Prompt>),
    MacroLibraryConsent(Box<macros::library::Consent>),
    MacroName(Box<macros::names::Draft>),
    MacroBind(Box<macros::names::Draft>),
    MacroConsent(Box<macros::names::Consent>),
    MacroCounter {
        command: crate::command::MacroCommand,
        view: crate::window::ViewId,
        point: crate::buffer::edit::Point,
    },
    ContextManagement(Box<contexts::management::Prompt>),
    WorkspaceName(Box<contexts::workspaces::Creation>),
    Workspaces(Vec<contexts::workspaces::WorkspaceChoice>),
    WorkspaceProjects(Vec<contexts::projects::Choice>),
    WorkspaceRoot(Box<contexts::workspaces::RootPicker>),
    ProjectName(Box<contexts::projects::Creation>),
    Projects(Vec<contexts::projects::Choice>),
    GitDiscard(Box<git::discard::Consent>),
    GitWorktree(Box<git::repository::worktrees::Prompt>),
    GitRepository(Box<git::repository::Prompt>),
    GitRef(Box<git::refs::Consent>),
    GitTagPrompt(Box<git::refs::tag::Prompt>),
    GitBranchName(Box<git::refs::Prompt>),
    GitHistoryRoot(crate::git::ContextId),
    GitHistoryParent(std::sync::Arc<crate::git::RevisionDetail>),
    GitSequenceMainline(Box<git::refs::sequences::Selection>),
    GitBisect(Box<git::refs::bisect::Prompt>),
    GitFixup(std::sync::Arc<crate::git::Observation>),
    GitQuit {
        operations: Vec<crate::git::OperationId>,
        message: String,
        sources: quit::Sources,
    },
    GitShutdown {
        message: String,
    },
    TerminalClose {
        kill: Kill,
        message: String,
    },
    TerminalQuit {
        message: String,
        sources: quit::Sources,
    },
    TerminalShutdown {
        message: String,
    },
    TerminalRename(crate::term::SessionId),
    Terminals(BufferId),
    Shell(Box<shell::Request>),
    Ai(Box<ai::Request>),
    AiDirectory,
    AiSave(Box<ai::save::Prompt>),
    AiModel(Vec<crate::ai::pi::Model>),
    Reload(Box<crate::buffer::reload::Reload>),
    LspSetup(Box<lsp::Prompt>),
    LspLifecycle(Box<lsp::LifecyclePrompt>),
    Bookmark(bookmarks::Prompt),
    BookmarkReload(Box<bookmarks::Reload>),
    BookmarkQuit(quit::Sources),
    Tangle(tangling::Prompt),
    Execution(execution::Prompt),
    Feed(feeds::Prompt),
    SourceHeader(Box<source_edit::HeaderPrompt>),
    Capture(capture::Prompt),
    Refile,
    Attachment,
    Link(Box<links::Prompt>),
    Document(Box<document::Prompt>),
    Table(Box<outline::Request>),
    TableFile(Box<tables::interchange::Prompt>),
    Export(Box<exports::Prompt>),
    Publish(Box<publishing::Prompt>),
    AgendaSources,
    AgendaDate,
    AgendaQuery(crate::org::agenda::QueryKind),
    AgendaCustom(crate::command::OrgCommand),
    AgendaBulk(agenda_actions::BulkPrompt),
    Timer(timers::Prompt),
    Habit(habits::Prompt),
    ClockReport(Box<outline::reports::Prompt>),
    Clock(Box<outline::clocks::Prompt>),
    Timestamp(Box<outline::timestamps::Prompt>),
    Estimate(Box<outline::estimates::Prompt>),
    Task(Box<outline::tasks::Prompt>),
    Tags(Box<outline::tags::Prompt>),
    Property(Box<outline::property::Prompt>),
    Quit(quit::Quit),
    TreeDelete(crate::file::Deletion),
    TreeCreate(crate::tree::Creation),
    TreeRename(crate::file::Rename),
    Replace(Box<replace::State>),
    Search(Box<search::State>),
    Command,
    GotoLine {
        view: crate::window::ViewId,
        source: crate::buffer::Stamp,
    },
    Path(Box<crate::file::completion::Navigation>),
    Save {
        confirmation: Box<Confirmation>,
        after: AfterSave,
    },
    Write {
        buffer: BufferId,
        after: AfterSave,
    },
    Buffer {
        command: crate::command::BufferCommand,
        default: BufferId,
    },
    Inventory(BufferId),
    Kill {
        kill: Kill,
        message: String,
    },
}

impl Session {
    /// Run native input, deadlines and drawing for this host-owned session.
    /// The caller acquires and enters the host terminal. On return, restore it
    /// before collecting bounded cleanup receipts outside the interactive UI.
    pub fn run_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        // Initialize Crossterm's native signal queue before enabling owned wakes.
        let result = crossterm::event::poll(std::time::Duration::ZERO).and_then(|_| {
            self.observation.wake.arm(true);
            self.plugins.host.arm(true);
            self.drive_terminal(terminal)
        });
        self.observation.wake.arm(false);
        self.plugins.host.arm(false);
        self.ai.stop();
        self.shell.stop();
        self.end_timer_session();
        self.lsp.set_enabled(false);
        // Restore the host before the noninteractive completion barrier. The
        // workers own all child discovery, signaling, waits and reaping.
        let restored = terminal.restore();
        if let Err(message) = self.flush_command_usage() {
            eprintln!("{message}");
        }
        let cleanup = self.shutdown_terminals();
        let plugins = self.shutdown_plugins();
        result.and(restored).and(cleanup).and(plugins)
    }
    fn drive_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        self.start_terminal(terminal)?;
        loop {
            let action = self.read_input()?;
            self.sync_terminal_mouse(terminal)?;
            match action {
                Action::Exit => return Ok(()),
                Action::Redraw => self.draw(terminal)?,
                Action::Wait => {}
            }
        }
    }

    fn start_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        self.sync_terminal_mouse(terminal)?;
        terminal.set_cursor_color(self.buffers.palette().cursor)?;
        self.draw(terminal)
    }

    fn sync_terminal_mouse(&self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        let (enabled, motion) = self.terminal_pointer_reporting();
        terminal.set_mouse_reporting(enabled)?;
        terminal.set_mouse_motion(motion)
    }

    /// Takes ownership of canonical buffers and a qualified initial view.
    pub fn new(buffers: Buffers, view: View, size: Size) -> Result<Self, view::Error> {
        let mut session = Self::from_buffers(buffers, view, size)?;
        session
            .tree
            .present()
            .map_err(|error| view::Error::WorkingDirectory(error.kind()))?;
        session.prepare_bookmark_markers();
        session.follow_tree();
        session.activate_lsp();
        session.startup_plugins();
        session.observe_files();
        Ok(session)
    }

    fn from_buffers(buffers: Buffers, view: View, size: Size) -> Result<Self, view::Error> {
        Self::from_buffers_with_wake(buffers, view, size, Default::default())
    }

    fn from_buffers_with_wake(
        buffers: Buffers,
        view: View,
        size: Size,
        wake: std::sync::Arc<crate::file::watch::Wake>,
    ) -> Result<Self, view::Error> {
        let buffer = buffers.get(view.point.stamp.buffer)?;
        view.check_point(buffer)?;
        let tree = crate::tree::Tree::startup(
            &buffers.snapshot().settings().tree,
            buffer.file().map(|file| file.path()),
            buffers.snapshot().settings().gutter.nerd_fonts,
        )
        .map_err(|error| view::Error::WorkingDirectory(error.kind()))?;
        let windows = Windows::with_settings(view, buffers.snapshot())?;
        let kills = crate::kill::KillRing::with_settings(buffers.snapshot());
        let mut input = Input::with_settings(buffers.snapshot());
        input.editing_owner(windows.selected_id(), buffer.id());
        input.observe_navigation(windows.selected_id(), windows.selected().point);
        input.set_org_context(buffer.is_org().then_some(crate::input::OrgContext::Editing));
        let mouse_enabled = buffers.snapshot().settings().mouse.enabled;
        let bookmark_storage = bookmarks::Persistence::configured(
            buffers.snapshot().settings().bookmarks.file.clone(),
        );
        let completion_policy = buffers.snapshot().settings().completion.clone();
        let lsp = crate::lsp::Client::new(buffers.snapshot().settings().lsp.clone());
        let observation = observation::Work::with_wake(wake);
        let terminals = terminals::Work::new(
            &buffers.snapshot().settings().terminal,
            std::env::current_dir().map_err(|e| view::Error::WorkingDirectory(e.kind()))?,
            observation.wake.clone(),
        );
        let git = git::Work::new(buffers.snapshot(), observation.wake.clone())
            .map_err(|e| view::Error::WorkingDirectory(e.kind()))?;
        Ok(Self {
            context: crate::context::Context::Standalone,
            startup_screen: None,
            context_generation: crate::context::Generation::INITIAL,
            context_prepared: None,
            directory_prompt: false,
            project_directory_prompt: false,
            workspace_directory_prompt: false,
            context_storage: Default::default(),
            context_record: None,
            context_membership: None,
            context_management: None,
            context_root_identity: None,
            context_unavailable: Vec::new(),
            context_positions: Default::default(),
            git,
            observation,
            terminals,
            plugins: Default::default(),
            reload: reload::Work::default(),
            shell: shell::Work::new(
                std::env::current_dir().map_err(|e| view::Error::WorkingDirectory(e.kind()))?,
            ),
            ai: ai::Work::new(
                std::env::current_dir().map_err(|e| view::Error::WorkingDirectory(e.kind()))?,
            ),
            lsp,
            installation: Default::default(),
            installation_notice: String::new(),
            completion_policy,
            completion: None,
            completion_generation: 0,
            completion_request: None,
            completion_state: None,
            completion_source: None,
            completion_work: crate::completion::job::Work::default(),
            completion_resolve: None,
            completion_recent: Vec::new(),
            bookmark_visit: None,
            bookmark_storage,
            bookmark_list: None,
            macro_draft: None,
            macro_menu: None,
            bookmark_iteration: None,
            buffers,
            org: org::Work::default(),
            capture: None,
            block_request: None,
            source_edit: None,
            tangling: None,
            execution: None,
            feeds: None,
            preview_request: None,
            feed_results: Vec::new(),
            execution_result: None,
            tangling_results: Vec::new(),
            refile: None,
            refile_result: Vec::new(),
            refile_label: "Refile",
            links: links::State::default(),
            document_request: None,
            table_request: None,
            export_request: None,
            publishing: None,
            publishing_results: Vec::new(),
            attachment: None,
            attachment_results: std::collections::VecDeque::new(),
            windows,
            tree,
            kills,
            rectangle: None,
            visual_source: None,
            one_normal: None,
            repeat: repeat::State::default(),
            macros: Default::default(),
            yank: None,
            selection: None,
            mouse_enabled,
            input,
            last_find: None,
            last_search: None,
            size,
            layout_pending: false,
            layout: layout::Work::default(),
            echo: "C-x C-c: exit".into(),
            echo_kind: Message::Normal,
            pending: None,
            command_origin: None,
            task_clock: None,
            org_clock: None,
            active_clock: None,
            retained_clock: None,
            clock_store: None,
            clock_recovery_pin: None,
            recent_clocks: std::collections::VecDeque::new(),
            calendar_date: None,
            timestamp_repeat: None,
            timer: crate::org::time::Timer::default(),
            timer_clock: None,
            timer_notice: false,
            habit_agenda: None,
            agenda_sources: None,
            agenda_run: None,
            agenda_results: Vec::new(),
            agenda_marks: agenda_actions::Marks::default(),
            agenda_results_view: None,
            calendar_agenda: None,
            query_agenda: None,
            agenda_query_plan: None,
            agenda_custom_plan: None,
        })
    }

    pub fn empty(size: Size) -> Result<Self, view::Error> {
        let buffers = Buffers::new()?;
        let view = View::new(buffers.get(buffers.initial())?);
        Self::new(buffers, view, size)
    }

    pub fn buffer(&self) -> Result<&Buffer, buffer::Error> {
        self.buffers.get(self.windows.selected().point.stamp.buffer)
    }

    pub fn view(&self) -> &View {
        self.windows.selected()
    }

    pub fn echo(&self) -> &str {
        &self.echo
    }

    fn prepare_frame_layout(
        &mut self,
        budget: &mut view::LayoutBudget,
    ) -> Result<bool, view::Error> {
        let geometry = self.windows.geometry(self.editing_size());
        self.layout_pending = false;
        if let Some(geometry) = geometry {
            for (id, area) in geometry.panes() {
                let view = self.windows.get(*id)?;
                let buffer = self.buffers.get(view.point.stamp.buffer)?;
                if !view.prepare_layout(buffer, area.view_size(), budget)? {
                    self.layout_pending = true;
                    self.cancel_pointer_capture();
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn layout_scopes(
        &self,
        mut work: crate::coordinates::Work,
    ) -> Result<(Vec<crate::coordinates::Scope>, Vec<view::LayoutScope>), view::Error> {
        for (_, view) in self.windows.iter() {
            let buffer = self.buffers.get(view.point.stamp.buffer)?;
            if let Some(current) = crate::coordinates::Coordinates::new(buffer)?.work() {
                work = current;
                break;
            }
        }
        let mut sources = std::collections::BTreeSet::new();
        let mut coordinates = Vec::new();
        let mut fragments = Vec::new();
        for (_, view) in self.windows.iter() {
            if sources.insert(view.point.stamp.buffer) {
                coordinates.push(crate::coordinates::Scope::new(
                    self.buffers.get(view.point.stamp.buffer)?,
                    work.clone(),
                ));
            }
            fragments.push(view.layout_scope(work.clone()));
        }
        Ok((coordinates, fragments))
    }

    /// Advance cold native projection by one bounded batch. An unfinished
    /// projection has no source cursor/hits; remaining_input owns its next wake.
    pub fn poll_frame(&mut self) -> Result<Frame, view::Error> {
        if self.startup_screen.is_some() {
            self.refresh_frame()?;
            return self.startup_frame();
        }
        if self.git_visible() {
            self.refresh_frame()?;
            if self.git_visible() {
                return self.git_frame();
            }
        }
        if self.layout_active() {
            return self.pending_frame();
        }
        // Observe completed source work before installing a derived-work budget.
        // Its canonical callbacks must never receive a projection-only yield.
        self.refresh_frame()?;
        if self.layout_active() {
            return self.pending_frame();
        }
        let mut budget = view::LayoutBudget::new();
        if !self.prepare_frame_layout(&mut budget)? {
            return self.pending_frame();
        }
        let work = std::sync::Arc::new(std::sync::Mutex::new(budget));
        let scopes = self.layout_scopes(work)?;
        let result = self.project_frame();
        drop(scopes);
        match result {
            Err(error) if error.pending() => {
                self.layout_pending = true;
                self.cancel_pointer_capture();
                self.pending_frame()
            }
            result => result,
        }
    }

    fn pending_frame(&self) -> Result<Frame, view::Error> {
        Frame::pending(self.buffer()?, self.size, self.buffers.palette())
    }

    /// Produce a complete synchronous observation for an embedding caller.
    /// The native terminal uses poll_frame so cold anchor work can yield.
    pub fn frame(&mut self) -> Result<Frame, view::Error> {
        self.refresh_frame()?;
        self.project_frame()
    }

    fn refresh_frame(&mut self) -> Result<(), view::Error> {
        // Canonical completion precedes the derived layout budget. Embedding
        // hosts may observe ready jobs through frame() without synthesizing input.
        self.expire_shell();
        self.expire_git();
        if self.startup_screen.is_none() {
            self.resize_terminal();
        }
        self.expire_terminals();
        self.invalidate_completion();
        for (_, view) in self.windows.iter() {
            if self.completion_work.owns(view.point.stamp.buffer)
                && self
                    .buffers
                    .get(view.point.stamp.buffer)?
                    .syntax()
                    .working()
            {
                continue;
            }
            self.buffers
                .get_mut(view.point.stamp.buffer)?
                .refresh_syntax();
        }
        self.refresh_org();
        self.advance_agenda_action();
        self.advance_capture();
        self.invalidate_search();
        self.invalidate_replace();
        Ok(())
    }

    fn project_frame(&mut self) -> Result<Frame, view::Error> {
        if self.startup_screen.is_some() {
            return self.startup_frame();
        }
        if self.git_visible() {
            return self.git_frame();
        }
        let composition = self.composition();
        let candidate_rows = composition
            .as_ref()
            .and_then(|composition| composition.auxiliary())
            .map_or(0, |panel| panel.content().height());
        self.input.reveal_commands(candidate_rows);
        self.input.reveal_names(candidate_rows);
        self.reveal_paths(candidate_rows);
        self.reveal_workspace(composition.as_ref())?;
        let mut frame = match &composition {
            Some(composition) => Frame::compose_workspace(
                &self.buffers,
                &self.windows,
                &self.tree,
                composition,
                &self.echo,
            )?,
            None => Frame::compose(&self.buffers, &self.windows, self.size, &self.echo)?,
        };
        self.native_overlays(&mut frame, composition.as_ref())?;
        frame.message_kind(self.echo_kind);
        self.highlight_search(&mut frame, composition.as_ref())?;
        self.prompt_frame(&mut frame, composition.as_ref())?;
        self.completion_frame(&mut frame, composition.as_ref())?;
        self.retire_frame_work();
        self.layout_pending = false;
        Ok(frame)
    }

    fn native_overlays(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        self.modal_frame(frame, composition)?;
        self.activity_frame(frame)?;
        self.habit_frame(frame, composition)?;
        self.agenda_frame(frame, composition)?;
        self.query_frame(frame, composition)?;
        self.agenda_results_frame(frame, composition)?;
        self.bookmark_list_frame(frame, composition)?;
        self.macro_menu_frame(frame, composition)?;
        self.terminal_frame(frame, composition)
    }

    fn retire_frame_work(&self) {
        // A fully painted origin must not discard the deferred request's
        // offscreen segmentation or counted viewport continuation.
        if self.layout_active() {
            return;
        }
        for (_, view) in self.windows.iter() {
            view.cancel_layout();
        }
    }

    fn reveal_workspace(
        &mut self,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        // Completion borrows presentation rows. It cannot scroll the retained
        // editing/tree workspace while the query owns the visible cursor.
        if self.input.names().is_some()
            || self.input.palette().is_some()
            || self.path_navigation().is_some()
            || self.timestamp_calendar().is_some()
        {
            return Ok(());
        }
        let tree = composition.and_then(|composition| composition.tree());
        self.tree
            .reveal(tree.map_or(0, |panel| panel.content().height()));
        let size = composition.map_or(self.size, |composition| composition.editing_size());
        self.windows.reveal(&self.buffers, size)?;
        Ok(())
    }

    fn prompt_frame(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        self.input_frame(frame, composition)?;
        if let Some(Pending::Inventory(selected)) = &self.pending {
            frame.inventory(&self.buffers, *selected, false)?;
        }
        if let Some(Pending::Terminals(selected)) = &self.pending {
            frame.inventory(&self.buffers, *selected, true)?;
        }
        if self.input.focus() == crate::input::Focus::Decision {
            frame.decision();
        }
        Ok(())
    }

    fn input_frame(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        let Some(prompt) = self.input.prompt() else {
            return Ok(());
        };
        if let Some(path) = self.path_navigation() {
            return frame.paths(prompt, path, composition, &self.echo);
        }
        if let Some(names) = self.input.names() {
            return frame.names(names, composition, &self.echo);
        }
        if let Some(palette) = self.input.palette() {
            return frame.commands(palette, composition, &self.echo);
        }
        if let Some(date) = self.timestamp_calendar() {
            return frame.calendar(prompt, date, composition, &self.echo);
        }
        frame.prompt(prompt, &self.echo)
    }

    fn draw(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        let frame = self.poll_frame().map_err(io::Error::other)?;
        if let Some(style) = frame.cursor_style() {
            terminal.set_cursor_style(style);
        } else if let Some(state) = self.editing_state() {
            let style = match state {
                crate::input::EditingState::Normal
                | crate::input::EditingState::VisualCharacter
                | crate::input::EditingState::VisualLine
                | crate::input::EditingState::VisualBlock => {
                    crossterm::cursor::SetCursorStyle::SteadyBlock
                }
                crate::input::EditingState::Insert => crossterm::cursor::SetCursorStyle::SteadyBar,
                crate::input::EditingState::Replace => {
                    crossterm::cursor::SetCursorStyle::SteadyUnderScore
                }
            };
            terminal.set_cursor_style(style);
        } else {
            terminal.set_cursor_style(crossterm::cursor::SetCursorStyle::DefaultUserShape);
        }
        terminal.draw(&frame)
    }

    fn outcome(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        self.qualify_one_normal();
        let complete = matches!(
            &outcome,
            Outcome::Argument(_)
                | Outcome::PathAccept
                | Outcome::Decision(_)
                | Outcome::Search(crate::input::SearchAction::Accept)
                | Outcome::Message(_)
        );
        if complete {
            self.start_one_normal();
        }
        let action = self.apply_outcome(outcome, now);
        if complete {
            self.complete_one_normal();
        }
        action
    }

    fn apply_outcome(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        if self.macro_menu_active() && matches!(outcome, Outcome::Paste(_)) {
            return self.message("Read-only macro menu; q returns to source");
        }
        if self.bookmark_list_active() && matches!(outcome, Outcome::Paste(_)) {
            return self.message("Read-only bookmark list; q returns to source");
        }
        match outcome {
            Outcome::Search(action) => self.search_action(action),
            Outcome::Command(command) => self.command_outcome(command),
            Outcome::Invocation(invocation) => {
                self.retire_command();
                self.execute_invocation(*invocation)
            }
            Outcome::TextInvocation(invocation) => self.execute_argument_text(*invocation),
            Outcome::ArgumentPrefix(_) => self.argument_feedback(),
            _ => self.argument_outcome(outcome, now),
        }
    }

    fn argument_outcome(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        match outcome {
            Outcome::Argument(path) => self.argument(path),
            Outcome::PathAccept => self.accept_path(),
            Outcome::Decision(accepted) => self.decision(accepted),
            Outcome::MacroStep(answer) => self.macro_step_answer(answer),
            Outcome::MacroQuery(answer) => self.macro_query_answer(answer),
            Outcome::MacroKey(sequence) if matches!(self.pending, Some(Pending::MacroStepKey)) => {
                self.pending = None;
                self.macro_step_key(sequence)
            }
            Outcome::MacroKey(sequence)
                if matches!(self.pending, Some(Pending::MacroDefinitionKey)) =>
            {
                self.pending = None;
                self.macro_definition_key(sequence)
            }
            Outcome::MacroKey(sequence) => self.macro_key_argument(sequence),
            Outcome::Selection(selection) => self.select_candidate(selection),
            Outcome::Complete => self.complete_argument(),
            Outcome::Changed => self.input_changed(),
            Outcome::Paste(text) => self.habit_paste(&text),
            _ => self.input_feedback(outcome, now),
        }
    }

    fn input_feedback(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        if let Some(message) = self.blocked_input(&outcome) {
            return self.message(message);
        }
        match outcome {
            Outcome::Pointer(event) => self.pointer(event, now),
            Outcome::Editing(key) => self.typing(key),
            Outcome::Prefix(prefix) => self.prefix_notice(prefix),
            Outcome::Message(message) => self.notice(message),
            _ => Action::Wait,
        }
    }

    fn blocked_input(&self, outcome: &Outcome) -> Option<&'static str> {
        if self.macro_menu_active()
            && matches!(
                outcome,
                Outcome::Pointer(_)
                    | Outcome::Editing(_)
                    | Outcome::Paste(_)
                    | Outcome::TextInvocation(_)
            )
        {
            return Some("Read-only macro menu; q returns to source");
        }

        if self.bookmark_list_active()
            && matches!(outcome, Outcome::Pointer(_) | Outcome::Editing(_))
        {
            return Some("Read-only bookmark list; q returns to source");
        }
        if (self.habit_agenda.is_some() || self.agenda_active())
            && matches!(outcome, Outcome::Pointer(_) | Outcome::Editing(_))
        {
            return Some("Read-only Org agenda; C-g returns to source");
        }
        if self.tangle_input_blocked(outcome) {
            return Some("Source output owns input; C-g cancels remaining work");
        }
        if self.capture_blocks_input() {
            return Some("Capture input pending; C-g cancels");
        }
        None
    }

    /// Executes the same implemented command enum produced by key bindings.
    pub fn execute(&mut self, command: Command) -> Action {
        let command = command.canonical();
        if let Some(action) = self.macro_menu_command(command) {
            return action;
        }
        if let Some(action) = self.macro_definition_guard(command) {
            return action;
        }
        if let Some(action) = self.macro_command_guard(command) {
            return action;
        }
        if let Command::Macro(control) = command {
            return self.macro_command(control, Default::default());
        }
        if let Command::MacroNamed(name) = command {
            return self.macro_named_call(name, Default::default());
        }
        if let Command::MacroBinding(id) = command {
            return self.macro_bound_call(id, Default::default());
        }
        if command.prefix_control() {
            return self.execute_prefix_control(command);
        }
        if command == Command::KeyboardQuit && self.cancel_context_operation() {
            return self.message("Project preparation canceled; live sources retained");
        }
        if self.context_storage.locked() {
            return self.message("Context state publication is pending; live sources retained");
        }
        self.remember_context_position();
        if let Some(action) = self.startup_command(command) {
            return action;
        }
        if let Some(action) = self.git_command(command) {
            return action;
        }
        if let Some(action) = self.terminal_copy_command(command) {
            return action;
        }
        if let Some(action) = self.terminal_source_guard(command) {
            return action;
        }
        self.shell_interaction();
        if let Some(action) = self.defer_layout_command(command) {
            return action;
        }
        let action = self.execute_now(command);
        self.follow_tree();
        self.retire_context_preparation();
        self.activate_lsp();
        self.observe_files();
        action
    }

    fn execute_now(&mut self, command: Command) -> Action {
        if let Some(action) = self.git_command(command) {
            return action;
        }
        if let Some(action) = self.terminal_copy_command(command) {
            return action;
        }
        if let Some(action) = self.terminal_source_guard(command) {
            return action;
        }
        if self.terminals.focused {
            return self.execute_operation(command);
        }
        self.qualify_one_normal();
        if command != Command::Modal(crate::command::ModalCommand::Once) {
            self.start_one_normal();
        }
        let repeat = self.before_repeat_command(command);
        let action = self.execute_operation(command);
        self.after_repeat_command(repeat);
        if command != Command::Modal(crate::command::ModalCommand::Once) {
            self.complete_one_normal();
        }
        action
    }

    fn execute_operation(&mut self, command: Command) -> Action {
        if let Some(action) = self.completion_boundary(command) {
            return action;
        }
        if let Some(action) = self.cancel_extended(command) {
            return action;
        }
        if let Command::Terminal(command) = command {
            return self.terminal_command(command);
        }
        self.bookmark_visit = None;
        if !matches!(
            command,
            Command::Bookmark(
                crate::command::BookmarkCommand::Next | crate::command::BookmarkCommand::Previous
            ) | Command::ExecuteExtended
        ) {
            self.bookmark_iteration = None;
        }
        self.timestamp_boundary(command);
        self.cancel_pointer_capture();
        if let Some(action) = self
            .bookmark_list_command(command)
            .or_else(|| self.source_edit_command(command))
            .or_else(|| self.capture_command(command))
            .or_else(|| self.execution_command(command))
            .or_else(|| self.feed_command(command))
            .or_else(|| self.preview_command(command))
            .or_else(|| self.tangle_command(command))
            .or_else(|| self.agenda_action_command(command))
            .or_else(|| self.prepare_command(command))
        {
            return action;
        }
        if let Some(action) = self.layout_command(command) {
            return action;
        }
        self.command_boundary(command);
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        let action = match self
            .editing_dispatch(command)
            .or_else(|| self.navigation_dispatch(command))
            .or_else(|| self.modal_dispatch(command))
            .or_else(|| self.lsp_command(command))
            .or_else(|| self.query_view_command(command))
            .or_else(|| self.agenda_view_command(command))
            .or_else(|| self.habit_command(command))
        {
            Some(action) => action,
            None => self.active_command(command),
        };
        self.sync_input_context();
        action
    }

    fn active_command(&mut self, command: Command) -> Action {
        match command {
            Command::Bookmark(command) => self.bookmark_command(command),
            Command::QueryReplace => self.query_replace(),
            Command::GotoLine => self.goto_line(),
            Command::SearchForward => self.search(crate::search::Direction::Forward),
            Command::SearchBackward => self.search(crate::search::Direction::Backward),
            Command::Split(direction) => self.split(direction),
            Command::Window(command) => self.window_command(command),
            Command::Kill(command) => self.kill_command(command),
            Command::Linear(command) => self.linear_command(command),
            _ => self.selection_command(command),
        }
    }

    fn selection_command(&mut self, command: Command) -> Action {
        match command {
            Command::Tree(command) => self.tree_command(command),
            Command::Rectangle(command) => self.rectangle_command(command),
            Command::Region(command) => self.region_command(command),
            Command::Whitespace(command) => self.whitespace_command(command),
            Command::Gutter(command) => self.gutter_command(command),
            Command::MouseMode => self.toggle_mouse(),
            Command::VisualLineMode => self.toggle_wrap(),
            Command::Mode(mode) => self.select_mode(mode),
            Command::Org(command) => self.org_command(command),
            _ => self.dispatch(command),
        }
    }

    fn dispatch(&mut self, command: Command) -> Action {
        self.navigation(command);
        match command {
            Command::Plugin(command) => self.plugin_command(command),
            Command::PluginDefined(name) => self.plugin_defined(name),
            Command::Quit => self.quit(),
            Command::FindFile => self.find_file(),
            Command::GenerateNewBuffer => self.generate_new_buffer(),
            Command::OpenDirectory => self.open_directory(),
            Command::ContextClose => self.close_context(),
            Command::DashboardOpen => self.dashboard_open(),
            Command::ProjectCreate => self.project_create(),
            Command::ProjectPromote => self.project_promote(),
            Command::ProjectSwitch | Command::ProjectList => self.project_list(),
            Command::WorkspaceCreate => self.workspace_create(),
            Command::WorkspaceOpen | Command::WorkspaceList => self.workspace_list(),
            Command::WorkspaceAddDirectory => self.workspace_add_directory(),
            Command::WorkspaceAddProject => self.workspace_add_project(),
            Command::WorkspaceRemoveRoot => self.workspace_remove_root(),
            command @ (Command::ProjectRename
            | Command::ProjectForget
            | Command::ProjectSetRoot
            | Command::ProjectPromoteToWorkspace
            | Command::ProjectDuplicateAsWorkspace
            | Command::WorkspaceRename
            | Command::WorkspaceForget
            | Command::WorkspaceSetRootPath) => self.manage_context(command),
            Command::SaveBuffer => self.save_buffer(),
            Command::RevertBuffer => self.revert_buffer(),
            Command::AiSaveCommand => self.ai_save_command(),
            Command::AiDefined(name) => self.ai_defined(name),
            Command::MacroNamed(name) => self.macro_named_call(name, Default::default()),
            Command::MacroBinding(id) => self.macro_bound_call(id, Default::default()),
            Command::AiExplain => self.ai_command(ai::Kind::Explain),
            Command::AiPrompt => self.ai_command(ai::Kind::Prompt),
            Command::AiImplement => self.ai_command(ai::Kind::Implement),
            Command::AiApplyResult => self.ai_apply_result(),
            Command::AiDiscardResult => self.ai_discard_result(),
            Command::AiSetDirectory => self.ai_set_directory(),
            Command::AiSelectModel => self.ai_select_model(),
            Command::AiResetSession => self.ai_reset_session(),
            Command::AiDisconnect => self.ai_disconnect(),
            Command::AiCancel => self.ai_cancel_command(),
            Command::ShellCommand => self.shell_command(crate::shell::Kind::Command),
            Command::ShellCommandOnRegion => self.shell_command(crate::shell::Kind::Region),
            Command::ShellFilterRegion => self.shell_command(crate::shell::Kind::Filter),
            Command::ShellCommandInsert => self.shell_command(crate::shell::Kind::Insert),
            Command::ShellCommandCancel => self.shell_cancel(),
            Command::ShellCommandOutput => self.shell_output(),
            Command::WriteFile => self.write_file(),
            Command::Buffer(command) => self.buffer_command(command),
            Command::Motion(motion) => self.motion(motion),
            Command::Scroll(scroll) => self.scroll(scroll),
            Command::History(history) => self.history(history),
            Command::KeyboardQuit => self.keyboard_quit(),
            _ => self.edit_command(command),
        }
    }

    fn command_boundary(&mut self, command: Command) {
        if self.input.focus() == crate::input::Focus::Editing
            && !self.tree.focused()
            && !matches!(command, Command::Tree(_))
        {
            self.tree.edit();
        }
        self.echo_kind = Message::Normal;
        if !matches!(command, Command::Org(_)) {
            self.org.cancel_request();
        }
        if !command.is_kill() {
            self.kills.break_chain();
        }
        if command != Command::Linear(crate::command::LinearCommand::Pop) {
            self.yank = None;
        }
    }

    fn edit_command(&mut self, command: Command) -> Action {
        let result = match command {
            Command::Newline => self.line_break(),
            Command::InsertTab => self.indent(),
            Command::BackwardDeleteChar => self.delete(true),
            Command::DeleteChar => self.delete(false),
            _ => return self.message("Command is not a text edit"),
        };
        self.finish(result)
    }

    fn navigation(&mut self, command: Command) {
        if matches!(
            command,
            Command::Quit
                | Command::Scroll(_)
                | Command::FindFile
                | Command::GotoLine
                | Command::SaveBuffer
                | Command::WriteFile
                | Command::Buffer(_)
        ) {
            return;
        }
        if command == Command::KeyboardQuit && self.pending.take().is_some() {
            return;
        }
        self.windows.selected_mut().reset_navigation();
    }

    /// Inserts one printable scalar, retaining a valid grapheme cursor after
    /// joins. Control bytes can enter only through explicit implemented commands.
    pub fn insert(&mut self, character: char) -> Action {
        if self.context_storage.locked() {
            return self.message("Context publication owns source admission");
        }
        if self.git_visible() {
            return self.message("Git presentation is read-only");
        }
        if self.terminal_copy_active() {
            return self.error("Terminal copy text is read-only");
        }
        if self.terminals.focused {
            if self.input.focus() != crate::input::Focus::Editing || self.pending.is_some() {
                return self.error("A host prompt owns input focus");
            }
            if character.is_control() {
                return self.error("Control character rejected");
            }
            return self.terminal_send(character.to_string().as_bytes());
        }
        self.dismiss_completion();
        if self.bookmark_list_active() {
            return self.message("Read-only bookmark list; q returns to source");
        }
        self.cancel_pointer_capture();
        if self.capture_blocks_input() {
            return self.message("Capture input pending; C-g cancels");
        }
        if self.tree.focused() {
            return self.message("File tree owns input focus");
        }
        self.yank = None;
        self.kills.break_chain();
        self.windows.selected_mut().reset_navigation();
        if character.is_control() {
            return self.notice("Control character rejected");
        }
        let mut bytes = [0; 4];
        self.tree.edit();
        let text = character.encode_utf8(&mut bytes);
        let result = self.type_text(text);
        self.finish(result)
    }

    fn typing(&mut self, key: Key) -> Action {
        self.timestamp_repeat = None;
        if let KeyCode::Char(character) = key.code
            && key.modifiers.is_empty()
        {
            return self.insert(character);
        }
        self.notice("Unbound key")
    }

    fn stop_typing(&mut self) -> Result<(), buffer::Error> {
        self.buffers
            .get_mut(self.windows.selected().point.stamp.buffer)
            .map(Buffer::break_typing)
    }

    fn notice(&mut self, message: &str) -> Action {
        self.retire_command();
        self.yank = None;
        self.kills.break_chain();
        match self.stop_typing() {
            Ok(()) => {
                let action = self.focus_message(message);
                self.echo_kind = Message::Warning;
                action
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn prefix_notice(&mut self, prefix: crate::input::Prefix) -> Action {
        let label = self
            .input
            .prefix_label()
            .unwrap_or_else(|| prefix.label().into());
        if prefix == crate::input::Prefix::Meta || self.input.editing_composition() {
            return self.focus_message(&label);
        }
        self.notice(&label)
    }

    fn resize(&mut self, width: u16, height: u16) -> Action {
        self.size = Size { width, height };
        if self.startup_screen.is_some() {
            return Action::Redraw;
        }
        let terminal = self.resize_terminal();
        if terminal != Action::Wait {
            return terminal;
        }
        if self.tree.resize(self.size) {
            self.focus_message("File tree hidden at this width");
        }
        match self.stop_typing() {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn finish(&mut self, result: Result<bool, buffer::edit::Error>) -> Action {
        match result {
            Ok(true) => self.message(""),
            Ok(false) => {
                if self.macros.active() && (self.macros.expanded || self.macros.dispatching) {
                    self.error("Buffer boundary")
                } else {
                    self.message("Buffer boundary")
                }
            }
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn message(&mut self, message: &str) -> Action {
        self.echo_kind = Message::Normal;
        self.echo = message.into();
        Action::Redraw
    }

    fn error(&mut self, message: &str) -> Action {
        let active_macro =
            self.macros.active() && (self.macros.expanded || self.macros.dispatching);
        let budget =
            self.macros.runner.as_ref().is_some_and(|run| {
                run.budget.load(std::sync::atomic::Ordering::Relaxed) == usize::MAX
            });
        if active_macro {
            self.restore_search();
            self.pending = None;
            self.input.cancel();
            self.buffers.automatic_group(None);
            self.macros.fail(if budget {
                crate::macros::StopReason::TextBudget
            } else {
                crate::macros::StopReason::NativeFailure
            });
        }
        let detail = self
            .macros
            .stop
            .filter(|stop| active_macro && stop.iteration > 0)
            .map_or(String::new(), |stop| {
                format!(" (macro iteration {}, step {})", stop.iteration, stop.step)
            });
        let message = if budget {
            "Keyboard macro generated text limit reached"
        } else {
            message
        };
        let action = self.message(&format!("{message}{detail}"));
        self.echo_kind = Message::Error;
        action
    }
}
