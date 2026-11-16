use std::time::{Duration, Instant};

use crate::buffer::edit::Point;
use crate::config::PairHighlight;
use crate::pairs::{PresentationTicket, Status};
use crate::session::{Action, Message, Session};
use crate::window::ViewId;

#[derive(Clone, Copy)]
pub(in crate::session) struct Origin {
    view: ViewId,
    point: Point,
    generation: u64,
}

impl Session {
    /// Supply monotonic time for an owned host or deterministic native fixture.
    pub fn with_pair_clock(mut self, clock: fn() -> Instant) -> Self {
        self.pair_clock = Some(clock);
        self
    }

    pub(in crate::session) fn pair_now(&self) -> Instant {
        self.pair_clock.unwrap_or(Instant::now)()
    }

    pub(in crate::session) fn pair_input_origin(&self) -> Option<Origin> {
        if !self.pair_presentation_focus() || !self.pair_style() {
            return None;
        }
        Some(Origin {
            view: self.windows.selected_id(),
            point: self.view().point,
            generation: self.view().pairs.generation,
        })
    }

    pub(in crate::session) fn pair_command_origin(
        &self,
        command: crate::command::Command,
    ) -> Option<Origin> {
        if matches!(
            command,
            crate::command::Command::KeyboardQuit | crate::command::Command::RevertBuffer
        ) {
            None
        } else {
            self.pair_input_origin()
        }
    }

    pub(in crate::session) fn pair_outcome_origin(
        &self,
        outcome: &crate::input::Outcome,
    ) -> Option<Origin> {
        use crate::input::Outcome;
        match outcome {
            Outcome::Editing(_)
            | Outcome::Paste(_)
            | Outcome::Pointer(_)
            | Outcome::TextInvocation(_) => self.pair_input_origin(),
            Outcome::Invocation(invocation) => self.pair_command_origin(invocation.command),
            _ => None,
        }
    }

    fn pair_style(&self) -> bool {
        self.buffer()
            .is_ok_and(|b| self.view().pairs.style(b.pair_policy()) == PairHighlight::Pulse)
    }

    pub(in crate::session) fn after_pair_input(
        &mut self,
        origin: Option<Origin>,
        now: Instant,
        action: Action,
    ) -> Action {
        let changed = self.reconcile_pair_pulses(now);
        if let Some(origin) = origin
            && self.changed_pair_input(origin)
        {
            self.arm_pair_pulse(now);
        }
        Self::pair_redraw(action, changed)
    }

    pub(in crate::session) fn pair_redraw(action: Action, changed: bool) -> Action {
        if changed && action == Action::Wait {
            Action::Redraw
        } else {
            action
        }
    }

    fn changed_pair_input(&self, origin: Origin) -> bool {
        self.pair_presentation_focus()
            && self.pair_style()
            && self.echo_kind != Message::Error
            && origin.view == self.windows.selected_id()
            && origin.point != self.view().point
            && origin.generation == self.view().pairs.generation
    }

    fn arm_pair_pulse(&mut self, now: Instant) {
        let point = self.view().point;
        let id = self.windows.selected_id();
        let Ok(buffer) = self.buffers.get_mut(point.stamp.buffer) else {
            return;
        };
        let Some(entry) = buffer.pair_candidate(point.position) else {
            return;
        };
        let duration = buffer.pair_policy().pulse_ms;
        let range = buffer
            .span(entry.0..entry.0 + 1)
            .expect("current source endpoint");
        let Ok(ticket) = buffer.refresh_pair_presentation(id, range) else {
            return;
        };
        let state = &mut self.windows.selected_mut().pairs;
        state.arm_pulse(point, now, duration, ticket);
        state.ticket = Some(ticket);
        state.pending = true;
    }

    pub(in crate::session) fn pair_pulse_remaining(&self, now: Instant) -> Option<Duration> {
        self.windows
            .iter()
            .filter_map(|(id, view)| {
                self.current_pair_pulse(id)
                    .then_some(view.pairs.pulse?)
                    .map(|pulse| pulse.expires.saturating_duration_since(now))
            })
            .min()
    }

    pub(in crate::session) fn pair_work_pending(&self) -> bool {
        self.windows.iter().any(|(id, view)| {
            view.pairs.pending && (view.pairs.pulse.is_none() || self.current_pair_pulse(id))
        })
    }

    fn current_pair_pulse(&self, id: ViewId) -> bool {
        let Ok(view) = self.windows.get(id) else {
            return false;
        };
        let Some(pulse) = view.pairs.pulse else {
            return false;
        };
        id == self.windows.selected_id()
            && pulse.point == view.point
            && self.pair_presentation_focus()
            && self.pair_style()
            && self
                .buffers
                .get(pulse.point.stamp.buffer)
                .is_ok_and(|b| b.pair_presentation_current(pulse.ticket))
    }

    pub(in crate::session) fn reconcile_pair_pulses(&mut self, now: Instant) -> bool {
        let expired: Vec<_> = self
            .windows
            .iter()
            .filter_map(|(id, view)| {
                let pulse = view.pairs.pulse?;
                (!self.current_pair_pulse(id) || now >= pulse.expires).then_some(id)
            })
            .collect();
        let changed = !expired.is_empty();
        for id in expired {
            self.cancel_pair_pulse(id);
        }
        changed
    }

    pub(in crate::session) fn cancel_pair_pulses(&mut self) -> bool {
        let ids: Vec<_> = self
            .windows
            .iter()
            .filter_map(|(id, v)| v.pairs.pulse.map(|_| id))
            .collect();
        let changed = !ids.is_empty();
        for id in ids {
            self.cancel_pair_pulse(id);
        }
        changed
    }

    fn cancel_pair_pulse(&mut self, id: ViewId) {
        let Ok(view) = self.windows.get_mut(id) else {
            return;
        };
        let ticket = view.pairs.pulse.map(|pulse| pulse.ticket);
        view.pairs.cancel_pulse();
        if let Some(ticket) = ticket
            && let Ok(buffer) = self.buffers.get_mut(ticket.stamp.buffer)
        {
            buffer.cancel_pair_presentation(ticket);
        }
    }

    pub(in crate::session) fn qualify_pair_pulse(&mut self, id: ViewId, status: Status) {
        let Ok(view) = self.windows.get(id) else {
            return;
        };
        if view.pairs.pulse.is_none() || status == Status::Pending {
            return;
        }
        let confirmed = view
            .pairs
            .ticket
            .and_then(|ticket| self.pulse_endpoint(ticket, view.point));
        if confirmed.is_none() {
            self.cancel_pair_pulse(id);
        }
    }

    fn pulse_endpoint(
        &self,
        ticket: PresentationTicket,
        point: Point,
    ) -> Option<crate::pairs::Delimiter> {
        self.buffers
            .get(point.stamp.buffer)
            .ok()?
            .active_pair_presentation(ticket, point.position)
    }

    pub(in crate::session) fn begin_pair_event(&mut self, event: &crossterm::event::Event) -> bool {
        if matches!(event, crossterm::event::Event::FocusLost)
            || crate::input::Input::interrupt(event)
        {
            self.cancel_pair_pulses()
        } else {
            false
        }
    }

    pub(in crate::session) fn begin_pair_command(&mut self, command: crate::command::Command) {
        if matches!(
            command,
            crate::command::Command::KeyboardQuit | crate::command::Command::RevertBuffer
        ) {
            self.cancel_pair_pulses();
        }
    }
}

impl Session {
    pub(in crate::session) fn pair_view_changed(&mut self, previous: ViewId) {
        if previous != self.windows.selected_id() {
            self.cancel_pair_pulses();
        }
    }
}
