#![forbid(unsafe_code)]

use std::io::{self, Write};

// A standalone native fixture lets the reference guard inspect the actual
// compiled resource without exposing another application command or CLI flag.
fn main() -> io::Result<()> {
    io::stdout()
        .lock()
        .write_all(rev::config::default_template().as_bytes())
}
