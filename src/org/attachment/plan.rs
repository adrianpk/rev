use super::{directory::Directory, *};
use crate::{
    file::{self, Observation},
    org::Cancellation,
};
use std::{io::Read, path::Path};
pub(crate) struct Plan {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub id: Option<String>,
    pub method: Method,
    pub(super) root: Directory,
    pub(super) source_parent: Directory,
    pub(super) observed: Observation,
    pub(super) collision: Option<Observation>,
    pub(super) limit: usize,
}
impl Plan {
    pub(crate) fn prepare(
        context: Context,
        path: &str,
        method: Method,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        if cancel.is_cancelled() {
            return Err("Attachment selection cancelled".into());
        }
        let selected = Selected::observe(&context, path)?;
        let source = selected.source;
        let source_parent = selected.parent;
        let observed = selected.observed;
        let name = selected.name;
        let id = context.new_id()?;
        let root = Directory::observe(&context.root(id.as_deref())?)?;
        let destination = root.path.join(name);
        if source == destination {
            return Err("Attachment input and destination are the same file".into());
        }
        let collision = file::observe(&destination).map_err(|e| e.to_string())?;
        same_file(&observed, collision.as_ref())?;
        Ok(Self {
            source,
            destination,
            id,
            method,
            root,
            source_parent,
            observed,
            collision,
            limit: context.policy.limits.agenda_file_bytes,
        })
    }
    pub(crate) fn replacing(&self) -> bool {
        self.collision.is_some()
    }
    pub(super) fn check_source(&self) -> Result<(), String> {
        self.source_parent.check()?;
        if file::observe(&self.source)
            .map_err(|e| e.to_string())?
            .as_ref()
            != Some(&self.observed)
        {
            return Err("Attachment input changed since selection".into());
        }
        Ok(())
    }
    pub(super) fn check_target(&self, path: &Path) -> Result<(), String> {
        if file::observe(path).map_err(|e| e.to_string())? != self.collision {
            return Err("Attachment destination changed since confirmation".into());
        }
        Ok(())
    }
}
fn same_file(source: &Observation, target: Option<&Observation>) -> Result<(), String> {
    if target.is_some_and(|t| t.device == source.device && t.inode == source.inode) {
        return Err("Attachment input and destination alias the same file".into());
    }
    Ok(())
}
fn identifier() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|e| format!("Create attachment ID: {e}"))?;
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

impl Context {
    fn new_id(&self) -> Result<Option<String>, String> {
        if self.dir.is_none() && self.id.is_none() {
            Ok(Some(identifier()?))
        } else {
            Ok(None)
        }
    }
}
struct Selected {
    source: PathBuf,
    parent: Directory,
    observed: Observation,
    name: String,
}
impl Selected {
    fn observe(context: &Context, path: &str) -> Result<Self, String> {
        let source = location::local(&context.base, path)?;
        let source_parent =
            Directory::observe(source.parent().ok_or("Attachment input has no parent")?)?;
        let _pin = source_parent.open()?;
        let observed = file::observe(&source)
            .map_err(|e| e.to_string())?
            .ok_or("Selected attachment does not exist")?;
        if observed.length > context.policy.limits.agenda_file_bytes as u64 {
            return Err("Attachment exceeds configured Org file byte limit".into());
        }
        let name = source
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Attachment filename is not UTF-8")?;
        location::name(name)?;
        let name = name.to_owned();
        Ok(Self {
            source,
            parent: source_parent,
            observed,
            name,
        })
    }
}
