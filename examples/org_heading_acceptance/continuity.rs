#[path = "continuity/fixture.rs"]
mod fixture;
#[path = "continuity/native.rs"]
mod native;
#[path = "continuity/pty.rs"]
mod pty;

use crate::support::Result;
pub(crate) use fixture::Format;
use std::path::Path;

pub(crate) fn native(root: &Path, format: Format) -> Result<()> {
    native::run(root, format)
}

pub(crate) fn pty(root: &Path, binary: &Path, wrapper: &Path, format: Format) -> Result<()> {
    pty::run(root, binary, wrapper, format)
}

// These two flags select only this delivery's finite acceptance cases. Existing
// historical journeys retain their original argument and timeout boundaries.
pub(crate) fn requested(args: &[std::path::PathBuf], format: Format) -> Option<Result<()>> {
    match args {
        [flag, root] if flag == "--continuity-native" => Some(native(root, format)),
        [flag, root, binary, wrapper] if flag == "--continuity-pty" => {
            Some(pty(root, binary, wrapper, format))
        }
        _ => None,
    }
}
