mod edit;
mod motion;

use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode};

use crate::buffer::{self, Buffer, Buffers};
use crate::command::Command;
use crate::display::{Frame, Size};
use crate::input::{Input, Key, Outcome};
use crate::terminal::Terminal;
use crate::view::{self, View};

pub(crate) fn run(terminal: &mut Terminal<impl Write>) -> io::Result<()> {
    let (width, height) = crossterm::terminal::size()?;
    let mut session = Session::empty(Size { width, height }).map_err(io::Error::other)?;
    session.draw(terminal.writer())?;
    loop {
        match session.handle(event::read()?) {
            Action::Exit => return Ok(()),
            Action::Redraw => session.draw(terminal.writer())?,
            Action::Wait => {}
        }
    }
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
        Frame::project(buffer, &self.view, self.size, &self.echo)
    }

    fn draw(&mut self, writer: &mut impl Write) -> io::Result<()> {
        self.frame().map_err(io::Error::other)?.write(writer)
    }

    pub fn handle(&mut self, event: Event) -> Action {
        if let Event::Resize(width, height) = event {
            self.size = Size { width, height };
            return Action::Redraw;
        }
        match self.input.handle(event) {
            Outcome::Command(command) => self.execute(command),
            Outcome::Editing(key) => self.typing(key),
            Outcome::Prefix(prefix) => self.message(prefix.label()),
            Outcome::Message(message) => self.message(message),
            _ => Action::Wait,
        }
    }

    /// Executes the same implemented command enum produced by key bindings.
    pub fn execute(&mut self, command: Command) -> Action {
        if !matches!(command, Command::Scroll(_)) {
            self.view.reset_navigation();
        }
        let point = self.view.point.position.0;
        let result = match command {
            Command::Quit => return self.quit(),
            Command::Motion(motion) => return self.motion(motion),
            Command::Scroll(scroll) => return self.scroll(scroll),
            Command::KeyboardQuit => {
                self.input.cancel();
                return self.message("Quit");
            }
            Command::Newline => self.line_break(),
            Command::InsertTab => self.replace(point..point, "\t"),
            Command::BackwardDeleteChar => self.delete(true),
            Command::DeleteChar => self.delete(false),
        };
        self.finish(result)
    }

    /// Inserts one printable scalar, retaining a valid grapheme cursor after
    /// joins. Control bytes can enter only through explicit implemented commands.
    pub fn insert(&mut self, character: char) -> Action {
        self.view.reset_navigation();
        if character.is_control() {
            return self.message("Control character rejected");
        }
        let mut bytes = [0; 4];
        let text = character.encode_utf8(&mut bytes);
        let point = self.view.point.position.0;
        let result = self.replace(point..point, text);
        self.finish(result)
    }

    fn typing(&mut self, key: Key) -> Action {
        if let KeyCode::Char(character) = key.code
            && key.modifiers.is_empty()
        {
            return self.insert(character);
        }
        self.message("Unbound key")
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
            return self.message("Unsaved changes; cannot quit before saving is available");
        }
        Action::Exit
    }

    fn message(&mut self, message: &str) -> Action {
        self.echo = message.into();
        Action::Redraw
    }
}
