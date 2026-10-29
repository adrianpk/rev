use super::*;
use crate::{
    buffer::Save,
    file::{self, Location},
    org::attachment::directory::Directory,
};
pub(crate) struct Prepared {
    pub target: Location,
    text: String,
    directory: Directory,
    mkdir: bool,
    inputs: std::sync::Arc<[PathBuf]>,
}
pub(crate) struct Write {
    prepared: Prepared,
    save: Save,
}
impl Prepared {
    pub fn new(
        target: Target,
        inputs: std::sync::Arc<[PathBuf]>,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        leaf(&target.path)?;
        let directory =
            Directory::observe(target.path.parent().ok_or("Tangle target has no parent")?)?;
        if !target.mkdir {
            directory.open()?;
        }
        let prepared = Self {
            target: file::resolve(&target.path).map_err(|e| e.to_string())?,
            text: target.text,
            directory,
            mkdir: target.mkdir,
            inputs,
        };
        prepared.guard(cancel)?;
        Ok(prepared)
    }
    pub fn failed(&self, detail: &str) -> Outcome {
        Outcome {
            target: self.target.path().into(),
            publication: Publication::NotPublished,
            detail: detail.into(),
        }
    }
    pub fn write(self, buffer: &Buffer) -> Write {
        let save = Save::projection(buffer, String::new(), self.target.clone());
        Write {
            prepared: self,
            save,
        }
    }
    fn guard(&self, cancel: &Cancellation) -> Result<(), String> {
        options::check(cancel)?;
        leaf(self.target.path())?;
        let identity = self.target.observation().map(|o| (o.device, o.inode));
        for input in self.inputs.iter() {
            options::check(cancel)?;
            if input == self.target.path() {
                return Err("Tangle destination is a live source document".into());
            }
            let current = file::resolve(input).map_err(|e| e.to_string())?;
            if identity.is_some() && identity == current.observation().map(|o| (o.device, o.inode))
            {
                return Err("Tangle destination aliases a live source document".into());
            }
        }
        Ok(())
    }
}
impl Write {
    pub fn run(mut self, cancel: &Cancellation) -> Outcome {
        match self.publish(cancel) {
            Ok(outcome) => outcome,
            Err(e) => self.prepared.failed(&e),
        }
    }
    fn publish(&mut self, cancel: &Cancellation) -> Result<Outcome, String> {
        self.prepared.guard(cancel)?;
        let directory = self.prepared.directory.updated()?;
        let pinned = if self.prepared.mkdir {
            directory.materialize(&mut Vec::new())?
        } else {
            directory.open()?
        };
        self.save.target = self
            .prepared
            .target
            .relocated(pinned.entry(self.prepared.target.basename()));
        let guard = || {
            pinned.current()?;
            self.prepared.guard(cancel)
        };
        let result = crate::org::io::publication::publish_bytes(
            &self.save,
            self.prepared.text.as_bytes(),
            cancel,
            guard,
        );
        Ok(Outcome {
            target: self.prepared.target.path().into(),
            publication: result.publication(),
            detail: result.error().map(ToString::to_string).unwrap_or_default(),
        })
    }
}
fn leaf(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() => Err("Tangle destination is a symlink or non-regular file".into()),
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    }
}
