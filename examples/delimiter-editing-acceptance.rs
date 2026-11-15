#![forbid(unsafe_code)]

#[path = "delimiter_editing_acceptance/journey.rs"]
mod journey;
#[allow(
    dead_code,
    reason = "Shared native settling and Linux process observations."
)]
#[path = "completion_acceptance/support.rs"]
mod support;
#[path = "completion_acceptance/terminal.rs"]
mod terminal;

fn main() -> support::Result<()> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let [root, binary, wrapper, profile] = arguments.as_slice() else {
        return Err("expected ROOT EDITOR WRAPPER PROFILE".into());
    };
    journey::run(
        std::path::Path::new(root),
        std::path::Path::new(binary),
        std::path::Path::new(wrapper),
        profile.to_str().ok_or("invalid profile")?,
    )
}
