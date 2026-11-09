//! Experimental, language-independent `rev.plugins/0.1` client contract.
//!
//! The host sends [`methods::Initialize`] first. A package acknowledges the
//! session, generation, exact API, implemented capabilities, scalar units and
//! finite profile before issuing requests. Standard output is reserved for
//! UTF-8 JSON-RPC 2.0 objects, one per line; standard error is for diagnostics.
//! Batch arrays, numeric request IDs and unsupported revisions are rejected.
//!
//! [`client::Client`] owns bounded correlation and output queues. Its incremental
//! input and output methods perform no I/O or background work. The caller owns
//! pipe readiness, deadlines, cancellation and process lifetime. In particular,
//! callers must not block an editor's input thread on plugin I/O.
//!
//! Source positions and half-open ranges count Unicode scalars, not bytes,
//! graphemes, UTF-16 units or terminal cells. Revisions and generations are
//! decimal strings; handles are qualified by their editor session. No private
//! editor crate, pointer, rope or native callback crosses this boundary.
//!
//! Schema availability does not advertise host implementation. Call only methods
//! whose capability was supplied in `initialize`. Native source/path/history,
//! keymap, resource and permission checks remain authoritative at execution.
#![forbid(unsafe_code)]

pub mod client;
pub mod identity;
pub mod limits;
pub mod methods;
pub mod rpc;
pub mod transport;

/// Exact experimental host API revision, independent of editor and SDK versions.
pub const API: &str = "rev.plugins/0.1";
