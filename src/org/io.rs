//! Explicit bounded clock state IO. This store has no source edit authority:
//! restoring requires separately checked canonical source and user selection.
mod clock;
pub use clock::{ClockState, ClockStore};
