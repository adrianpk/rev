use std::time::Instant;

use crate::coordinates::Scalar;
use crate::pairs::{
    Answer, GroupAnswer, GroupRequest, Kind, Profile, Query, Stats, Status, Ticket,
};

use super::{Buffer, Error};

impl Buffer {
    /// Prefer a delimiter at point, then immediately before it. Qualification
    /// uses one existing bounded request; there is no later-line search.
    pub fn start_mate_request(&mut self, point: Scalar) -> Result<Ticket, Error> {
        self.span(point.0..point.0)?;
        let at = self.pair_entry(point);
        self.start_pair_request(Query::At(at))
    }

    pub(crate) fn pair_entry(&self, point: Scalar) -> Scalar {
        if self.text.get_char(point.0).is_some_and(delimiter) {
            point
        } else {
            Scalar(point.0.saturating_sub(1))
        }
    }

    pub(crate) fn pair_candidate(&self, point: Scalar) -> Option<Scalar> {
        let entry = self.pair_entry(point);
        self.text
            .get_char(entry.0)
            .filter(|c| delimiter(*c))
            .map(|_| entry)
    }

    pub(crate) fn active_pair_presentation(
        &self,
        ticket: crate::pairs::PresentationTicket,
        point: Scalar,
    ) -> Option<crate::pairs::Delimiter> {
        let facts = self.pair_presentation(ticket)?;
        let entry = self.pair_entry(point);
        facts
            .spans
            .binary_search_by_key(&entry, |s| s.at)
            .ok()
            .map(|i| facts.spans[i])
    }

    /// Begin one buffer-owned request and retire its previous ticket. Verified
    /// prefix facts can be reused; source, point, history and settings are unchanged.
    pub fn start_pair_request(&mut self, query: Query) -> Result<Ticket, Error> {
        self.span(query.point().0..query.point().0)?;
        match query {
            Query::Typing(input) => {
                self.span(input.point.0..input.end.0)?;
            }
            Query::Wrap(input) => {
                self.span(input.start.0..input.end.0)?;
            }
            _ => (),
        }
        let byte = self.text.char_to_byte(query.point().0);
        let profile = self.pair_profile();
        let hint = self.syntax.pair_hint(self.stamp(), byte, profile);
        Ok(self.pairing.start(query, (self.stamp(), profile), hint))
    }

    /// Start one counted group/expression request, retiring typed-pair work. Its
    /// outcomes never supply typed-pair facts to editing or presentation.
    pub fn start_group_request(&mut self, request: GroupRequest) -> Result<Ticket, Error> {
        self.span(request.point.0..request.point.0)?;
        let profile = self.pair_profile();
        let hint = self.syntax.pair_hint(
            self.stamp(),
            self.text.char_to_byte(request.point.0),
            profile,
        );
        Ok(self
            .pairing
            .start_groups(request, (self.stamp(), profile), hint))
    }

    pub fn group_answer(&self, ticket: Ticket) -> Option<&GroupAnswer> {
        self.pairing.group_answer(ticket)
    }

    fn pair_profile(&self) -> Profile {
        if self.is_org() {
            Profile::Org
        } else if self.is_markdown() {
            Profile::Markdown
        } else {
            Profile::language(&self.language_identity)
        }
    }

    /// At most 256 lexical steps and the native four-millisecond target per poll.
    /// An earlier caller deadline wins. No worker or timer is started.
    pub fn poll_pair_request(&mut self, ticket: Ticket, deadline: Instant) -> Status {
        if ticket.stamp != self.stamp() {
            return Status::Retired;
        }
        self.pairing.poll(ticket, self.text.slice(..), deadline)
    }

    /// Borrow a completed result only while the captured request is still owned.
    pub fn pair_answer(&self, ticket: Ticket) -> Option<&Answer> {
        self.pairing.answer(ticket)
    }

    pub fn pair_stats(&self, ticket: Ticket) -> Option<Stats> {
        self.pairing.stats(ticket)
    }

    /// Reuse one admitted request for another counted lookup without resetting
    /// inspection or retained-memory budgets. Cached facts retain their stamp.
    pub fn continue_pair_request(&mut self, ticket: Ticket, query: Query) -> Result<bool, Error> {
        self.span(query.point().0..query.point().0)?;
        Ok(self.pairing.continue_query(ticket, query))
    }

    pub fn cancel_pair_request(&mut self, ticket: Ticket) -> bool {
        self.pairing.cancel(ticket)
    }
}

fn delimiter(c: char) -> bool {
    Kind::structural(c).is_some() || matches!(c, '"' | '\'' | '`')
}

impl Buffer {
    /// Capture a live view's visible source range without retiring foreground
    /// editing/navigation. The caller cancels its ticket when that view closes.
    pub fn start_pair_presentation(
        &mut self,
        view: crate::window::ViewId,
        visible: super::Span,
    ) -> Result<crate::pairs::PresentationTicket, Error> {
        self.slice(&visible)?;
        self.pairing
            .start_presentation(
                view,
                visible,
                std::mem::size_of::<crate::view::PairPresentation>(),
            )
            .ok_or(Error::Exhausted)
    }

    /// Repaint retains an identical live interest. Explicit start requests still
    /// retire previous generations; source publication invalidates their entries.
    pub(crate) fn refresh_pair_presentation(
        &mut self,
        view: crate::window::ViewId,
        visible: super::Span,
    ) -> Result<crate::pairs::PresentationTicket, Error> {
        self.slice(&visible)?;
        match self.pairing.presentation_interest(view, &visible) {
            Some(ticket) => Ok(ticket),
            None => self.start_pair_presentation(view, visible),
        }
    }

    /// One cooperative lexical or export slice. Pending foreground work wins;
    /// source/view generations are checked before accessing retained facts.
    pub fn poll_pair_presentation(
        &mut self,
        ticket: crate::pairs::PresentationTicket,
        deadline: Instant,
    ) -> Status {
        if ticket.stamp != self.stamp() {
            return Status::Retired;
        }
        let profile = self.pair_profile();
        self.pairing
            .poll_presentation(ticket, (self.text.slice(..), profile), deadline)
    }

    pub fn pair_presentation(
        &self,
        ticket: crate::pairs::PresentationTicket,
    ) -> Option<crate::pairs::Presentation<'_>> {
        if ticket.stamp != self.stamp() {
            return None;
        }
        self.pairing.presentation(ticket)
    }

    pub(crate) fn pair_presentation_current(
        &self,
        ticket: crate::pairs::PresentationTicket,
    ) -> bool {
        ticket.stamp == self.stamp() && self.pairing.presentation_current(ticket)
    }

    pub fn cancel_pair_presentation(&mut self, ticket: crate::pairs::PresentationTicket) -> bool {
        self.pairing.cancel_presentation(ticket)
    }

    /// Actual capacities of the scanner, publication, export and view interests,
    /// plus their inline owners, counted once per canonical buffer.
    pub fn retained_pair_bytes(&self) -> usize {
        self.pairing.retained()
    }
}

impl Buffer {
    pub(crate) fn retain_pair_presentations(&mut self, windows: &crate::window::Windows) {
        let id = self.id();
        let policy = self.pair_policy();
        let source = self.terminal().is_none();
        let visible = source
            && windows
                .iter()
                .any(|(_, v)| v.point.stamp.buffer == id && v.pairs.visible(policy, v.point));
        self.pairing.retain_presentations(visible, |view| {
            windows.get(view).is_ok_and(|v| {
                source && v.point.stamp.buffer == id && v.pairs.visible(policy, v.point)
            })
        });
    }
}
