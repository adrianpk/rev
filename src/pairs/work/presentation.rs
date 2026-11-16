use super::{Fact, Opening, Work};
use crate::coordinates::Scalar;
use crate::pairs::presentation::{Delimiter, PendingDelimiter};
use crate::pairs::reader::Pause;
use crate::pairs::{Context, Kind, Outcome, Query, Reason};

impl Work {
    pub(in crate::pairs) fn poll_retention() -> usize {
        2 * crate::pairs::SLICE_TOKENS * std::mem::size_of::<Fact>()
    }
    pub(in crate::pairs) fn resume(
        &mut self,
        ticket: crate::pairs::Ticket,
        query: Query,
        stats: crate::pairs::Stats,
    ) {
        self.ticket = ticket;
        self.reader.bytes = stats.inspected_bytes;
        self.stats = stats;
        self.retarget(query);
    }

    pub(in crate::pairs) fn presentation_fact_count(&self) -> usize {
        self.facts.len()
    }

    pub(in crate::pairs) fn presentation_endpoint(&self, index: usize) -> Option<Delimiter> {
        match self.facts.get(index)? {
            Fact::Endpoint(endpoint) => endpoint.confirmed(),
            _ => None,
        }
    }

    pub(in crate::pairs) fn inspected_to(&self) -> usize {
        self.reader.at
    }

    pub(super) fn open_endpoint(
        &mut self,
        at: usize,
        kind: Kind,
        admitted: bool,
    ) -> Result<Option<usize>, Pause> {
        if !admitted {
            return Ok(None);
        }
        let index = self.facts.len();
        self.retain(Fact::Endpoint(PendingDelimiter {
            at: Scalar(at),
            mate: None,
            kind,
            depth: self.stack.len(),
            opening: true,
        }))?;
        Ok(Some(index))
    }

    pub(super) fn confirm_endpoints(&mut self, opening: &Opening, at: usize) -> Result<(), Pause> {
        let Some(index) = opening.endpoint else {
            return Ok(());
        };
        // Admit the closing record before publishing either endpoint's mate.
        self.retain(Fact::Endpoint(PendingDelimiter {
            at: Scalar(at),
            mate: Some(Scalar(opening.at)),
            kind: opening.kind,
            depth: opening.depth,
            opening: false,
        }))?;
        let Fact::Endpoint(endpoint) = &mut self.facts[index] else {
            unreachable!("opening endpoint fact has stable identity");
        };
        endpoint.mate = Some(Scalar(at));
        Ok(())
    }

    pub(super) fn restore_open_endpoints(&mut self) {
        for opening in &self.stack {
            if let Some(index) = opening.endpoint {
                let Fact::Endpoint(endpoint) = &mut self.facts[index] else {
                    unreachable!("checkpoint retains opening endpoint identity");
                };
                endpoint.mate = None;
            }
        }
    }

    pub(super) fn retarget_presentation(&mut self) {
        if let Some(outcome) = self.terminal.clone() {
            self.finish(outcome);
        } else {
            self.presentation_boundary();
        }
    }

    pub(super) fn presentation_boundary(&mut self) {
        if matches!(self.query, Query::Presentation(_))
            && self.reader.at >= self.query.point().0
            && self.stack.is_empty()
            && self.lexical.complete_at_eof()
            && self.answer.is_none()
        {
            self.finish(Outcome::Context(self.lexical.context()));
        }
    }

    pub(super) fn presentation_eof(&mut self) -> bool {
        if !matches!(self.query, Query::Presentation(_)) {
            return false;
        }
        let outcome = if !self.lexical.complete_at_eof() {
            Outcome::Uncertain(Reason::LiteralContext)
        } else if !self.stack.is_empty() {
            Outcome::Unmatched
        } else {
            Outcome::Context(Context::Code)
        };
        self.finish(outcome);
        true
    }
}
