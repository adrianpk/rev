use std::ops::Range;
use std::time::Instant;

use super::{Delimiter, MAX_VISIBLE_SPANS, Presentation, PresentationOutcome, PresentationTicket};
use crate::buffer::Stamp;
use crate::pairs::{
    Answer, Limit, MAX_METADATA, POLICY_VERSION, Profile, Provenance, SLICE_TIME, SLICE_TOKENS,
    Stats, work::Work,
};
use crate::window::ViewId;

pub(in crate::pairs) struct Entry {
    ticket: PresentationTicket,
    range: Range<usize>,
}

pub(in crate::pairs) struct Snapshot {
    pub stamp: Stamp,
    pub profile: Profile,
    pub provenance: Provenance,
    pub spans: Vec<Delimiter>,
    pub inspected: usize,
    target: usize,
    outcome: PresentationOutcome,
    stats: Stats,
}

#[derive(Default)]
pub(in crate::pairs) struct Store {
    publication: u64,
    generation: u64,
    entries: Vec<Entry>,
    pub snapshot: Option<Snapshot>,
    build: Vec<Delimiter>,
    cursor: usize,
    target: usize,
    export_slices: usize,
    maximum_export_tokens: usize,
}

impl Store {
    pub fn proof(
        &self,
        record: &crate::buffer::edit::Transaction,
        texts: (&ropey::Rope, &ropey::Rope),
    ) -> Option<super::Advance> {
        super::Advance::prepare((self.snapshot.as_ref()?, self.publication), record, texts)
    }

    pub fn heap(&self) -> usize {
        self.entries.capacity() * std::mem::size_of::<Entry>()
            + self.build.capacity() * std::mem::size_of::<Delimiter>()
            + self
                .snapshot
                .as_ref()
                .map_or(0, |s| s.spans.capacity() * std::mem::size_of::<Delimiter>())
    }

    pub fn request(
        &mut self,
        view: ViewId,
        span: crate::buffer::Span,
        retained: usize,
    ) -> Option<PresentationTicket> {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("presentation generation exhausted");
        let ticket = PresentationTicket {
            stamp: span.stamp,
            view,
            generation: self.generation,
        };
        let entry = Entry {
            ticket,
            range: span.range,
        };
        if let Some(old) = self.entries.iter_mut().find(|e| e.ticket.view == view) {
            *old = entry;
        } else {
            self.reserve_entry(retained)?;
            self.entries.push(entry);
        }
        Some(ticket)
    }

    fn reserve_entry(&mut self, retained: usize) -> Option<()> {
        if self.entries.len() < self.entries.capacity() {
            return Some(());
        }
        if retained.saturating_add(std::mem::size_of::<Entry>()) > MAX_METADATA {
            return None;
        }
        self.entries.try_reserve_exact(1).ok()?;
        Some(())
    }

    pub fn needs_target(&self, end: usize) -> bool {
        end > self.target || self.snapshot.is_some()
    }

    pub fn empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn range(&self, ticket: PresentationTicket) -> Option<Range<usize>> {
        self.entries
            .iter()
            .find(|e| e.ticket == ticket)
            .map(|e| e.range.clone())
    }

    pub fn ready(&self, ticket: PresentationTicket) -> bool {
        let Some(range) = self.range(ticket) else {
            return false;
        };
        self.snapshot
            .as_ref()
            .is_some_and(|s| s.stamp == ticket.stamp && range.end <= s.target)
    }

    pub fn cancel(&mut self, ticket: PresentationTicket) -> bool {
        let Some(index) = self.entries.iter().position(|e| e.ticket == ticket) else {
            return false;
        };
        self.entries.swap_remove(index);
        if self.entries.is_empty() {
            self.abandon();
        }
        true
    }

    pub fn retire(&mut self) {
        self.abandon();
        self.entries = Vec::new();
    }

    pub fn abandon(&mut self) {
        self.snapshot = None;
        self.build = Vec::new();
        self.cursor = 0;
        self.export_slices = 0;
        self.maximum_export_tokens = 0;
    }

    pub fn begin(&mut self, target: usize) {
        self.abandon();
        self.target = target;
    }

    pub fn stop_export(&mut self) {
        self.build = Vec::new();
        self.cursor = 0;
        self.export_slices = 0;
        self.maximum_export_tokens = 0;
    }

