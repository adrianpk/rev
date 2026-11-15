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
        let at = if self.text.get_char(point.0).is_some_and(delimiter) {
            point
        } else {
            Scalar(point.0.saturating_sub(1))
        };
        self.start_pair_request(Query::At(at))
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

    /// Start one counted group request, retiring any typed-pair work. Group
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
