use std::time::{Duration, Instant};

use crate::pairs::{SLICE_TIME, Status};
use crate::panel::Composition;
use crate::session::{Action, Session};
use crate::view::Error;

impl Session {
    pub(in crate::session) fn pair_presentation_remaining(&self, now: Instant) -> Option<Duration> {
        if self.startup_screen.is_some() || self.git_visible() {
            return None;
        }
        [
            self.pair_work_pending().then_some(Duration::ZERO),
            self.pair_pulse_remaining(now),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub(in crate::session) fn expire_pair_presentation(&mut self, now: Instant) -> Action {
        let expired = self.reconcile_pair_pulses(now);
        if !expired && !self.pair_work_pending() {
            return Action::Wait;
        }
        let deadline = Instant::now() + SLICE_TIME;
        let pending: Vec<_> = self
            .windows
            .iter()
            .filter_map(|(id, view)| {
                view.pairs
                    .ticket
                    .filter(|_| view.pairs.pending)
                    .map(|ticket| (id, ticket))
            })
            .collect();
        for (id, ticket) in pending {
            let status = self
                .buffers
                .get_mut(ticket.stamp.buffer)
                .map_or(Status::Retired, |buffer| {
                    buffer.poll_pair_presentation(ticket, deadline)
                });
            if let Ok(view) = self.windows.get_mut(id) {
                view.pairs.pending = status == Status::Pending;
            }
            self.qualify_pair_pulse(id, status);
        }
        Action::Redraw
    }

    pub(in crate::session) fn refresh_pair_presentation(
        &mut self,
        composition: Option<&Composition>,
    ) -> Result<(), Error> {
        self.retire_pair_views()?;
        let Some(composition) = composition else {
            return Ok(());
        };
        let deadline = Instant::now() + SLICE_TIME;
        for (id, panel) in composition.panels() {
            self.refresh_pair_view(*id, *panel, deadline)?;
        }
        self.reconcile_pair_pulses(self.pair_now());
        Ok(())
    }

    fn refresh_pair_view(
        &mut self,
        id: crate::window::ViewId,
        panel: crate::panel::Panel,
        deadline: Instant,
    ) -> Result<(), Error> {
        let Some(range) = self.pair_view_range(id, panel)? else {
            return Ok(());
        };
        let buffer = self.buffers.get_mut(range.stamp.buffer)?;
        let Ok(ticket) = buffer.refresh_pair_presentation(id, range) else {
            self.windows.get_mut(id)?.pairs.retire();
            self.qualify_pair_pulse(id, Status::Retired);
            return Ok(());
        };
        let status = buffer.poll_pair_presentation(ticket, deadline);
        let state = &mut self.windows.get_mut(id)?.pairs;
        if let Some(pulse) = &mut state.pulse {
            pulse.ticket = ticket;
        }
        state.ticket = Some(ticket);
        state.pending = status == Status::Pending;
        self.qualify_pair_pulse(id, status);
        Ok(())
    }

    fn pair_view_range(
        &self,
        id: crate::window::ViewId,
        panel: crate::panel::Panel,
    ) -> Result<Option<crate::buffer::Span>, Error> {
        let view = self.windows.get(id)?;
        let buffer = self.buffers.get(view.point.stamp.buffer)?;
        if buffer.terminal().is_some() || !view.pairs.visible(buffer.pair_policy(), view.point) {
            return Ok(None);
        }
        let range = view.pair_visible_span(buffer, panel.content())?;
        Ok((!range.range.is_empty()).then_some(range))
    }

    fn retire_pair_views(&mut self) -> Result<(), Error> {
        let ids: Vec<_> = self.buffers.iter().map(crate::buffer::Buffer::id).collect();
        for id in ids {
            self.buffers
                .get_mut(id)?
                .retain_pair_presentations(&self.windows);
        }
        // Hidden/suspended source panes own no wakeup. Their next visible frame
        // re-admits current source interests without reviving an old ticket.
        self.windows.retire_pair_tickets();
        Ok(())
    }
}
