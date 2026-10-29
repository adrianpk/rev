use crate::{
    buffer::Save,
    file::{self, Outcome, Output},
    org::Cancellation,
};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
struct Checked<'a, F> {
    cancel: &'a Cancellation,
    guard: F,
}
impl<F: Fn() -> Result<(), String>> Output for Checked<'_, F> {
    fn write(&mut self, save: &Save, file: &mut File) -> io::Result<()> {
        check(self.cancel)?;
        (self.guard)().map_err(io::Error::other)?;
        save.write(&mut Writer {
            file,
            cancel: self.cancel,
        })
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        check(self.cancel)?;
        (self.guard)().map_err(io::Error::other)?;
        fs::rename(temporary, target)
    }
}
struct Writer<'a> {
    file: &'a mut File,
    cancel: &'a Cancellation,
}
impl Write for Writer<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        check(self.cancel)?;
        self.file.write(&bytes[..bytes.len().min(8192)])
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
pub(in crate::org) fn publish(
    save: &Save,
    cancel: &Cancellation,
    guard: impl Fn() -> Result<(), String>,
) -> Outcome {
    file::publish(save, &mut Checked { cancel, guard })
}
fn check(cancel: &Cancellation) -> io::Result<()> {
    if cancel.is_cancelled() {
        return Err(io::Error::other("Org file work cancelled or expired"));
    }
    Ok(())
}
