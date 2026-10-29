use super::*;
use crate::{buffer::Save, file, org::attachment::directory::Directory};
use std::{io::Read, path::Path};
pub(crate) struct Prepared {
    pub source: PathBuf,
    pub target: Location,
    content: Content,
    directory: Directory,
}
enum Content {
    Document(Box<export::Snapshot>, Vec<u8>, Vec<String>),
    Asset(file::Observation, Vec<u8>),
}
pub(crate) struct Write {
    prepared: Prepared,
    save: Save,
    inputs: Vec<PathBuf>,
}
impl Prepared {
    pub(super) fn document(
        snapshot: export::Snapshot,
        target: &Path,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let (bytes, diagnostics) = if snapshot.request().backend == ExportBackend::Odt {
            let archive = snapshot.package(cancel)?;
            (archive.bytes().to_vec(), archive.diagnostics().to_vec())
        } else {
            let rendered = snapshot.render(cancel)?;
            (
                rendered.text().as_bytes().to_vec(),
                rendered.diagnostics().to_vec(),
            )
        };
        let source = snapshot.sources()[0]
            .path()
            .ok_or("Project export lacks a source path")?
            .to_owned();
        Self::new(
            source,
            target,
            Content::Document(Box::new(snapshot), bytes, diagnostics),
        )
    }
    pub(super) fn asset(
        source: &Path,
        target: &Path,
        limits: &config::OrgLimits,
        budget: &Budget<'_>,
    ) -> Result<Self, String> {
        discover::regular(source)?;
        let before = file::observe(source)
            .map_err(|e| e.to_string())?
            .ok_or("Project asset missing")?;
        let bytes = asset(source, (&before, limits.agenda_file_bytes), budget)?;
        if file::observe(source).map_err(|e| e.to_string())? != Some(before.clone()) {
            return Err("Project asset changed while reading".into());
        }
        Self::new(source.into(), target, Content::Asset(before, bytes))
    }
    fn new(source: PathBuf, target: &Path, content: Content) -> Result<Self, String> {
        let directory =
            Directory::observe(target.parent().ok_or("Project destination has no parent")?)?;
        leaf(target)?;
        let target = file::resolve(target).map_err(|e| e.to_string())?;
        Ok(Self {
            source,
            target,
            content,
            directory,
        })
    }
    pub fn size(&self) -> usize {
        match &self.content {
            Content::Document(snapshot, bytes, _) => {
                bytes.len()
                    + snapshot
                        .sources()
                        .iter()
                        .map(|s| s.text().len_bytes())
                        .sum::<usize>()
            }
            Content::Asset(_, bytes) => bytes.len(),
        }
    }
    pub fn inputs(&self) -> Vec<PathBuf> {
        match &self.content {
            Content::Document(snapshot, _, _) => snapshot
                .sources()
                .iter()
                .filter_map(|s| s.path().map(PathBuf::from))
                .collect(),
            Content::Asset(_, _) => vec![self.source.clone()],
        }
    }
    pub fn evidence(&self) -> Vec<export::Evidence> {
        match &self.content {
            Content::Document(snapshot, _, _) => snapshot.sources().to_vec(),
            Content::Asset(_, _) => Vec::new(),
        }
    }
    pub fn current(&self, buffers: &Buffers) -> Result<(), String> {
        if let Content::Document(snapshot, _, _) = &self.content {
            snapshot.buffers_current(buffers)?;
        }
        Ok(())
    }
    pub fn write(self, buffer: &Buffer, inputs: Vec<PathBuf>) -> Write {
        let save = Save::projection(buffer, String::new(), self.target.clone());
        Write {
            prepared: self,
            save,
            inputs,
        }
    }
    pub fn failed(&self, detail: &str) -> Outcome {
        Outcome::failed(
            self.source.clone(),
            self.target.path().into(),
            detail.into(),
        )
    }
    fn bytes(&self) -> &[u8] {
        match &self.content {
            Content::Document(_, bytes, _) | Content::Asset(_, bytes) => bytes,
        }
    }
    fn fresh(&self, cancel: &Cancellation) -> Result<(), String> {
        if cancel.is_cancelled() {
            return Err("Project publication cancelled or expired".into());
        }
        match &self.content {
            Content::Document(snapshot, _, _) => snapshot.files_current(cancel),
            Content::Asset(before, _) => {
                discover::regular(&self.source)?;
                if file::observe(&self.source)
                    .map_err(|e| e.to_string())?
                    .as_ref()
                    != Some(before)
                {
                    return Err("Project asset changed before publication".into());
                }
                Ok(())
            }
        }
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
        self.guard(cancel)?;
        let mut created = Vec::new();
        let pinned = self
            .prepared
            .directory
            .updated()?
            .materialize(&mut created)?;
        self.save.target = self
            .prepared
            .target
            .relocated(pinned.entry(self.prepared.target.basename()));
        let guard = || {
            pinned.current()?;
            self.guard(cancel)
        };
        let result = crate::org::io::publication::publish_bytes(
            &self.save,
            self.prepared.bytes(),
            cancel,
            guard,
        );
        let mut detail = result.error().map(ToString::to_string).unwrap_or_default();
        if let Content::Document(_, _, diagnostics) = &self.prepared.content {
            detail.push_str(&diagnostics.join("; "));
        }
        Ok(Outcome {
            source: self.prepared.source.clone(),
            target: self.prepared.target.path().into(),
            publication: result.publication(),
            detail,
        })
    }
    fn guard(&self, cancel: &Cancellation) -> Result<(), String> {
        self.prepared.fresh(cancel)?;
        leaf(self.prepared.target.path())?;
        let identity = self
            .prepared
            .target
            .observation()
            .map(|o| (o.device, o.inode));
        for path in &self.inputs {
            if path == self.prepared.target.path() {
                return Err("Project destination is an input source".into());
            }
            // Resolve rejected aliases only to protect their physical inputs.
            // This observation does not admit them for rendering or copying.
            let resolved = file::resolve(path).map_err(|e| e.to_string())?;
            let observed = resolved.observation().map(|o| (o.device, o.inode));
            if identity.is_some() && identity == observed {
                return Err("Project destination aliases an input source".into());
            }
        }
        Ok(())
    }
}
fn leaf(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if !m.is_file() => Err("Project destination is a symlink or non-regular file".into()),
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    }
}
fn asset(
    path: &Path,
    observed: (&file::Observation, usize),
    budget: &Budget<'_>,
) -> Result<Vec<u8>, String> {
    let (before, limit) = observed;
    let mut file = crate::org::attachment::directory::input(path)?;
    if file::Observation::from_metadata(file.metadata().map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?
        != *before
    {
        return Err("Project asset changed before opening".into());
    }
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        budget.check()?;
        let n = file.read(&mut chunk).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(bytes);
        }
        if bytes.len().saturating_add(n) > limit {
            return Err("Project asset exceeds org.limits.agenda_file_bytes".into());
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
}
