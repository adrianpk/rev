//! Effect ownership stays with the checked atomic file publisher.
use super::*;
use crate::{
    buffer::{Buffer, Save},
    file::{self, Location, Outcome},
};
pub(crate) struct Prepared {
    pub snapshot: Snapshot,
    pub target: Location,
    pub rendered: Rendered,
    pub pdf: Option<Vec<u8>>,
}
pub(crate) struct Publication {
    snapshot: Snapshot,
    save: Save,
    diagnostics: Vec<String>,
    pdf: Option<Vec<u8>>,
}
pub(crate) struct Published {
    pub outcome: Outcome,
    pub diagnostics: Vec<String>,
}
pub(crate) enum Output {
    Prepared(Box<Prepared>),
    Published(Published),
}
pub(crate) enum Work {
    Prepare(Box<super::Work>, Location),
    Publish(Box<Publication>),
    Compile(Box<super::compiler::Compilation>),
}
impl Work {
    pub fn effect(&self) -> bool {
        matches!(self, Self::Publish(_))
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Output, String> {
        match self {
            Self::Prepare(work, target) => {
                let snapshot = work.run(cancel)?;
                let rendered = snapshot.render(cancel)?;
                protect(&snapshot, &target, cancel)?;
                Ok(Output::Prepared(Box::new(Prepared {
                    snapshot,
                    target,
                    rendered,
                    pdf: None,
                })))
            }
            Self::Compile(work) => Ok(Output::Prepared(Box::new(work.run(cancel)?))),
            Self::Publish(work) => Ok(Output::Published(work.run(cancel))),
        }
    }
}
impl Prepared {
    pub(super) fn protect(&self, cancel: &Cancellation) -> Result<(), String> {
        protect(&self.snapshot, &self.target, cancel)
    }
    pub fn publication(self, buffer: &Buffer) -> Publication {
        Publication {
            snapshot: self.snapshot,
            save: Save::projection(buffer, self.rendered.text, self.target),
            diagnostics: self.rendered.diagnostics,
            pdf: self.pdf,
        }
    }
}
impl Publication {
    pub fn target(&self) -> &Location {
        &self.save.target
    }
    fn run(self, cancel: &Cancellation) -> Published {
        let guard = || protect(&self.snapshot, &self.save.target, cancel);
        let outcome = match guard() {
            Ok(()) => match &self.pdf {
                Some(bytes) => {
                    crate::org::io::publication::publish_bytes(&self.save, bytes, cancel, guard)
                }
                None => crate::org::io::publication::publish(&self.save, cancel, guard),
            },
            Err(e) => Outcome::failed(self.save.target(), std::io::Error::other(e)),
        };
        Published {
            outcome,
            diagnostics: self.diagnostics,
        }
    }
}
fn protect(snapshot: &Snapshot, target: &Location, cancel: &Cancellation) -> Result<(), String> {
    snapshot.files_current(cancel)?;
    let identity = target.observation().map(|o| (o.device, o.inode));
    for source in &snapshot.sources {
        document::check(cancel)?;
        let Some(path) = source.path() else {
            continue;
        };
        if path == target.path() {
            return Err("Export destination is an input source; choose another path".into());
        }
        let actual = file::observe(path)
            .map_err(|e| e.to_string())?
            .map(|o| (o.device, o.inode));
        if identity.is_some() && identity == actual {
            return Err("Export destination aliases an input source; choose another path".into());
        }
    }
    Ok(())
}
