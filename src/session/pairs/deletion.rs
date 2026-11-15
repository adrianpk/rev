use std::ops::Range;
use std::time::Instant;

use crate::buffer::edit::{Error, Point};
use crate::config::Pairs;
use crate::coordinates::{Coordinates, Scalar};
use crate::input::EditingState;
use crate::pairs::{Outcome, Query, Status, Ticket};
use crate::session::{Action, Session};
use crate::window::ViewId;

/// One physical Backspace. Native regions and grapheme deletion keep precedence;
/// the qualifier may extend only the adjacent delimiter's ordinary range.
pub(in crate::session) struct Deletion {
    point: Point,
    view: ViewId,
    mark: (Option<Scalar>, bool),
    state: Option<EditingState>,
    policy: Pairs,
    ticket: Option<Ticket>,
    range: Option<Range<usize>>,
}

impl Session {
    pub(in crate::session) fn start_pair_deletion(&mut self) -> Option<Action> {
        if !self.pair_entry_focus() {
            return None;
        }
        let buffer = self.buffer().ok()?;
        let policy = buffer.pair_policy();
        if !policy.enabled || !policy.delete_empty || buffer.mark_active() {
            return None;
        }
        if !self.empty_pair_candidate().ok()? {
            return None;
        }
        if buffer.is_readonly() {
            return Some(self.error("Source text is read-only"));
        }
        self.layout.deletion = Some(Deletion {
            point: self.view().point,
            view: self.windows.selected_id(),
            mark: (buffer.mark(), buffer.mark_active()),
            state: self.editing_state(),
            policy,
            ticket: None,
            range: None,
        });
        Some(
            self.layout_pair_deletion()
                .unwrap_or_else(|| self.publish_pair_deletion()),
        )
    }

    fn empty_pair_candidate(&self) -> Result<bool, Error> {
        let buffer = self.buffer()?;
        let at = self.view().point.position.0;
        if at == 0 || at == buffer.len_chars() {
            return Ok(false);
        }
        // Constant-size admission only. All context and mate proof belongs to
        // the bounded request; no local characters substitute for that proof.
        let source = buffer.slice(&buffer.span(at - 1..at + 1)?)?;
        let mut chars = source.chars();
        let candidate = adjacent_pair(chars.next(), chars.next());
        Ok(candidate && self.pair_deletion_boundaries(at)?)
    }

    fn pair_deletion_boundaries(&self, at: usize) -> Result<bool, Error> {
        let coordinates = Coordinates::new(self.buffer()?)?;
        Ok(coordinates.is_boundary(Scalar(at - 1))? && coordinates.is_boundary(Scalar(at + 1))?)
    }

    fn pair_deletion_current(&self, work: &Deletion) -> bool {
        self.pair_entry_focus()
            && work.view == self.windows.selected_id()
            && work.point == self.view().point
            && work.state == self.editing_state()
            && self.buffer().is_ok_and(|b| {
                !b.is_readonly()
                    && work.policy == b.pair_policy()
                    && work.mark == (b.mark(), b.mark_active())
            })
    }

    pub(in crate::session) fn cancel_pair_deletion(&mut self) {
        if let Some(work) = self.layout.deletion.take()
            && let Some(ticket) = work.ticket
            && let Ok(buffer) = self.buffers.get_mut(ticket.stamp.buffer)
        {
            buffer.cancel_pair_request(ticket);
        }
    }

    pub(in crate::session) fn prepare_pair_deletion(&mut self) -> Result<(), crate::view::Error> {
        let mut work = self.layout.deletion.take().expect("owned pair deletion");
        let result = self.prepare_deletion_range(&mut work);
        self.layout.deletion = Some(work);
        result.map_err(Into::into)
    }

    fn prepare_deletion_range(&mut self, work: &mut Deletion) -> Result<(), Error> {
        if !self.pair_deletion_current(work) {
            return Err(Error::ChangedPositions);
        }
        let at = work.point.position.0;
        let buffer = self.buffers.get_mut(work.point.stamp.buffer)?;
        let ticket = match work.ticket {
            Some(ticket) => ticket,
            None => {
                let ticket = buffer.start_pair_request(Query::At(Scalar(at - 1)))?;
                work.ticket = Some(ticket);
                ticket
            }
        };
        match buffer.poll_pair_request(ticket, Instant::now() + crate::pairs::SLICE_TIME) {
            Status::Pending => return Err(crate::coordinates::Error::Pending.into()),
            Status::Retired => return Err(Error::ChangedPositions),
            Status::Ready => (),
        }
        let answer = buffer.pair_answer(ticket).expect("ready deletion proof");
        let paired = empty_admitted(&answer.outcome, at, work.policy.quotes);
        work.range = Some(at - 1..at + usize::from(paired));
        Ok(())
    }

    pub(in crate::session) fn publish_pair_deletion(&mut self) -> Action {
        let Some(work) = self.layout.deletion.take() else {
            return self.message("Pair deletion retired");
        };
        if !self.pair_deletion_current(&work) {
            return self.message("Pair deletion source changed");
        }
        // Re-enter the existing modal text capture after a cooperative wait;
        // deferred preparation must not lose the native repeat/history owner.
        let range = work.range.expect("complete deletion range");
        let cursor = Scalar(range.start);
        let result = self.entry_pair_text(range, "", cursor);
        self.sync_input_context();
        self.finish(result)
    }
}

fn empty_admitted(outcome: &Outcome, at: usize, quotes: bool) -> bool {
    let Outcome::Confirmed(pair) = outcome else {
        return false;
    };
    pair.opening.range == (at - 1..at)
        && pair.closing.range == (at..at + 1)
        && pair.depth.is_some()
        && (quotes || !pair.kind.quote())
}

fn adjacent_pair(opening: Option<char>, closing: Option<char>) -> bool {
    matches!(
        (opening, closing),
        (Some('('), Some(')'))
            | (Some('['), Some(']'))
            | (Some('{'), Some('}'))
            | (Some('"'), Some('"'))
            | (Some('\''), Some('\''))
            | (Some('`'), Some('`'))
    )
}
