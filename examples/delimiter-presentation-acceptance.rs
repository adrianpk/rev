#![forbid(unsafe_code)]

#[path = "delimiter_presentation_acceptance/fixture.rs"]
mod fixture;
#[path = "delimiter_presentation_acceptance/held.rs"]
mod held;
#[path = "delimiter_presentation_acceptance/journey.rs"]
mod journey;
#[path = "delimiter_presentation_acceptance/observe.rs"]
mod observe;
#[expect(dead_code, reason = "Reuse existing native and Linux process helpers.")]
#[path = "completion_acceptance/support.rs"]
mod support;
#[expect(dead_code, reason = "Shared cell decoder also serves native tests.")]
#[path = "../tests/support/surface.rs"]
mod surface;
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
        profile.to_str().ok_or("invalid interaction profile")?,
    )
}
