//! Read-only endpoint publications from the buffer's existing typed fact owner.
mod advance;
mod store;

pub(crate) use advance::Advance;
pub(super) use store::Store;

use super::{Kind, Limit, Outcome, Profile, Provenance, Reason, Stats};
use crate::buffer::Stamp;
use crate::coordinates::Scalar;
use crate::window::ViewId;

pub const MAX_VISIBLE_SPANS: usize = 8_192;

/// Both endpoints and nesting were confirmed by the native typed lexical pass.
/// Coordinates are scalar offsets qualified by their containing publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delimiter {
    pub at: Scalar,
    pub mate: Scalar,
    pub kind: Kind,
    pub depth: usize,
    pub opening: bool,
}

#[derive(Clone, Copy)]
pub(super) struct PendingDelimiter {
    pub at: Scalar,
    pub mate: Option<Scalar>,
    pub kind: Kind,
    pub depth: usize,
    pub opening: bool,
}

impl PendingDelimiter {
    pub fn confirmed(self) -> Option<Delimiter> {
        Some(Delimiter {
            at: self.at,
            mate: self.mate?,
            kind: self.kind,
            depth: self.depth,
            opening: self.opening,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationOutcome {
    Complete,
    Unmatched,
    Mismatched { expected: Kind, found: Kind },
    Uncertain(Reason),
    Limited(Limit),
}

impl PresentationOutcome {
    pub(super) fn from_pair(outcome: &Outcome) -> Self {
        match *outcome {
            Outcome::Mismatched { expected, found } => Self::Mismatched { expected, found },
            Outcome::Unmatched => Self::Unmatched,
            Outcome::Uncertain(reason) => Self::Uncertain(reason),
            Outcome::Limited(limit) => Self::Limited(limit),
            _ => Self::Complete,
        }
    }
}

/// View identity and generation do not authorize use after source publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentationTicket {
    pub stamp: Stamp,
    pub view: ViewId,
    pub(super) generation: u64,
}

/// Shared immutable spans, sliced to one captured view's visible source range.
/// The borrow cannot survive native source publication or view cancellation.
#[derive(Debug)]
pub struct Presentation<'a> {
    pub stamp: Stamp,
    pub view: ViewId,
    pub profile: Profile,
    pub policy_version: u32,
    pub provenance: Provenance,
    pub outcome: PresentationOutcome,
    pub spans: &'a [Delimiter],
    pub stats: Stats,
}
