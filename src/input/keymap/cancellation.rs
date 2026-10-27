use crate::command::{Binding, Command};

use super::{Context, Error, Sequence, Site};

pub(super) fn admit(
    context: Context,
    spelling: &str,
    sequence: &Sequence,
    target: Binding,
) -> Result<(), Error> {
    if !sequence.chords().iter().any(|key| key.control('g')) {
        return Ok(());
    }
    if sequence.chords().len() != 1 {
        return Err(Error::new(
            context,
            spelling,
            Site::Key,
            "C-g cancels pending prefixes; a sequence containing it cannot complete",
        ));
    }
    if target != Binding::Command(Command::KeyboardQuit) {
        return Err(Error::new(
            context,
            spelling,
            Site::Value,
            "C-g is reserved for keyboard-quit in every context",
        ));
    }
    Ok(())
}
