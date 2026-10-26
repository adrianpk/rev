use std::io::{self, Write};

use crossterm::event::{self, Event};

use crate::buffer::{self, Buffers};
use crate::command::Command;
use crate::display::{Frame, Size};
use crate::input::{Input, Outcome};
use crate::terminal::Terminal;
use crate::view::View;

pub(super) fn run(terminal: &mut Terminal<impl Write>) -> io::Result<()> {
    let (width, height) = crossterm::terminal::size()?;
    let mut session = Session::new(Size { width, height }).map_err(io::Error::other)?;
    session.draw(terminal.writer())?;
    loop {
        match session.handle(event::read()?) {
            Action::Exit => return Ok(()),
            Action::Redraw => session.draw(terminal.writer())?,
            Action::Wait => {}
        }
    }
}

enum Action {
    Exit,
    Redraw,
    Wait,
}

struct Session {
    buffers: Buffers,
    view: View,
    input: Input,
    size: Size,
    echo: String,
}

impl Session {
    fn new(size: Size) -> Result<Self, buffer::Error> {
        let buffers = Buffers::new()?;
        let view = View::new(buffers.get(buffers.initial())?);
        Ok(Self {
            buffers,
            view,
            input: Input::default(),
            size,
            echo: "C-x C-c: exit".into(),
        })
    }

    fn draw(&mut self, writer: &mut impl Write) -> io::Result<()> {
        let buffer = self
            .buffers
            .get(self.view.point.stamp.buffer)
            .map_err(io::Error::other)?;
        self.view
            .reveal(buffer, self.size)
            .map_err(io::Error::other)?;
        let frame =
            Frame::project(buffer, &self.view, self.size, &self.echo).map_err(io::Error::other)?;
        frame.write(writer)
    }

    fn handle(&mut self, event: Event) -> Action {
        if let Event::Resize(width, height) = event {
            self.size = Size { width, height };
            return Action::Redraw;
        }
        let outcome = self.input.handle(event);
        let message = match outcome {
            Outcome::Command(Command::Quit) => return Action::Exit,
            Outcome::Command(Command::KeyboardQuit) => "Quit",
            Outcome::Prefix(prefix) => prefix.label(),
            Outcome::Message(message) => message,
            _ => return Action::Wait,
        };
        self.echo = message.into();
        Action::Redraw
    }
}
