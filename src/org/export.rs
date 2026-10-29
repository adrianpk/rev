//! Checked immutable export data. Preparation has no save, edit, destination,
//! process or network authority. Projected offsets never identify canonical text.
pub(crate) mod compiler;
mod current;
mod html;
mod odt;
pub use odt::Archive;
pub(crate) mod output;
mod render;
mod tex;
pub use render::Rendered;
mod document;
mod include;
mod limits;
mod macros;
mod options;
mod scope;
use super::{io::source::Budget, structure};
pub use crate::org::agenda::SourceVersion;
use crate::{
    buffer::{BufferId, Buffers},
    config::{ExportBackend, Org},
    org::{Cancellation, InlineContext, Source, SourceRange, StructureKind, time::ZonedOperation},
};
pub use options::{Metadata, Options};
use ropey::{Rope, RopeSlice};
use std::{
    ops::Range,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub enum Scope {
    Buffer,
    Region(SourceRange),
    Subtree(crate::coordinates::Byte),
}
/// Options supplied here override the file OPTIONS declaration as one explicit
/// selection. Without an override the checked file declaration supplies them.
#[derive(Clone, Debug)]
pub struct Request {
    pub backend: ExportBackend,
    pub scope: Scope,
    pub options: Option<Options>,
    pub time: ZonedOperation,
}
#[derive(Clone, Debug)]
pub struct Evidence {
    path: Option<PathBuf>,
    version: SourceVersion,
    bom: bool,
    text: Rope,
}
impl Evidence {
    pub fn has_bom(&self) -> bool {
        self.bom
    }
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    pub fn version(&self) -> &SourceVersion {
        &self.version
    }
    pub fn text(&self) -> RopeSlice<'_> {
        self.text.slice(..)
    }
}
/// Byte ranges refer exclusively to the projected export text. Original source
/// revisions remain separately available as Evidence, never manufactured stamps.
pub struct Element<'a> {
    pub kind: StructureKind,
    pub source: Range<usize>,
    pub content: Range<usize>,
    pub opening: Option<Range<usize>>,
    pub closing: Option<Range<usize>>,
    pub name: Option<Range<usize>>,
    pub affiliated: Option<Range<usize>>,
    pub parent: Option<usize>,
    pub context: Option<InlineContext>,
    pub text: RopeSlice<'a>,
}
pub struct Snapshot {
    text: Rope,
    nodes: Vec<structure::Raw>,
    sources: Vec<Evidence>,
    request: Request,
    metadata: Metadata,
    options: Options,
    export_bytes: usize,
    job_seconds: usize,
    todo: Vec<String>,
    diagnostics: Vec<String>,
}
impl Snapshot {
    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
    pub fn text(&self) -> RopeSlice<'_> {
        self.text.slice(..)
    }
    pub fn sources(&self) -> &[Evidence] {
        &self.sources
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn options(&self) -> &Options {
        &self.options
    }
    pub fn export_bytes(&self) -> usize {
        self.export_bytes
    }
    pub fn elements(&self) -> impl Iterator<Item = Element<'_>> {
        self.nodes.iter().map(|n| Element {
            kind: n.kind,
            source: n.source.clone(),
            content: n.content.clone(),
            opening: n.opening.clone(),
            closing: n.closing.clone(),
            name: n.name.clone(),
            affiliated: n.affiliated.clone(),
            parent: n.parent,
            context: n.context,
            text: self.text.byte_slice(n.content.clone()),
        })
    }
}
pub struct Work {
    reader: super::io::Sources,
    source: Evidence,
    scope: Range<usize>,
    request: Request,
    policy: Org,
}
impl Work {
    /// Capture current canonical unsaved text without IO, a visit or a save.
    pub fn new(
        buffers: &Buffers,
        id: BufferId,
        request: Request,
        policy: &Org,
    ) -> Result<Self, String> {
        policy.export.validate()?;
        limits::validate(&policy.limits)?;
        if !policy.export.backends.contains(&request.backend) {
            return Err("Selected native export backend is disabled".into());
        }
        let buffer = buffers.get(id).map_err(|e| e.to_string())?;
        let text = Source::new(buffer).snapshot().map_err(|e| e.to_string())?;
        if text.len_bytes() > super::Limits::default().bytes {
            return Err("Export source exceeds native analysis bytes".into());
        }
        if text.len_bytes() + usize::from(buffer.has_bom()) * 3 > policy.limits.agenda_file_bytes {
            return Err("Export source byte limit reached".into());
        }
        let scope = scope::capture(buffer, &request.scope)?;
        Ok(Self {
            reader: super::io::Sources::new(buffers, &policy.limits)?,
            source: Evidence {
                path: buffer.file().map(|f| f.path().to_owned()),
                version: SourceVersion::Buffer(buffer.stamp()),
                bom: buffer.has_bom(),
                text,
            },
            scope,
            request,
            policy: policy.clone(),
        })
    }
    /// Run through the session's existing owned Job slot. Cancellation and the
    /// single configured deadline cover all native preparation work.
    pub fn run(self, cancel: &Cancellation) -> Result<Snapshot, String> {
        let cancel = cancel
            .bounded(Instant::now() + Duration::from_secs(self.policy.limits.job_seconds as u64));
        let budget = Budget {
            cancel: &cancel,
            deadline: Instant::now() + Duration::from_secs(self.policy.limits.job_seconds as u64),
        };
        budget.check()?;
        let nodes = document::nodes(&self.source.text, &cancel)?;
        let todo = render::workflow(&self.source.text, &nodes, &self.policy.todo, &cancel)?;
        let range = scope::select(&nodes, self.scope, &self.request.scope)?;
        let definitions = macros::Definitions::observe(&self.source.text, &nodes, &cancel)?;
        let mut metadata = Metadata::observe(&self.source.text, &nodes, &cancel)?;
        let options = self
            .request
            .options
            .clone()
            .unwrap_or(metadata.options.clone());
        let (text, sources) =
            include::expand(self.source, range, &self.reader, &self.policy, &cancel)?;
        let expanded = macros::expand(
            text,
            definitions,
            &mut metadata,
            &self.policy.limits,
            &cancel,
        )?;
        let text = expanded.text;
        let nodes = document::nodes(&text, &cancel)?;
        Ok(Snapshot {
            text,
            nodes,
            sources,
            diagnostics: expanded.diagnostics,
            request: self.request,
            metadata,
            options,
            export_bytes: self.policy.limits.export_bytes,
            job_seconds: self.policy.limits.job_seconds,
            todo,
        })
    }
}
