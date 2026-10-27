mod buffers;
mod edit;
mod events;
mod extended;
mod file;
mod history;
mod kill;
mod linear;
mod motion;
mod paste;
mod prompt;
mod quit;
mod rectangle;
mod region;
mod replace;
mod save;
mod search;
mod tree;
mod windows;
mod yank;

use std::io::{self, Write};
use std::path::Path;

use crossterm::event::KeyCode;

use crate::buffer::{self, Buffer, BufferId, Buffers, Confirmation, Kill};
use crate::command::Command;
use crate::display::{Frame, Size};
use crate::input::{Input, Key, Outcome};
use crate::terminal::Terminal;
use crate::view::{self, View};
use crate::window::Windows;

pub(crate) fn run(terminal: &mut Terminal<impl Write>, path: Option<&Path>) -> io::Result<()> {
    let mut session = initial(path)?;
    session.draw(terminal.writer())?;
    loop {
        match session.read_input()? {
            Action::Exit => return Ok(()),
            Action::Redraw => session.draw(terminal.writer())?,
            Action::Wait => {}
        }
    }
}

fn initial(path: Option<&Path>) -> io::Result<Session> {
    let (width, height) = crossterm::terminal::size()?;
    let mut session = Session::empty(Size { width, height }).map_err(io::Error::other)?;
    if let Some(path) = path {
        session.load(path)?;
        if let Some(file) = session.buffer().map_err(io::Error::other)?.file() {
            let path = file.path().to_path_buf();
            session.tree.initial(&path);
        }
    }
    Ok(session)
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
    buffers: Buffers,
    windows: Windows,
    tree: crate::tree::Tree,
    kills: crate::kill::KillRing,
    rectangle: Option<crate::rectangle::Rectangle>,
    yank: Option<yank::Capture>,
    input: Input,
    size: Size,
    echo: String,
    pending: Option<Pending>,
}

enum AfterSave {
    Stay,
    Close(Kill),
    Quit(quit::Quit),
}

enum Pending {
    Quit(quit::Quit),
    TreeDelete(crate::file::Deletion),
    TreeCreate(crate::tree::Creation),
    TreeRename(crate::file::Rename),
    Replace(Box<replace::State>),
    Search(Box<search::State>),
    Command,
    Argument(Command),
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
    /// Takes ownership of canonical buffers and a qualified initial view.
    pub fn new(buffers: Buffers, view: View, size: Size) -> Result<Self, view::Error> {
        view.cell(buffers.get(view.point.stamp.buffer)?)?;
        Ok(Self {
            buffers,
            windows: Windows::new(view)?,
            tree: crate::tree::Tree::default(),
            kills: crate::kill::KillRing::default(),
            rectangle: None,
            yank: None,
            input: Input::default(),
            size,
            echo: "C-x C-c: exit".into(),
            pending: None,
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

    pub fn frame(&mut self) -> Result<Frame, view::Error> {
        for (_, view) in self.windows.iter() {
            self.buffers
                .get_mut(view.point.stamp.buffer)?
                .refresh_syntax();
        }
        self.invalidate_search();
        self.invalidate_replace();
        self.tree.reveal(self.size);
        let size = self.editing_size();
        self.windows.reveal(&self.buffers, size)?;
        let mut frame = Frame::compose(&self.buffers, &self.windows, size, &self.echo)?;
        self.highlight_search(&mut frame)?;
        frame = frame.with_tree(&self.tree, self.size, &self.echo)?;
        self.prompt_frame(&mut frame)?;
        Ok(frame)
    }

    fn prompt_frame(&self, frame: &mut Frame) -> Result<(), view::Error> {
        if let Some(prompt) = self.input.prompt() {
            let hint = self.input.palette().map(crate::input::Palette::hint);
            let notice = hint
                .as_deref()
                .filter(|_| self.echo.is_empty())
                .unwrap_or(&self.echo);
            frame.prompt(prompt, notice)?;
        }
        if let Some(Pending::Inventory(selected)) = &self.pending {
            frame.inventory(&self.buffers, *selected)?;
        }
        if self.input.focus() == crate::input::Focus::Decision {
            frame.decision();
        }
        Ok(())
    }

    fn draw(&mut self, writer: &mut impl Write) -> io::Result<()> {
        self.frame().map_err(io::Error::other)?.write(writer)
    }

    fn outcome(&mut self, outcome: Outcome) -> Action {
        match outcome {
            Outcome::Search(action) => self.search_action(action),
            Outcome::Command(command) => self.command_outcome(command),
            Outcome::Argument(path) => self.argument(path),
            Outcome::Decision(accepted) => self.decision(accepted),
            Outcome::Selection(selection) => self.select_buffer(selection),
            Outcome::Complete => self.complete_argument(),
            Outcome::Changed => self.input_changed(),
            Outcome::Paste(text) => self.paste_text(&text),
            _ => self.input_feedback(outcome),
        }
    }

    fn input_feedback(&mut self, outcome: Outcome) -> Action {
        match outcome {
            Outcome::Editing(key) => self.typing(key),
            Outcome::Prefix(prefix) => self.prefix_notice(prefix),
            Outcome::Message(message) => self.notice(message),
            _ => Action::Wait,
        }
    }

    /// Executes the same implemented command enum produced by key bindings.
    pub fn execute(&mut self, command: Command) -> Action {
        if let Some(action) = self.prepare_command(command) {
            return action;
        }
        self.command_boundary(command);
        if let Err(error) = self.stop_typing() {
            return self.message(&error.to_string());
        }
        self.active_command(command)
    }

    fn active_command(&mut self, command: Command) -> Action {
        match command {
            Command::QueryReplace => self.query_replace(),
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
        let text = character.encode_utf8(&mut bytes);
        let result = self.type_text(text);
        self.finish(result)
    }

    fn typing(&mut self, key: Key) -> Action {
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
            Ok(()) => self.focus_message(message),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn prefix_notice(&mut self, prefix: crate::input::Prefix) -> Action {
        if prefix == crate::input::Prefix::Meta {
            return self.focus_message(prefix.label());
        }
        self.notice(prefix.label())
    }

    fn resize(&mut self, width: u16, height: u16) -> Action {
        self.size = Size { width, height };
        if self.tree.resize(self.size) {
            self.focus_message("File tree hidden at this width");
        }
        match self.stop_typing() {
            Ok(()) => Action::Redraw,
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn finish(&mut self, result: Result<bool, buffer::edit::Error>) -> Action {
        match result {
            Ok(true) => self.message(""),
            Ok(false) => self.message("Buffer boundary"),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn message(&mut self, message: &str) -> Action {
        self.echo = message.into();
        Action::Redraw
    }
}
