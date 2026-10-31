use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossterm::event::{Event, MouseButton, MouseEventKind};

use crate::command::{Command, KillCommand, Motion};
use crate::input::{Focus, Input};
use crate::view::{Error, LayoutBudget, Size, View};
use crate::window::ViewId;

use super::{Action, Session};

const INPUT_LIMIT: usize = 256;

#[derive(Clone, Copy)]
enum Request {
    Command(Command),
    Pointer(crate::pointer::Event, Instant),
}

enum Deferred {
    Event(Event, Instant),
    Command(Command),
}

struct Held {
    request: Request,
    selected: ViewId,
    point: crate::buffer::edit::Point,
    views: Vec<(ViewId, View)>,
    size: Size,
}

/// One pure layout-dependent operation and a finite FIFO of later native input.
/// The host stops reading before this FIFO fills; it never discards typed input.
#[derive(Default)]
pub(super) struct Work {
    held: Option<Held>,
    input: VecDeque<Deferred>,
    interrupt: bool,
    replaying: bool,
}

impl Session {
    pub(super) fn layout_active(&self) -> bool {
        self.layout.held.is_some() || !self.layout.input.is_empty()
    }

    pub(super) fn layout_full(&self) -> bool {
        self.layout.input.len() >= INPUT_LIMIT
    }

    pub(super) fn defer_layout_event(
        &mut self,
        event: Event,
        now: Instant,
    ) -> Result<Event, Action> {
        if !self.layout_active() || matches!(event, Event::Resize(..)) {
            return Ok(event);
        }
        if Input::interrupt(&event) {
            self.layout.held = None;
            self.layout.interrupt = true;
            // One interrupt may follow the full ordinary-input allowance.
            if self.layout.input.len() <= INPUT_LIMIT {
                self.layout.input.push_back(Deferred::Event(event, now));
                return Err(self.message("Quit"));
            }
            return Err(self.error("Native input queue full; service its deadline"));
        }
        if let Event::Mouse(mouse) = &event {
            if mouse.kind != MouseEventKind::Up(MouseButton::Left) {
                self.cancel_pointer_capture();
                return Err(Action::Wait);
            }
            self.stop_pointer_scroll();
        }
        if self.layout_full() {
            return Err(self.error("Native input queue full; service its deadline"));
        }
        self.layout.input.push_back(Deferred::Event(event, now));
        Err(Action::Wait)
    }

    pub(super) fn defer_layout_command(&mut self, command: Command) -> Option<Action> {
        if self.layout.replaying || !self.layout_active() {
            return None;
        }
        if command == Command::KeyboardQuit {
            self.layout.held = None;
            self.layout.interrupt = true;
            if self.layout.input.len() <= INPUT_LIMIT {
                self.layout.input.push_back(Deferred::Command(command));
                return Some(self.message("Quit"));
            }
        } else if !self.layout_full() {
            self.layout.input.push_back(Deferred::Command(command));
            return Some(Action::Wait);
        }
        Some(self.error("Native input queue full; service its deadline"))
    }

    pub(super) fn layout_command(&mut self, command: Command) -> Option<Action> {
        let applicable = matches!(
            command,
            Command::Scroll(_)
                | Command::Kill(KillCommand::Line)
                | Command::Motion(
                    Motion::LineStart
                        | Motion::LineEnd
                        | Motion::ForwardChar
                        | Motion::BackwardChar
                )
        );
        if !applicable
            || self.input.focus() != Focus::Editing
            || self.tree.focused()
            || self.habit_agenda.is_some()
            || self.agenda_active()
            || self.query_agenda.is_some()
        {
            return None;
        }
        self.prepare_request(Request::Command(command))
    }

    pub(super) fn layout_pointer(
        &mut self,
        event: crate::pointer::Event,
        now: Instant,
    ) -> Option<Action> {
        if !self.mouse_enabled || self.input.focus() != Focus::Editing {
            return None;
        }
        self.prepare_request(Request::Pointer(event, now))
    }

