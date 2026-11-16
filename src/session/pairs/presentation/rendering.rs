use std::time::{Duration, Instant};

use crate::config::PairHighlight;
use crate::pairs::{SLICE_TIME, Status};
use crate::panel::Composition;
use crate::session::{Action, Session};
use crate::view::Error;

impl Session {
    pub(in crate::session) fn pair_presentation_remaining(&self) -> Option<Duration> {
        if self.startup_screen.is_some() || self.git_visible() {
            return None;
        }
        self.windows
            .iter()
            .any(|(_, v)| v.pairs.pending)
            .then_some(Duration::ZERO)
    }

    pub(in crate::session) fn expire_pair_presentation(&mut self) -> Action {
        if self.pair_presentation_remaining().is_none() {
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
            return Ok(());
        };
        let status = buffer.poll_pair_presentation(ticket, deadline);
        let state = &mut self.windows.get_mut(id)?.pairs;
        state.ticket = Some(ticket);
        state.pending = status == Status::Pending;
        Ok(())
    }

    fn pair_view_range(
        &self,
        id: crate::window::ViewId,
        panel: crate::panel::Panel,
    ) -> Result<Option<crate::buffer::Span>, Error> {
        let view = self.windows.get(id)?;
        let buffer = self.buffers.get(view.point.stamp.buffer)?;
        if buffer.terminal().is_some()
            || matches!(
                view.pairs.style(buffer.pair_policy()),
                PairHighlight::Off | PairHighlight::Pulse
            )
        {
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
