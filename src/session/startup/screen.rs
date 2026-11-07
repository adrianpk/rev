use super::super::{Action, Session};
use crate::command::{Command, ModalCommand, Motion, Scroll, navigation::Target};
use crate::display::Frame;
use crate::input::{Focus, Selection};
use crate::pointer::Kind;
use crate::search::Direction;
use crate::view::Size;
use crossterm::event::Event;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Choice {
    NewBuffer,
    Open,
    OpenProject,
    OpenWorkspace,
}
impl Choice {
    const ALL: [Self; 4] = [
        Self::NewBuffer,
        Self::Open,
        Self::OpenProject,
        Self::OpenWorkspace,
    ];
    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|v| *v == self)
            .expect("startup action is admitted")
    }
    fn command(self) -> Command {
        match self {
            Self::NewBuffer => Command::GenerateNewBuffer,
            Self::Open => Command::FindFile,
            Self::OpenProject => Command::ProjectSwitch,
            Self::OpenWorkspace => Command::WorkspaceOpen,
        }
    }
}
pub(in crate::session) struct Screen {
    selected: Choice,
    armed: Option<(Choice, Size)>,
}
impl Default for Screen {
    fn default() -> Self {
        Self {
            selected: Choice::NewBuffer,
            armed: None,
        }
    }
}
impl Session {
    /// The startup screen retains the live context; it owns no buffer or durable
    /// registration. Embedded hosts opt into it explicitly through this command.
    pub fn dashboard_visible(&self) -> bool {
        self.startup_screen.is_some()
    }
    pub(in crate::session) fn dashboard_open(&mut self) -> Action {
        if self.startup_screen.is_some() {
            return self.message("Startup screen is already open");
        }
        if self.context_storage.busy()
            || self.context_prepared.is_some()
            || self.context_management.is_some()
            || self.context_membership.is_some()
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
            || self.layout_active()
            || self.git_shutdown_active()
            || self.terminal_shutdown_active()
        {
            return self.error("Finish or cancel the current interaction before dashboard-open");
        }
        if self.capture.is_some()
            || self.source_edit.is_some()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
        {
            return self
                .error("A source draft or output owns this context; finish or cancel it first");
        }
        self.dismiss_completion();
        self.cancel_pointer_capture();
        self.cancel_terminal_pointer();
        self.layout_pending = false;
        self.startup_screen = Some(Screen::default());
        self.sync_input_context();
        self.message("Arrows select; Enter opens; M-x or modal colon runs commands; C-g returns")
    }
    pub(in crate::session) fn leave_dashboard(&mut self) {
        self.startup_screen = None;
        self.input.set_startup(false);
    }
    pub(in crate::session) fn startup_new_buffer(&mut self) -> Action {
        match self.prepare_context(None) {
            Ok(r) => self.begin_context(r),
            Err(e) => self.error(&e.to_string()),
        }
    }
    pub(in crate::session) fn startup_selection(&mut self, selection: Selection) -> Action {
        if selection == Selection::Accept {
            self.input.suppress_repeat();
            let Some(screen) = &self.startup_screen else {
                return Action::Wait;
            };
            let command = screen.selected.command();
            return self.execute(command);
        }
        let Some(screen) = &mut self.startup_screen else {
            return Action::Wait;
        };
        let old = screen.selected.index();
        let next = match selection {
            Selection::Next => (old + 1).min(Choice::ALL.len() - 1),
            Selection::Previous => old.saturating_sub(1),
            Selection::First => 0,
            Selection::Last => Choice::ALL.len() - 1,
            Selection::Accept => old,
        };
        screen.selected = Choice::ALL[next];
        screen.armed = None;
        Action::Redraw
    }
    pub(in crate::session) fn startup_command(&mut self, command: Command) -> Option<Action> {
        if command == Command::DashboardOpen {
            return Some(self.dashboard_open());
        }
        self.startup_screen.as_ref()?;
        if let Some(screen) = &mut self.startup_screen {
            screen.armed = None;
        }
        if command == Command::KeyboardQuit || command == Command::Modal(ModalCommand::Normal) {
            let nested = self.pending.is_some() || self.context_prepared.is_some();
            self.cancel_quit();
            self.context_prepared = None;
            self.context_membership = None;
            self.context_management = None;
            self.pending = None;
            self.command_origin = None;
            self.input.cancel();
            if !nested {
                self.leave_dashboard();
                self.sync_input_context();
            }
            return Some(self.message(if nested {
                "Selection canceled; startup screen retained"
            } else {
                "Returned to retained context"
            }));
        }
        let selection = match command {
            Command::Scroll(Scroll::NextLine | Scroll::PageForward) => Some(Selection::Next),
            Command::Scroll(Scroll::PreviousLine | Scroll::PageBackward) => {
                Some(Selection::Previous)
            }
            Command::Motion(Motion::BufferStart | Motion::LineStart) => Some(Selection::First),
            Command::Motion(Motion::BufferEnd | Motion::LineEnd) => Some(Selection::Last),
            Command::Navigate(n) => match n.target {
                Target::Line(Direction::Forward) => Some(Selection::Next),
                Target::Line(Direction::Backward) => Some(Selection::Previous),
                Target::Buffer(Direction::Backward) | Target::LineStart => Some(Selection::First),
                Target::Buffer(Direction::Forward) | Target::LineEnd => Some(Selection::Last),
                _ => None,
            },
            Command::Newline => Some(Selection::Accept),
            _ => None,
        };
        if let Some(selection) = selection {
            return Some(self.startup_selection(selection));
        }
        Some(match command {
            Command::ExecuteExtended => self.extended(),
            Command::Quit => self.quit(),
            Command::MouseMode => self.toggle_mouse(),
            Command::GenerateNewBuffer => self.generate_new_buffer(),
            Command::FindFile => self.find_file(),
            Command::OpenDirectory => self.open_directory(),
            Command::ContextClose => self.close_context(),
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
            _ => {
                self.leave_dashboard();
                self.sync_input_context();
                return None;
            }
        })
    }
    pub(in crate::session) fn startup_input(&mut self, event: Event, now: Instant) -> Action {
        if let Event::Mouse(mouse) = event {
            if self.input.focus() != Focus::Editing || self.pending.is_some() || !self.mouse_enabled
            {
                return Action::Wait;
            }
            return crate::pointer::Event::from_mouse(mouse)
                .map_or(Action::Wait, |event| self.startup_pointer(event));
        }
        if let Some(screen) = &mut self.startup_screen {
            screen.armed = None;
        }
        let named = matches!(self.pending, Some(super::super::Pending::Command));
        let outcome = self.input.handle_at(event, now);
        if let crate::input::Outcome::Command(command) = &outcome
            && !named
            && !crate::input::Input::startup_command(*command)
        {
            return self
                .message("Startup screen is read-only; select an action or a named command");
        }
        if matches!(
            outcome,
            crate::input::Outcome::Paste(_) | crate::input::Outcome::Editing(_)
        ) {
            return self.message("Startup screen is read-only; select an action");
        }
        self.dispatch_completion_outcome(outcome, now)
    }
    fn startup_pointer(&mut self, event: crate::pointer::Event) -> Action {
        if !event.modifiers.is_empty() {
            return Action::Wait;
        }
        let screen = self
            .startup_screen
            .as_mut()
            .expect("startup pointer owns its screen");
        let layout = crate::display::startup::Layout::new(
            self.size,
            screen.selected.index(),
            startup_color(),
        );
        match event.kind {
            Kind::Press => {
                screen.armed = layout
                    .hit(event.position)
                    .map(|i| (Choice::ALL[i], self.size));
                Action::Redraw
            }
            Kind::Release => {
                let Some((choice, size)) = screen.armed.take() else {
                    return Action::Wait;
                };
                if size != self.size || layout.hit(event.position) != Some(choice.index()) {
                    return Action::Wait;
                }
                screen.selected = choice;
                self.startup_selection(Selection::Accept)
            }
            Kind::ScrollUp => self.startup_selection(Selection::Previous),
            Kind::ScrollDown => self.startup_selection(Selection::Next),
            _ => Action::Wait,
        }
    }
    pub(in crate::session) fn startup_frame(&mut self) -> Result<Frame, crate::view::Error> {
        let selected = self
            .startup_screen
            .as_ref()
            .expect("startup presentation owns its screen")
            .selected
            .index();
        let composition = self.composition();
        let rows = composition
            .as_ref()
            .and_then(|c| c.auxiliary())
            .map_or(0, |p| p.content().height());
        self.input.reveal_commands(rows);
        self.input.reveal_names(rows);
        self.reveal_paths(rows);
        let mut frame = Frame::pending(self.buffer()?, self.size, self.buffers.palette())?;
        frame.startup(selected, startup_color(), &self.echo)?;
        if self.size.height > 1 {
            frame.message_kind(self.echo_kind);
        }
        self.prompt_frame(&mut frame, composition.as_ref())?;
        Ok(frame)
    }
    pub(in crate::session) fn resize_startup(&mut self) {
        if let Some(s) = &mut self.startup_screen {
            s.armed = None;
        }
    }
}
fn startup_color() -> bool {
    !crossterm::style::Colored::ansi_color_disabled_memoized()
}
