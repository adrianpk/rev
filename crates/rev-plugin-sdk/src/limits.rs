//! Normative finite profile. Lower negotiated ceilings are permitted; higher are not.

use serde::{Deserialize, Serialize};

const FRAME_LIMIT: u32 = 4 * 1024 * 1024;
const TEXT_LIMIT: u32 = 512 * 1024;
const STATE_LIMIT: u32 = 256 * 1024;
pub const FRAME_BYTES: usize = FRAME_LIMIT as usize;
pub const TEXT_BYTES: usize = TEXT_LIMIT as usize;
pub const STATE_BYTES: usize = STATE_LIMIT as usize;

macro_rules! profile {
    ($($name:ident = $value:expr),+ $(,)?) => {
        /// All byte counts are UTF-8 bytes; all deadlines are milliseconds.
        #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct Limits { $(pub $name: u32,)+ }
        impl Default for Limits {
            fn default() -> Self { Self { $($name: $value,)+ } }
        }
        impl Limits {
            pub fn validate(&self) -> bool {
                $(self.$name > 0 && self.$name <= $value)&&+
            }
        }
    };
}
profile! {
    packages = 64,
    active_processes = 8,
    commands_per_package = 64,
    commands_total = 512,
    bindings_per_package = 128,
    bindings_total = 512,
    frame_bytes = FRAME_LIMIT,
    text_bytes = TEXT_LIMIT,
    edit_buffers = 16,
    replacements = 256,
    pending_per_package = 16,
    pending_total = 64,
    outbound_per_package = FRAME_LIMIT,
    outbound_total = 16 * 1024 * 1024,
    events_per_package = 256,
    causal_depth = 8,
    startup_ms = 5000,
    command_ms = 60000,
    shutdown_grace_ms = 2000,
    stderr_bytes = 128 * 1024,
    state_record_bytes = STATE_LIMIT,
    state_store_bytes = FRAME_LIMIT,
    state_records = 256,
    service_items = 32,
    service_micros = 2000,
}
