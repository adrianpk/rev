use std::fs;
use std::io;

use crate::file::{Deleted, Deletion};

use super::{Buffer, Buffers};

impl Buffers {
    /// Retain canonical text, history and path reservations. Only observed
    /// absent associations change; a failed recursive operation can be partial.
    pub fn delete(&mut self, deletion: Deletion) -> Deleted {
        let error = deletion.publish().err();
        let mut warning = None;
        for buffer in self.entries.values_mut() {
            if let Err(error) = reconcile(buffer, &deletion) {
                warning.get_or_insert_with(|| error.to_string());
            }
        }
        Deleted {
            source: deletion.source().into(),
            error,
            warning,
        }
    }
}

fn reconcile(buffer: &mut Buffer, deletion: &Deletion) -> io::Result<()> {
    let Some(location) = &buffer.file else {
        return Ok(());
    };
    if !deletion.includes(location.path()) {
        return Ok(());
    }
    match fs::symlink_metadata(location.path()) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => missing(buffer),
        Err(error) => {
            return Err(io::Error::new(
                error.kind(),
                format!("observe deleted {}: {error}", location.path().display()),
            ));
        }
        Ok(_) => {}
    }
    Ok(())
}

fn missing(buffer: &mut Buffer) {
    let location = buffer.file.as_ref().expect("visiting association admitted");
    if !location.is_present() {
        return;
    }
    let notice = format!(
        "Deleted on disk; explicit save required: {}",
        location.path().display()
    );
    buffer.file = Some(location.missing());
    let prior = buffer.warning.take().unwrap_or_default();
    buffer.warning = Some(format!("{prior} {notice}").trim().into());
    buffer.break_typing();
}