    pub fn export(&mut self, work: &Work, retained: usize, deadline: Instant) -> bool {
        self.export_slices += 1;
        let deadline = deadline.min(Instant::now() + SLICE_TIME);
        let mut steps = 0;
        while self.cursor < work.presentation_fact_count()
            && steps < SLICE_TOKENS
            && Instant::now() < deadline
        {
            steps += 1;
            self.maximum_export_tokens = self.maximum_export_tokens.max(steps);
            if !self.export_one(work, retained) {
                self.publish(work, Some(PresentationOutcome::Limited(Limit::Retention)));
                return true;
            }
            self.cursor += 1;
        }
        if self.cursor == work.presentation_fact_count() {
            self.publish(work, None);
            return true;
        }
        false
    }

    fn export_one(&mut self, work: &Work, retained: usize) -> bool {
        let Some(endpoint) = work.presentation_endpoint(self.cursor) else {
            return true;
        };
        if self.build.len() == self.build.capacity() && !self.reserve_span(retained) {
            return false;
        }
        self.build.push(endpoint);
        true
    }

    fn reserve_span(&mut self, retained: usize) -> bool {
        let available = MAX_METADATA.saturating_sub(retained + self.heap());
        let extra = (available / std::mem::size_of::<Delimiter>()).min(SLICE_TOKENS);
        extra > 0 && self.build.try_reserve_exact(extra).is_ok()
    }

    fn publish(&mut self, work: &Work, limited: Option<PresentationOutcome>) {
        self.publication = self
            .publication
            .checked_add(1)
            .expect("pair publication exhausted");
        let answer: &Answer = work.answer.as_ref().expect("completed typed qualification");
        self.snapshot = Some(Snapshot {
            stamp: answer.stamp,
            profile: answer.profile,
            provenance: answer.provenance,
            spans: std::mem::take(&mut self.build),
            inspected: work.inspected_to(),
            target: self.target,
            outcome: limited.unwrap_or_else(|| PresentationOutcome::from_pair(&answer.outcome)),
            stats: Stats {
                slices: answer.stats.slices + self.export_slices,
                maximum_slice_tokens: answer
                    .stats
                    .maximum_slice_tokens
                    .max(self.maximum_export_tokens),
                ..answer.stats
            },
        });
        self.export_slices = 0;
        self.maximum_export_tokens = 0;
    }

    pub fn answer(&self, ticket: PresentationTicket, retained: usize) -> Option<Presentation<'_>> {
        if !self.ready(ticket) {
            return None;
        }
        let range = self.range(ticket)?;
        let snapshot = self.snapshot.as_ref()?;
        let start = snapshot.spans.partition_point(|s| s.at.0 < range.start);
        let end = snapshot.spans.partition_point(|s| s.at.0 < range.end);
        let outcome = if end - start > MAX_VISIBLE_SPANS {
            PresentationOutcome::Limited(Limit::Retention)
        } else {
            snapshot.outcome
        };
        Some(Presentation {
            stamp: snapshot.stamp,
            view: ticket.view,
            profile: snapshot.profile,
            policy_version: POLICY_VERSION,
            provenance: snapshot.provenance,
            outcome,
            spans: &snapshot.spans[start..end.min(start + MAX_VISIBLE_SPANS)],
            stats: Stats {
                retained_bytes: retained,
                ..snapshot.stats
            },
        })
    }

    pub fn invalidate_views(&mut self) {
        self.entries.clear();
    }

    pub fn map_snapshot(&mut self, advance: Option<super::Advance>) {
        self.invalidate_views();
        let Some(proof) = advance else {
            self.abandon();
            return;
        };
        let Some(snapshot) = &mut self.snapshot else {
            return;
        };
        if snapshot.stamp != proof.before || self.publication != proof.publication {
            self.abandon();
            return;
        }
        for endpoint in &mut snapshot.spans {
            endpoint.at.0 = proof.map(endpoint.at.0).expect("prepared endpoint map");
            endpoint.mate.0 = proof.map(endpoint.mate.0).expect("prepared mate map");
        }
        snapshot.inspected = proof
            .map(snapshot.inspected)
            .expect("prepared coverage map");
        snapshot.target = proof.map(snapshot.target).expect("prepared target map");
        snapshot.stamp = proof.after;
        snapshot.provenance = Provenance::MappedLexical;
    }
}

impl Snapshot {
    pub(super) fn outcome(&self) -> PresentationOutcome {
        self.outcome
    }
    pub(super) fn target(&self) -> usize {
        self.target
    }
}
