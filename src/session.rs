mod agenda;
mod agenda_actions;
mod agenda_custom;
mod agenda_queries;
mod agenda_sources;
mod bookmarks;
mod buffers;
mod capture;
mod refile;
pub use refile::{ResultEntry as RefileResult, Status as RefileStatus};
mod attachment;
mod document;
mod edit;
mod events;
mod exports;
mod extended;
mod file;
mod goto;
mod gutter;
mod habits;
mod history;
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
mod mode;
mod motion;
mod org;
mod outline;
mod paste;
mod path;
mod pointer;
mod prompt;
mod quit;
mod rectangle;
mod region;
mod replace;
mod save;
mod search;
mod source_batch;
mod source_edit;
mod startup;
mod tangling;
mod timers;
mod tree;
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
    bookmark_visit: Option<bookmarks::Navigation>,
    bookmark_storage: bookmarks::Persistence,
    bookmark_list: Option<bookmarks::List>,
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
    rectangle: Option<crate::rectangle::Rectangle>,
    yank: Option<yank::Capture>,
    selection: Option<pointer::Selection>,
    mouse_enabled: bool,
    input: Input,
    size: Size,
    layout_pending: bool,
    layout: layout::Work,
    echo: String,
    echo_kind: Message,
    pending: Option<Pending>,
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
    Bookmark(bookmarks::Prompt),
    BookmarkReload(Box<bookmarks::Reload>),
    BookmarkQuit,
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
    /// The caller owns terminal acquisition, entry and restoration on return.
    pub fn run_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        let result = self.drive_terminal(terminal);
        self.end_timer_session();
        result
    }
    fn drive_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        self.start_terminal(terminal)?;
        loop {
            let action = self.read_input()?;
            terminal.set_mouse_reporting(self.mouse_enabled())?;
            match action {
                Action::Exit => return Ok(()),
                Action::Redraw => self.draw(terminal)?,
                Action::Wait => {}
            }
        }
    }

    fn start_terminal(&mut self, terminal: &mut Terminal<impl Write>) -> io::Result<()> {
        terminal.set_mouse_reporting(self.mouse_enabled())?;
        terminal.set_cursor_color(self.buffers.palette().cursor)?;
        self.draw(terminal)
    }

    /// Takes ownership of canonical buffers and a qualified initial view.
    pub fn new(buffers: Buffers, view: View, size: Size) -> Result<Self, view::Error> {
        let mut session = Self::from_buffers(buffers, view, size)?;
        session
            .tree
            .present()
            .map_err(|error| view::Error::WorkingDirectory(error.kind()))?;
        Ok(session)
    }

    fn from_buffers(buffers: Buffers, view: View, size: Size) -> Result<Self, view::Error> {
        let buffer = buffers.get(view.point.stamp.buffer)?;
        view.check_point(buffer)?;
        let tree = crate::tree::Tree::startup(
            &buffers.snapshot().settings().tree,
            buffer.file().map(|file| file.path()),
        )
        .map_err(|error| view::Error::WorkingDirectory(error.kind()))?;
        let windows = Windows::with_settings(view, buffers.snapshot())?;
        let kills = crate::kill::KillRing::with_settings(buffers.snapshot());
        let mut input = Input::with_settings(buffers.snapshot());
        input.set_org_context(buffer.is_org().then_some(crate::input::OrgContext::Editing));
        let mouse_enabled = buffers.snapshot().settings().mouse.enabled;
        let bookmark_storage = bookmarks::Persistence::configured(
            buffers.snapshot().settings().bookmarks.file.clone(),
        );
        Ok(Self {
            bookmark_visit: None,
            bookmark_storage,
            bookmark_list: None,
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
            yank: None,
            selection: None,
            mouse_enabled,
            input,
            size,
            layout_pending: false,
            layout: layout::Work::default(),
            echo: "C-x C-c: exit".into(),
            echo_kind: Message::Normal,
            pending: None,
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
        if self.layout_active() {
            return Frame::pending(self.buffer()?, self.size, self.buffers.palette());
        }
        // Observe completed source work before installing a derived-work budget.
        // Its canonical callbacks must never receive a projection-only yield.
        self.refresh_frame()?;
        if self.layout_active() {
            return Frame::pending(self.buffer()?, self.size, self.buffers.palette());
        }
        let mut budget = view::LayoutBudget::new();
        if !self.prepare_frame_layout(&mut budget)? {
            return Frame::pending(self.buffer()?, self.size, self.buffers.palette());
        }
        let work = std::sync::Arc::new(std::sync::Mutex::new(budget));
        let scopes = self.layout_scopes(work)?;
        let result = self.project_frame();
        drop(scopes);
        match result {
            Err(error) if error.pending() => {
                self.layout_pending = true;
                self.cancel_pointer_capture();
                Frame::pending(self.buffer()?, self.size, self.buffers.palette())
            }
            result => result,
        }
    }

    /// Produce a complete synchronous observation for an embedding caller.
    /// The native terminal uses poll_frame so cold anchor work can yield.
    pub fn frame(&mut self) -> Result<Frame, view::Error> {
        self.refresh_frame()?;
        self.project_frame()
    }

    fn refresh_frame(&mut self) -> Result<(), view::Error> {
        for (_, view) in self.windows.iter() {
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
        for (_, view) in self.windows.iter() {
            view.cancel_layout();
        }
        self.layout_pending = false;
        Ok(frame)
    }

    fn native_overlays(
        &self,
        frame: &mut Frame,
        composition: Option<&crate::panel::Composition>,
    ) -> Result<(), view::Error> {
        self.timer_frame(frame)?;
        self.habit_frame(frame, composition)?;
        self.agenda_frame(frame, composition)?;
        self.query_frame(frame, composition)?;
        self.agenda_results_frame(frame, composition)?;
        self.bookmark_list_frame(frame, composition)
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
            frame.inventory(&self.buffers, *selected)?;
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
        terminal.draw(&self.poll_frame().map_err(io::Error::other)?)
    }

    fn outcome(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        if self.bookmark_list_active() && matches!(outcome, Outcome::Paste(_)) {
            return self.message("Read-only bookmark list; q returns to source");
        }
        match outcome {
            Outcome::Search(action) => self.search_action(action),
            Outcome::Command(command) => self.command_outcome(command),
            Outcome::Argument(path) => self.argument(path),
            Outcome::PathAccept => self.accept_path(),
            Outcome::Decision(accepted) => self.decision(accepted),
            Outcome::Selection(selection) => self.select_candidate(selection),
            Outcome::Complete => self.complete_argument(),
            Outcome::Changed => self.input_changed(),
            Outcome::Paste(text) => self.habit_paste(&text),
            _ => self.input_feedback(outcome, now),
        }
    }

    fn input_feedback(&mut self, outcome: Outcome, now: std::time::Instant) -> Action {
        if self.bookmark_list_active()
            && matches!(outcome, Outcome::Pointer(_) | Outcome::Editing(_))
        {
            return self.message("Read-only bookmark list; q returns to source");
        }
        if (self.habit_agenda.is_some() || self.agenda_active())
            && matches!(outcome, Outcome::Pointer(_) | Outcome::Editing(_))
        {
            return self.message("Read-only Org agenda; C-g returns to source");
        }
        if self.tangle_input_blocked(&outcome) {
            return self.message("Source output owns input; C-g cancels remaining work");
        }
        if self.capture_blocks_input() {
            return self.message("Capture input pending; C-g cancels");
        }
        match outcome {
            Outcome::Pointer(event) => self.pointer(event, now),
            Outcome::Editing(key) => self.typing(key),
            Outcome::Prefix(prefix) => self.prefix_notice(prefix),
            Outcome::Message(message) => self.notice(message),
            _ => Action::Wait,
        }
    }

    /// Executes the same implemented command enum produced by key bindings.
    pub fn execute(&mut self, command: Command) -> Action {
        if let Some(action) = self.defer_layout_command(command) {
            return action;
        }
        self.execute_now(command)
    }

    fn execute_now(&mut self, command: Command) -> Action {
        self.bookmark_visit = None;
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
            .query_view_command(command)
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
            Command::Quit => self.quit(),
            Command::FindFile => self.find_file(),
            Command::SaveBuffer => self.save_buffer(),
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
        if prefix == crate::input::Prefix::Meta {
            return self.focus_message(&label);
        }
        self.notice(&label)
    }

    fn resize(&mut self, width: u16, height: u16) -> Action {
        self.size = Size { width, height };
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
            Ok(false) => self.message("Buffer boundary"),
            Err(error) => self.error(&error.to_string()),
        }
    }

    fn message(&mut self, message: &str) -> Action {
        self.echo_kind = Message::Normal;
        self.echo = message.into();
        Action::Redraw
    }

    fn error(&mut self, message: &str) -> Action {
        let action = self.message(message);
        self.echo_kind = Message::Error;
        action
    }
}
