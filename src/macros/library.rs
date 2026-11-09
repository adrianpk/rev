//! Explicit, inert schema-1/2 snapshots. This owner never executes saved input.
mod codec;
mod store;
pub(crate) use codec::{Catalog, Record};
pub(crate) use store::Store;

use super::*;
use crate::input::keymap::Sequence;

#[derive(Clone)]
pub(crate) struct Saved {
    pub id: String,
    pub revision: u64,
    pub baseline: Arc<Definition>,
    pub description: Option<String>,
    pub binding: Option<Sequence>,
    pub active_binding: Option<Sequence>,
}
impl Saved {
    pub fn clean(&self, named: &Named, binding: Option<&Sequence>) -> bool {
        Arc::ptr_eq(&self.baseline.steps, &named.definition.steps)
            && self.baseline.entry == named.definition.entry
            && self.baseline.counter == named.definition.counter
            && self.baseline.initial_format == named.definition.initial_format
            && self.description == named.description
            && self.active_binding.as_ref() == binding
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Invalid(&'static str),
    Capacity(&'static str),
    Conflict(&'static str),
    Unavailable(&'static str),
    Busy,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(s) => write!(f, "Invalid macro library: {s}"),
            Self::Capacity(s) => write!(f, "Macro library limit: {s}"),
            Self::Conflict(s) => write!(f, "Macro library conflict: {s}"),
            Self::Unavailable(s) => write!(f, "Macro library unavailable: {s}"),
            Self::Busy => f.write_str("Macro library is busy; no change was written"),
        }
    }
}

#[cfg(test)]
mod store_tests;