    fn prepare_request(&mut self, request: Request) -> Option<Action> {
        match self.prepare_layout_request(request) {
            Ok(()) => None,
            Err(error) if error.pending() && self.layout.interrupt => Some(Action::Wait),
            Err(error) if error.pending() => {
                self.layout.held = Some(Held {
                    request,
                    selected: self.windows.selected_id(),
                    point: self.view().point,
                    views: if matches!(request, Request::Pointer(..)) {
                        self.windows
                            .iter()
                            .map(|(id, view)| (id, view.clone()))
                            .collect()
                    } else {
                        Vec::new()
                    },
                    size: self.size,
                });
                Some(self.message("Reflowing text"))
            }
            Err(error) => Some(self.error(&error.to_string())),
        }
    }

    fn prepare_layout_request(&self, request: Request) -> Result<(), Error> {
        let work = Arc::new(Mutex::new(LayoutBudget::new()));
        let scopes = self.layout_scopes(work.clone())?;
        let work = scopes.0.first().expect("live windows own a source").work();
        let view = self.view();
        let buffer = self.buffer()?;
        let character = matches!(
            request,
            Request::Command(Command::Motion(Motion::ForwardChar | Motion::BackwardChar))
        );
        if !character
            && matches!(request, Request::Command(_))
            && !view.prepare_point(
                buffer,
                self.pane_size(),
                &mut work.lock().expect("layout budget poisoned"),
            )?
        {
            return Err(crate::coordinates::Error::Pending.into());
        }
        let result = match request {
            Request::Command(Command::Scroll(scroll)) => self.scrolling_view(scroll).map(|_| ()),
            Request::Command(Command::Motion(motion)) => self.motion_target(motion).map(|_| ()),
            Request::Command(Command::Kill(command)) => self.kill_span(command).map(|_| ()),
            Request::Command(_) => unreachable!("only layout-dependent commands are prepared"),
            Request::Pointer(event, _) => self.prepare_pointer(event),
        };
        drop(scopes);
        result
    }

    pub(super) fn expire_layout_input(&mut self) -> Option<Action> {
        if !self.layout_active() {
            return None;
        }
        let started = Instant::now();
        self.layout.replaying = true;
        let mut action = Action::Wait;
        if let Some(held) = self.layout.held.take() {
            let valid = held.selected == self.windows.selected_id()
                && match held.request {
                    Request::Command(_) => held.point == self.view().point,
                    Request::Pointer(..) => {
                        held.size == self.size
                            && held.views.len() == self.windows.iter().len()
                            && held.views.iter().all(|(id, view)| {
                                self.windows.get(*id).is_ok_and(|current| current == view)
                            })
                    }
                };
            if !valid {
                action = self.message("Layout source or geometry changed");
            } else if let Some(outcome) = self.prepare_request(held.request) {
                action = outcome;
            } else {
                action = match held.request {
                    Request::Command(command) => self.execute_now(command),
                    Request::Pointer(event, now) => self.pointer_now(event, now),
                };
            }
        }
        for _ in 0..super::events::INPUT_BATCH_EVENTS {
            if self.layout.held.is_some()
                || action == Action::Exit
                || started.elapsed() >= super::events::INPUT_BATCH_TIME
            {
                break;
            }
            let Some(input) = self.layout.input.pop_front() else {
                break;
            };
            let interrupt = match &input {
                Deferred::Command(command) => *command == Command::KeyboardQuit,
                Deferred::Event(event, _) => Input::interrupt(event),
            };
            if interrupt {
                self.layout.interrupt = false;
            }
            let outcome = match input {
                Deferred::Command(command) => self.execute_now(command),
                Deferred::Event(event, now) => self.handle_ready_at(event, now),
            };
            if outcome != Action::Wait {
                action = outcome;
            }
        }
        self.layout.replaying = false;
        Some(action)
    }
}

#[cfg(test)]
mod tests;
