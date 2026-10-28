//! Derived clock observations retain canonical provenance, never edit authority.
pub(crate) mod clocks;
pub use clocks::{ClockQuery, ClockRow, ClockScope, ClockSummary};
