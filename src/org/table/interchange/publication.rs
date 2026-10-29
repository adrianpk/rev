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
struct Checked<'a>(&'a Cancellation);
impl Checked<'_> {
    fn check(&self) -> io::Result<()> {
        super::super::check(self.0).map_err(io::Error::other)
    }
}
impl Output for Checked<'_> {
    fn write(&mut self, save: &Save, file: &mut File) -> io::Result<()> {
        save.write(&mut Writer {
            file,
            cancel: self.0,
        })
    }
    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        self.check()?;
        fs::rename(temporary, target)
    }
}
struct Writer<'a> {
    file: &'a mut File,
    cancel: &'a Cancellation,
}
impl Write for Writer<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        super::super::check(self.cancel).map_err(io::Error::other)?;
        self.file.write(&bytes[..bytes.len().min(8192)])
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
pub(super) fn publish(save: &Save, cancel: &Cancellation) -> Outcome {
    file::publish(save, &mut Checked(cancel))
}
