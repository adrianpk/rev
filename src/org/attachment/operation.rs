mod output;
use super::{
    directory::{self, Pinned},
    *,
};
use crate::{
    file::{self, temporary::Temporary},
    org::Cancellation,
};
pub(super) use output::{Files, Output};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Rejected,
    Published,
    Removed,
}
pub struct FileOutcome {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub id: Option<String>,
    pub effect: Effect,
    pub created: Vec<PathBuf>,
    pub retained: Option<PathBuf>,
    pub error: Option<String>,
}
impl Plan {
    pub(crate) fn run(self, cancel: &Cancellation) -> FileOutcome {
        self.run_with(cancel, &mut Files)
    }
    pub(super) fn run_with(self, cancel: &Cancellation, output: &mut impl Output) -> FileOutcome {
        let mut result = FileOutcome {
            source: self.source.clone(),
            destination: self.destination.clone(),
            id: self.id.clone(),
            effect: Effect::Rejected,
            created: Vec::new(),
            retained: None,
            error: None,
        };
        if let Err(e) = self.perform(cancel, &mut result, output) {
            result.error = Some(e);
        }
        result
    }
    fn perform(
        &self,
        cancel: &Cancellation,
        result: &mut FileOutcome,
        output: &mut impl Output,
    ) -> Result<(), String> {
        active(cancel)?;
        self.check_source()?;
        let root = self.root.materialize(&mut result.created)?;
        self.install(&root, cancel, result, output)?;
        output
            .sync_directory(&root.handle)
            .map_err(|e| format!("Attachment published; directory sync failed: {e}"))?;
        if self.method == Method::Move {
            self.remove_source(cancel, result, output)?;
        }
        Ok(())
    }
    fn install(
        &self,
        root: &Pinned,
        cancel: &Cancellation,
        result: &mut FileOutcome,
        output: &mut impl Output,
    ) -> Result<(), String> {
        let path = root.entry(
            self.destination
                .file_name()
                .ok_or("Attachment filename missing")?,
        );
        self.check_target(&path)?;
        let mut temp = self.temporary(root)?;
        let staged = self.stage(&mut temp, cancel, output);
        if let Err(error) = staged {
            return Err(temp
                .failed(std::io::Error::other(error), &mut file::Files)
                .to_string());
        }
        let publish = self.publish(root, &path, &mut temp, cancel);
        let cleanup = match publish {
            Ok(cleanup) => cleanup,
            Err(error) => {
                return Err(temp
                    .failed(std::io::Error::other(error), &mut file::Files)
                    .to_string());
            }
        };
        result.effect = Effect::Published;
        if let Some(error) = cleanup {
            result.retained = Some(
                self.root
                    .path
                    .join(temp.path.file_name().ok_or("Staging name missing")?),
            );
            return Err(error);
        }
        Ok(())
    }
    fn remove_source(
        &self,
        cancel: &Cancellation,
        result: &mut FileOutcome,
        output: &mut impl Output,
    ) -> Result<(), String> {
        active(cancel)?;
        self.check_source()?;
        let parent = self.source_parent.open()?;
        parent.current()?;
        output
            .remove_input(
                &parent.entry(
                    self.source
                        .file_name()
                        .ok_or("Attachment input name missing")?,
                ),
            )
            .map_err(|e| format!("Attachment published; input retained: {e}"))?;
        result.effect = Effect::Removed;
        output
            .sync_directory(&parent.handle)
            .map_err(|e| format!("Input removed; directory sync failed: {e}"))?;
        Ok(())
    }
    fn temporary(&self, root: &Pinned) -> Result<Temporary, String> {
        let parent = root.entry(std::ffi::OsStr::new("."));
        if self.method == Method::Link {
            self.check_source()?;
            let input = self.checked_input()?;
            Temporary::link(&parent, &self.source, &input)
                .map_err(|e| format!("Hard-link attachment: {e}"))
        } else {
            Temporary::new(&parent).map_err(|e| e.to_string())
        }
    }
    fn stage(
        &self,
        temp: &mut Temporary,
        cancel: &Cancellation,
        output: &mut impl Output,
    ) -> Result<(), String> {
        if self.method == Method::Link {
            return self.stage_link(temp);
        }
        self.check_source()?;
        let mut input = self.checked_input()?;
        copy(&mut input, &mut temp.file, self.limit, cancel, output)?;
        temp.file.sync_all().map_err(|e| e.to_string())
    }
    fn checked_input(&self) -> Result<File, String> {
        let input = directory::input(&self.source)?;
        let observed =
            file::Observation::from_metadata(input.metadata().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if observed != self.observed {
            return Err("Opened attachment input changed".into());
        }
        Ok(input)
    }
    fn stage_link(&self, temp: &Temporary) -> Result<(), String> {
        let linked = file::observe(&temp.path)
            .map_err(|e| e.to_string())?
            .ok_or("Staged link disappeared")?;
        if !self.observed.same_content(&linked) {
            return Err("Hard-link input changed while linking".into());
        }
        Ok(())
    }
    fn check_staged_source(&self, temp: &Temporary) -> Result<(), String> {
        if self.method != Method::Link {
            return self.check_source();
        }
        let observed = file::observe(&temp.path)
            .map_err(|e| e.to_string())?
            .ok_or("Staged hard link disappeared")?;
        if !self.observed.same_content(&observed) {
            return Err("Hard-link input changed before publication".into());
        }
        Ok(())
    }
    fn publish(
        &self,
        root: &Pinned,
        path: &Path,
        temp: &mut Temporary,
        cancel: &Cancellation,
    ) -> Result<Option<String>, String> {
        active(cancel)?;
        root.current()?;
        self.check_target(path)?;
        self.check_staged_source(temp)?;
        if self.collision.is_some() {
            fs::rename(&temp.path, path).map_err(|e| e.to_string())?;
        } else {
            fs::hard_link(&temp.path, path)
                .map_err(|e| format!("Publish attachment without overwrite: {e}"))?;
            // Publication has occurred even if removing the owned staging name fails.
            if let Err(e) = fs::remove_file(&temp.path) {
                temp.disarm();
                return Ok(Some(format!(
                    "Attachment published; staged name retained {:?}: {e}",
                    temp.path
                )));
            }
        }
        temp.disarm();
        Ok(None)
    }
}
fn active(cancel: &Cancellation) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Attachment operation cancelled".into())
    } else {
        Ok(())
    }
}
fn copy(
    input: &mut File,
    output: &mut File,
    limit: usize,
    cancel: &Cancellation,
    io: &mut impl Output,
) -> Result<(), String> {
    let mut total = 0usize;
    let mut bytes = [0u8; 32768];
    loop {
        active(cancel)?;
        let n = input.read(&mut bytes).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n)
            .filter(|n| *n <= limit)
            .ok_or("Attachment byte limit reached")?;
        io.write(output, &bytes[..n]).map_err(|e| e.to_string())?;
    }
    output.flush().map_err(|e| e.to_string())
}
