//! Explicit bounded clock state IO. This store has no source edit authority:
//! restoring requires separately checked canonical source and user selection.
mod clock;
pub(in crate::org) mod publication;
pub use clock::{ClockState, ClockStore};

mod capture;
pub(in crate::org) use capture::Sources;
pub(in crate::org) mod source;
pub use capture::{CaptureRead, DestinationSource, DestinationVersion};

pub(crate) mod process;

pub(in crate::org) mod bytes;
