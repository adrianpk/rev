#![forbid(unsafe_code)]

#[path = "plugins/protocol.rs"]
mod protocol;

#[path = "plugins/declarations.rs"]
mod declarations;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "plugins/lifecycle.rs"]
mod lifecycle;

#[path = "plugins/buffers.rs"]
mod buffers;

#[path = "plugins/transactions.rs"]
mod transactions;

#[path = "plugins/invocation.rs"]
mod invocation;
