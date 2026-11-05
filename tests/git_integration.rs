#![forbid(unsafe_code)]
#[path = "git_integration/commits.rs"]
mod commits;
#[path = "git_integration/diffs.rs"]
mod diffs;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "git_integration/index.rs"]
mod index;
#[path = "git_integration/status.rs"]
mod status;
#[path = "git_integration/support.rs"]
mod support;

#[path = "git_integration/history.rs"]
mod history;

#[path = "git_integration/refs.rs"]
mod refs;

#[path = "git_integration/transport.rs"]
mod transport;

#[path = "git_integration/integration.rs"]
mod integration;
