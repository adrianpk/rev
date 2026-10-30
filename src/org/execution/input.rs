//! Execution preparation owns immutable inputs; observation never starts tools.
pub(super) mod calls;
use super::*;
use crate::{
    buffer::{Buffers, Stamp},
    org::io::{DestinationSource, Sources, source::Budget},
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub(crate) struct Input {
    pub definition: Definition,
    pub stamp: Stamp,
    pub site: super::results::Site,
    pub related: Vec<DestinationSource>,
    pub source_path: Option<PathBuf>,
    pub scan: Scan,
    pub query: bool,
}
pub(crate) struct Prepare {
    scan: Scan,
    stamp: Stamp,
    path: Option<PathBuf>,
    sources: Sources,
    files: Vec<PathBuf>,
    limits: config::OrgLimits,
}
impl Prepare {
    pub fn new(buffers: &Buffers, buffer: &Buffer, at: Byte) -> Result<Self, String> {
        let snapshot = buffers.snapshot();
        let limits = snapshot.settings().org.limits.clone();
        let base = snapshot
            .origin(&["org", "source", "library_files"])
            .and_then(Path::parent)
            .unwrap_or(Path::new("."));
        let files = snapshot
            .settings()
            .org
            .source
            .library_files
            .iter()
            .map(|p| base.join(p))
            .collect();
        Ok(Self {
            scan: Scan::new(buffer, at)?,
            stamp: buffer.stamp(),
            path: buffer.file().map(|f| f.path().to_owned()),
            sources: Sources::new(buffers, &limits)?,
            files,
            limits,
        })
    }
    pub fn run(self, cancel: &Cancellation) -> Result<Input, String> {
        if let Ok(definition) = self.scan.clone().run(cancel) {
            let site = super::results::Site::block(&self.scan, &definition)?;
            return Ok(Input {
                definition,
                stamp: self.stamp,
                site,
                related: Vec::new(),
                source_path: self.path,
                scan: self.scan,
                query: false,
            });
        }
        self.call(cancel)
    }
    fn call(self, cancel: &Cancellation) -> Result<Input, String> {
        let call = calls::Call::observe(&self.scan, cancel)?;
        let (definition, scan, related, path) = self.resolve(&call, cancel)?;
        // A call override cannot remove the target's execution restriction.
        headers::admit(&definition)?;
        let query = call.restrictions(&self.scan, &definition.language, cancel)?
            || definition.headers.get("eval") == Some("query");
        let mut definition = definition;
        definition.headers.apply(&call.inside, cancel)?;
        call.arguments(&mut definition, cancel)?;
        headers::admit(&definition)?;
        let site = super::results::Site::call(&self.scan, &call, &definition)?;
        Ok(Input {
            definition,
            stamp: self.stamp,
            site,
            related,
            source_path: path,
            scan,
            query,
        })
    }
    fn resolve(&self, call: &calls::Call, cancel: &Cancellation) -> Result<Resolved, String> {
        let blocks = self.scan.clone().all(cancel)?;
        if !call.name.contains(':')
            && let Some(block) = named(&blocks, &call.name)
        {
            return Ok((block, self.scan.clone(), Vec::new(), self.path.clone()));
        }
        self.library(call, cancel)
    }
    fn library(&self, call: &calls::Call, cancel: &Cancellation) -> Result<Resolved, String> {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.limits.job_seconds as u64),
        };
        let mut found = None;
        let mut related = Vec::new();
        let mut bytes = 0;
        for path in &self.files {
            let source = self.sources.read(path, &budget)?;
            bytes += source.text().len_bytes();
            if bytes > self.limits.agenda_total_bytes {
                return Err("Source library total byte limit reached".into());
            }
            let scan = Scan::observed(&source)?;
            let blocks = scan.clone().all(cancel)?;
            if let Some(block) = call
                .library_name(&source.path, self.path.as_deref())
                .and_then(|name| named(&blocks, name))
            {
                if found.is_some() {
                    return Err("Library block name is ambiguous".into());
                }
                found = Some((block, scan, source.path.clone()));
            }
            related.push(source);
        }
        let (definition, scan, path) =
            found.ok_or("Named block not found in local source or declared library_files")?;
        Ok((definition, scan, related, Some(path)))
    }
}
type Resolved = (Definition, Scan, Vec<DestinationSource>, Option<PathBuf>);
fn named(blocks: &[Definition], name: &str) -> Option<Definition> {
    blocks
        .iter()
        .find(|b| b.name.as_deref() == Some(name))
        .cloned()
}
impl Input {
    pub fn current(&self, buffers: &Buffers) -> Result<(), String> {
        let buffer = buffers.get(self.stamp.buffer).map_err(|e| e.to_string())?;
        crate::org::source::current(buffer, self.stamp).map_err(|e| e.to_string())?;
        related_current(&self.related, buffers)
    }
}
pub(super) fn related_current(
    sources: &[DestinationSource],
    buffers: &Buffers,
) -> Result<(), String> {
    for source in sources {
        source.prepare_visit(buffers)?;
    }
    Ok(())
}
