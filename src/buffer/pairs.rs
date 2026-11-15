use std::time::Instant;

use crate::pairs::{Answer, Profile, Query, Stats, Status, Ticket};

use super::{Buffer, Error};

impl Buffer {
    /// Begin one buffer-owned request and retire its previous ticket. Verified
    /// prefix facts can be reused; source, point, history and settings are unchanged.
    pub fn start_pair_request(&mut self, query: Query) -> Result<Ticket, Error> {
        self.span(query.point().0..query.point().0)?;
        let byte = self.text.char_to_byte(query.point().0);
        let profile = if self.is_org() {
            Profile::Org
        } else if self.is_markdown() {
            Profile::Markdown
        } else {
            Profile::language(&self.language_identity)
        };
        let hint = self.syntax.pair_hint(self.stamp(), byte, profile);
        Ok(self.pairing.start(query, (self.stamp(), profile), hint))
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
