use crate::{
    file::{self, Observation},
    org::Cancellation,
};
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    time::Instant,
};

pub(in crate::org) struct Budget<'a> {
    pub cancel: &'a Cancellation,
    pub deadline: Instant,
}
impl Budget<'_> {
    pub fn check(&self) -> Result<(), String> {
        if self.cancel.is_cancelled() {
            return Err("Agenda discovery cancelled".into());
        }
        if Instant::now() >= self.deadline {
            return Err("Agenda job time limit reached".into());
        }
        Ok(())
    }
}
struct Reader<'a> {
    file: File,
    budget: &'a Budget<'a>,
    limit: usize,
    used: usize,
}
impl Read for Reader<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.budget.check().map_err(io::Error::other)?;
        let count = bytes
            .len()
            .min(8192)
            .min(self.limit.saturating_sub(self.used) + 1);
        let read = self.file.read(&mut bytes[..count])?;
        self.used += read;
        if self.used > self.limit {
            return Err(io::Error::other("Agenda byte limit reached"));
        }
        std::thread::yield_now();
        Ok(read)
    }
}
/// Count bytes even when decoding or consistency checks fail. Each source is
/// published only after whole UTF-8/NUL/BOM ingress and checked disk evidence.
pub(in crate::org) fn source(
    path: &Path,
    limit: usize,
    budget: &Budget<'_>,
) -> (Result<(file::Loaded, Observation), String>, usize) {
    let opened = open(path);
    let (file, before) = match opened {
        Ok(value) => value,
        Err(e) => return (Err(e.to_string()), 0),
    };
    let mut reader = Reader {
        file,
        budget,
        limit,
        used: 0,
    };
    let text = file::Loaded::read(&mut reader);
    let result = text.map_err(|e| e.to_string()).and_then(|loaded| {
        budget.check()?;
        let after = Observation::from_metadata(reader.file.metadata().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if before != after
            || file::observe(path).map_err(|e| e.to_string())? != Some(before.clone())
        {
            return Err("Agenda file changed while reading; refresh required".into());
        }
        Ok((loaded, before))
    });
    (result, reader.used)
}
fn open(path: &Path) -> io::Result<(File, Observation)> {
    let before = file::observe(path)?.ok_or_else(|| io::Error::other("Agenda file is missing"))?;
    let file = File::open(path)?;
    let opened = Observation::from_metadata(file.metadata()?)?;
    if opened != before {
        return Err(io::Error::other("Agenda file changed before reading"));
    }
    Ok((file, before))
}
