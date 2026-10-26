mod buffers;
mod edit;
mod file;
mod history;
mod motion;
mod save;

use std::io::{self, Write};
use std::path::Path;

use crossterm::event::{self, Event, KeyCode};

use crate::buffer::{self, Buffer, BufferId, Buffers, Confirmation, Kill};
use crate::command::Command;
use crate::display::{Frame, Size};
use crate::input::{Input, Key, Outcome};
use crate::terminal::Terminal;
use crate::view::{self, View};

pub(crate) fn run(terminal: &mut Terminal<impl Write>, path: Option<&Path>) -> io::Result<()> {
    let mut session = initial(path)?;
    session.draw(terminal.writer())?;
    loop {
        match session.handle(event::read()?) {
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
    view: View,
    input: Input,
    size: Size,
    echo: String,
    pending: Option<Pending>,
}

enum Pending {
    Argument(Command),
    Save {
        confirmation: Box<Confirmation>,
        close: Option<Kill>,
    },
    Write {
        buffer: BufferId,
        close: Option<Kill>,
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
            view,
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
        self.buffers.get(self.view.point.stamp.buffer)
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn echo(&self) -> &str {
        &self.echo
    }

    pub fn frame(&mut self) -> Result<Frame, view::Error> {
        let buffer = self.buffers.get(self.view.point.stamp.buffer)?;
        self.view.reveal(buffer, self.size)?;
        let mut frame = Frame::project(buffer, &self.view, self.size, &self.echo)?;
        if let Some(prompt) = self.input.prompt() {
            frame.prompt(prompt, &self.echo)?;
        }
        match &self.pending {
            Some(Pending::Inventory(selected)) => frame.inventory(&self.buffers, *selected)?,
            Some(Pending::Save { .. } | Pending::Kill { .. }) => frame.decision(),
            _ => {}
        }
        Ok(frame)
    }

    fn draw(&mut self, writer: &mut impl Write) -> io::Result<()> {
        self.frame().map_err(io::Error::other)?.write(writer)
    }

    pub fn handle(&mut self, event: Event) -> Action {
        if let Event::Resize(width, height) = event {
            return self.resize(width, height);
        }
        let outcome = self.input.handle(event);
        self.outcome(outcome)
    }

    fn outcome(&mut self, outcome: Outcome) -> Action {
        match outcome {
            Outcome::Command(command) => self.execute(command),
            Outcome::Argument(path) => self.argument(path),
            Outcome::Decision(accepted) => self.decision(accepted),
            Outcome::Selection(selection) => self.select_buffer(selection),
            Outcome::Complete => self.complete_buffer(),
            Outcome::Changed => Action::Redraw,
            Outcome::Editing(key) => self.typing(key),
            Outcome::Prefix(prefix) => self.notice(prefix.label()),
            Outcome::Message(message) => self.notice(message),
            _ => Action::Wait,
        }
    }

    /// Executes the same implemented command enum produced by key bindings.
    pub fn execute(&mut self, command: Command) -> Action {
        if let Err(error) = self.stop_typing() {
            return self.message(&error.to_string());
        }
        self.dispatch(command)
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
            Command::KeyboardQuit => {
                self.input.cancel();
                self.message("Quit")
            }
            _ => self.edit_command(command),
        }
    }

    fn edit_command(&mut self, command: Command) -> Action {
        let point = self.view.point.position.0;
        let result = match command {
            Command::Newline => self.line_break(),
            Command::InsertTab => self.replace(point..point, "\t"),
            Command::BackwardDeleteChar => self.delete(true),
            Command::DeleteChar => self.delete(false),
            _ => return self.message("Command is not a text edit"),
        };
        self.finish(result)
    }

    fn navigation(&mut self, command: Command) {
        if matches!(
            command,
            Command::Scroll(_)
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
        self.view.reset_navigation();
    }

    /// Inserts one printable scalar, retaining a valid grapheme cursor after
    /// joins. Control bytes can enter only through explicit implemented commands.
    pub fn insert(&mut self, character: char) -> Action {
        self.view.reset_navigation();
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
            .get_mut(self.view.point.stamp.buffer)
            .map(Buffer::break_typing)
    }

    fn notice(&mut self, message: &str) -> Action {
        match self.stop_typing() {
            Ok(()) => self.focus_message(message),
            Err(error) => self.message(&error.to_string()),
        }
    }

    fn resize(&mut self, width: u16, height: u16) -> Action {
        self.size = Size { width, height };
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

    fn quit(&mut self) -> Action {
        if self.buffers.is_modified() {
            return self.message("Unsaved changes; cannot quit");
        }
        Action::Exit
    }

    fn message(&mut self, message: &str) -> Action {
        self.echo = message.into();
        Action::Redraw
    }
}
