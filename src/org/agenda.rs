//! Derived clock observations retain canonical provenance, never edit authority.
pub(crate) mod clocks;
pub use clocks::{ClockQuery, ClockRow, ClockScope, ClockSummary};
mod table;
pub use table::ClockTable;
mod render;

mod scan;
pub use scan::{ClockRequest, ClockScan};
mod update;
pub(crate) use update::ClockUpdate;
