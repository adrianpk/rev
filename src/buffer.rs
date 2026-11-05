mod delete;
pub mod edit;
mod endings;
mod lifecycle;
mod mode;
mod org;
mod policy;
mod region;
pub mod reload;
pub use region::Visual;
mod rename;
mod save;
mod syntax;
mod visits;
pub(crate) use visits::Visit;
mod whitespace;

pub use lifecycle::Kill;
pub use save::{Confirmation, Save, SaveRequest};
pub use whitespace::Whitespace;

pub use mode::{Mode, ModeError};

use std::collections::{BTreeMap, VecDeque};

use crate::coordinates::Scalar;
use std::fmt;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use ropey::{Rope, RopeSlice, iter::Chunks};

static NEXT: AtomicU64 = AtomicU64::new(1);

/// A process-local buffer identity, never reused for another buffer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BufferId(u64);

impl BufferId {
    /// Zero is never allocated to a live buffer. Loaded durable evidence has
    /// no source binding until checked navigation publishes one.
    pub(crate) const UNBOUND: Self = Self(0);
    fn next() -> Result<Self, Error> {
        NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map(Self)
        .map_err(|_| Error::Exhausted)
    }
}

impl fmt::Display for BufferId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Freshness metadata for one buffer's published text, not a saved-state marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub buffer: BufferId,
    pub revision: u64,
}

/// Buffer-qualified logical history identity, independent of publication revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    buffer: BufferId,
    sequence: u64,
}

/// A half-open Unicode scalar range. Every use rechecks its stamp and bounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub stamp: Stamp,
    pub range: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Missing(BufferId),
    Foreign {
        expected: BufferId,
        actual: BufferId,
    },
    Stale {
        expected: u64,
        actual: u64,
    },
    Bounds {
        start: usize,
        end: usize,
        len: usize,
    },
    Boundary(usize),
    Line {
        index: usize,
        len: usize,
    },
    Exhausted,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(id) => write!(formatter, "buffer {id} is not live in this registry"),
            Self::Foreign { expected, actual } => {
                write!(
                    formatter,
                    "range belongs to buffer {actual}, expected {expected}"
                )
            }
            Self::Stale { expected, actual } => {
                write!(
                    formatter,
                    "range revision {actual} differs from current revision {expected}"
                )
            }
            Self::Bounds { start, end, len } => {
                write!(
                    formatter,
                    "text range {start}..{end} is invalid for length {len}"
                )
            }
            Self::Boundary(byte) => write!(formatter, "byte {byte} is not a UTF-8 scalar boundary"),
            Self::Line { index, len } => write!(formatter, "line {index} is outside {len} lines"),
            Self::Exhausted => formatter.write_str("buffer identities are exhausted"),
        }
    }
}

impl std::error::Error for Error {}

/// Owns the only rope for this identity. Explicit transient owners cannot
/// publish source files; all text access remains borrowed.
/// Canonical navigation identity; terminal output belongs to its emulator,
/// never to the inert text storage used by source-only APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Text,
    Terminal(crate::term::SessionId),
}

pub struct Buffer {
    resource: Resource,
    observer: Option<std::sync::Arc<dyn edit::Observer>>,
    pub(crate) bookmarks: crate::bookmark::Shared,
    pub(crate) locations: crate::location::Shared,
    pub(crate) bookmark_rows: std::sync::Mutex<crate::bookmark::markers::Cache>,
    id: BufferId,
    name: String,
    policy: crate::config::Editor,
    wrap_override: Option<bool>,
    pub(crate) coordinates: std::sync::Arc<std::sync::Mutex<crate::coordinates::Checkpoints>>,
    crlf: usize,
    revision: u64,
    text: Rope,
    read_only: bool,
    transient: bool,
    text_byte_limit: Option<usize>,
    source_language: Option<String>,
    mark: Option<Scalar>,
    mark_mode: region::Mode,
    history: VecDeque<edit::Transaction>,
    history_bytes: usize,
    history_policy: crate::config::History,
    redo: VecDeque<edit::Transaction>,
    state: State,
    saved: State,
    next_state: u64,
    typing: Option<Stamp>,
    syntax: crate::syntax::Syntax,
    org: Option<crate::org::Index>,
    mode_choice: Option<Mode>,
    org_active: bool,
    // Startup projection policy, shared by all views of this buffer.
    pub(crate) org_indent: bool,
    org_highlights: Option<crate::org::Highlights>,
    org_attempt: Option<Stamp>,
    org_outline: Option<std::sync::Arc<crate::org::Outline>>,
    org_enabled: bool,
    syntax_policy: crate::config::Syntax,
    grammar: crate::config::Grammar,
    language_identity: String,
    syntax_active: bool,
    external: Option<crate::syntax::assets::Handle>,
    external_seen: bool,
    external_applied: bool,
    syntax_generation: u64,
    bom: bool,
    default_newline: crate::config::Newline,
    file: Option<crate::file::Location>,
    warning: Option<String>,
    external_notice: Option<String>,
}

impl Buffer {
    fn new(text: &str, files: &crate::config::Files) -> Result<Self, Error> {
        let mut buffer = Self::from_rope(Rope::from_str(text))?;
        buffer.bom = files.bom;
        buffer.default_newline = files.newline;
        Ok(buffer)
    }

    fn from_rope(text: Rope) -> Result<Self, Error> {
        let id = BufferId::next()?;
        let state = State {
            buffer: id,
            sequence: 0,
        };
        Ok(Self {
            resource: Resource::Text,
            observer: None,
            bookmarks: crate::bookmark::Shared::default(),
            locations: crate::location::Shared::default(),
            bookmark_rows: std::sync::Mutex::default(),
            id,
            name: format!("*buffer-{id}*"),
            policy: crate::config::Editor::default(),
            wrap_override: None,
            coordinates: std::sync::Arc::default(),
            crlf: endings::count(text.slice(..)),
            revision: 1,
            text,
            read_only: false,
            transient: false,
            text_byte_limit: None,
            source_language: None,
            mark: None,
            mark_mode: region::Mode::Inactive,
            history: VecDeque::new(),
            history_bytes: 0,
            history_policy: crate::config::History::default(),
            redo: VecDeque::new(),
            state,
            saved: state,
            next_state: 1,
            typing: None,
            syntax: crate::syntax::Syntax::default(),
            org: None,
            mode_choice: None,
            org_active: false,
            org_indent: crate::config::Org::default().indent,
            org_highlights: None,
            org_attempt: None,
            org_outline: None,
            org_enabled: true,
            syntax_policy: crate::config::Syntax::default(),
            grammar: crate::config::Grammar::Plain,
            language_identity: "plain".into(),
            syntax_active: false,
            external: None,
            external_seen: false,
            external_applied: false,
            syntax_generation: 1,
            bom: false,
            default_newline: crate::config::Newline::Lf,
            file: None,
            warning: None,
            external_notice: None,
        })
    }

    pub(crate) fn from_loaded(loaded: crate::file::Loaded) -> Result<Self, Error> {
        let (text, bom) = loaded.into_parts();
        let mut buffer = Self::from_rope(text)?;
        buffer.bom = bom;
        Ok(buffer)
    }

    pub(crate) fn unchanged_prefix(&self, before: Stamp) -> Option<Scalar> {
        self.coordinates
            .lock()
            .expect("coordinate cache poisoned")
            .unchanged_prefix(before, self.stamp())
    }

    /// Transient text participates in checked edits, never source publication.
    pub fn is_transient(&self) -> bool {
        self.transient
    }

    pub fn resource(&self) -> Resource {
        self.resource
    }

    pub fn terminal(&self) -> Option<crate::term::SessionId> {
        match self.resource {
            Resource::Terminal(id) => Some(id),
            Resource::Text => None,
        }
    }

    pub fn has_bom(&self) -> bool {
        self.bom
    }

    /// Destination identity is separate from the unique display label.
    pub fn file(&self) -> Option<&crate::file::Location> {
        self.file.as_ref()
    }

    /// Insert the prevailing separator; LF wins mixed ties. With no separators,
    /// use the creation default (LF for loaded files). CRLF counts change only
    /// with atomic text publication, including joins at edit seams.
    pub fn newline(&self) -> &'static str {
        let lf = self.len_lines() - 1 - self.crlf;
        if lf == 0 && self.crlf == 0 && self.default_newline == crate::config::Newline::Crlf {
            return "\r\n";
        }
        if self.crlf > lf { "\r\n" } else { "\n" }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Compare logical text states rather than increasing publication revisions.
    pub fn is_modified(&self) -> bool {
        self.state != self.saved || self.warning.is_some()
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// A published but unresolved save remains pending even at an old clean state.
    pub fn save_warning(&self) -> Option<&str> {
        self.warning.as_deref()
    }
    /// Pending external-file condition; it never acknowledges save evidence.
    pub fn external_change_notice(&self) -> Option<&str> {
        self.external_notice.as_deref()
    }
    pub(crate) fn set_external_notice(&mut self, notice: Option<String>) {
        self.external_notice = notice;
    }

    /// A completed save may acknowledge only the exact text revision it wrote.
    /// This metadata operation preserves undo/redo and makes no filesystem claim.
    pub fn mark_saved(&mut self, stamp: Stamp) -> Result<(), Error> {
        self.current(stamp)?;
        self.saved = self.state;
        self.break_typing();
        Ok(())
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn stamp(&self) -> Stamp {
        Stamp {
            buffer: self.id,
            revision: self.revision,
        }
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    pub fn len_bytes(&self) -> usize {
        self.text.len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    /// Admits scalar ranges, including empty insertion ranges at either end.
    pub fn span(&self, range: Range<usize>) -> Result<Span, Error> {
        bounds(&range, self.len_chars())?;
        Ok(Span {
            stamp: self.stamp(),
            range,
        })
    }

    /// Admits a byte range only when both ends are exact UTF-8 scalar boundaries.
    pub fn byte_span(&self, range: Range<usize>) -> Result<Span, Error> {
        bounds(&range, self.len_bytes())?;
        let start = self.scalar(range.start)?;
        let end = self.scalar(range.end)?;
        self.span(start..end)
    }

    pub fn slice(&self, span: &Span) -> Result<RopeSlice<'_>, Error> {
        self.current(span.stamp)?;
        self.text
            .get_slice(span.range.clone())
            .ok_or_else(|| range_error(&span.range, self.len_chars()))
    }

    /// Iterates borrowed chunks clipped to the admitted range, without flattening.
    pub fn chunks(&self, span: &Span) -> Result<Chunks<'_>, Error> {
        Ok(self.slice(span)?.chunks())
    }

    /// Borrows a scalar range within one line, retaining its original separator.
    pub fn line(&self, index: usize, range: Range<usize>) -> Result<RopeSlice<'_>, Error> {
        let line = self.text.get_line(index).ok_or(Error::Line {
            index,
            len: self.len_lines(),
        })?;
        line.get_slice(range.clone())
            .ok_or_else(|| range_error(&range, line.len_chars()))
    }

    fn current(&self, stamp: Stamp) -> Result<(), Error> {
        if stamp.buffer != self.id {
            return Err(Error::Foreign {
                expected: self.id,
                actual: stamp.buffer,
            });
        }
        if stamp.revision != self.revision {
            return Err(Error::Stale {
                expected: self.revision,
                actual: stamp.revision,
            });
        }
        Ok(())
    }

    fn scalar(&self, byte: usize) -> Result<usize, Error> {
        let scalar = self
            .text
            .try_byte_to_char(byte)
            .map_err(|_| Error::Boundary(byte))?;
        if !self
            .text
            .try_char_to_byte(scalar)
            .is_ok_and(|end| end == byte)
        {
            return Err(Error::Boundary(byte));
        }
        Ok(scalar)
    }
}

/// Owns live buffers; file paths and view references do not own additional text.
pub struct Buffers {
    pub(crate) bookmarks: crate::bookmark::Shared,
    pub(crate) locations: crate::location::Shared,
    entries: BTreeMap<BufferId, Buffer>,
    initial: BufferId,
    files: BTreeMap<PathBuf, BufferId>,
    settings: crate::config::Snapshot,
}

impl Buffers {
    pub(crate) fn snapshot(&self) -> &crate::config::Snapshot {
        &self.settings
    }

    /// The global palette is shared by all buffers and does not vary by path.
    pub(crate) fn palette(&self) -> &crate::config::Palette {
        &self.settings.settings().palette
    }

    /// Creates a registry with one initially empty buffer.
    pub fn new() -> Result<Self, Error> {
        Self::with_settings(crate::config::Snapshot::default())
    }

    /// Use one supplied, admitted snapshot for newly installed buffer policies.
    /// Construction performs no configuration discovery or file reads.
    pub fn with_settings(settings: crate::config::Snapshot) -> Result<Self, Error> {
        let mut buffer = Buffer::new("", &settings.settings().files)?;
        buffer.name = "*scratch*".into();
        buffer.apply_settings(&settings);
        {
            let mut bookmarks = buffer.bookmarks.lock().expect("bookmark owner");
            bookmarks.max_entries = settings.settings().bookmarks.max_entries;
            bookmarks.context_chars = settings.settings().bookmarks.context_chars;
        }
        let initial = buffer.id();
        Ok(Self {
            bookmarks: buffer.bookmarks.clone(),
            locations: buffer.locations.clone(),
            entries: BTreeMap::from([(initial, buffer)]),
            initial,
            files: BTreeMap::new(),
            settings,
        })
    }

    /// Quit admission examines every live buffer's current dirty metadata.
    pub fn is_modified(&self) -> bool {
        self.entries.values().any(Buffer::is_modified)
    }

    pub fn initial(&self) -> BufferId {
        self.initial
    }

    /// Creates another canonical buffer from validated Rust UTF-8 text.
    pub fn create(&mut self, text: &str) -> Result<BufferId, Error> {
        let buffer = Buffer::new(text, &self.settings.settings().files)?;
        Ok(self.insert(buffer))
    }

    pub(crate) fn capture_draft(&mut self, text: &str) -> Result<BufferId, Error> {
        let mut buffer = Buffer::new(text, &self.settings.settings().files)?;
        buffer.name = "*Org Capture*".into();
        buffer.transient = true;
        buffer.bom = false;
        buffer.mode_choice = Some(Mode::Text);
        Ok(self.insert(buffer))
    }

    pub(crate) fn source_draft(&mut self, text: &str, language: &str) -> Result<BufferId, Error> {
        let mut buffer = Buffer::new(text, &self.settings.settings().files)?;
        buffer.name = "*Org Source*".into();
        buffer.transient = true;
        buffer.source_language = Some(language.to_owned());
        buffer.default_newline = crate::config::Newline::Lf;
        Ok(self.insert(buffer))
    }

    /// A Git message is native text with a finite session-only owner. Its
    /// smaller history and transaction limits fit the composer's admission.
    pub(crate) fn git_message_draft(
        &mut self,
        text: &str,
        annotation: bool,
    ) -> Result<BufferId, Error> {
        self.git_draft(
            text,
            if annotation {
                "*Git Tag Annotation: C-Enter reviews; C-g retains draft*"
            } else {
                "*Git Commit: C-Enter submits; C-g retains draft*"
            },
        )
    }
    pub(crate) fn git_argument_draft(&mut self, text: &str) -> Result<BufferId, Error> {
        self.git_draft(text, "*Git Arguments: C-Enter reviews; C-g cancels*")
    }
    fn git_draft(&mut self, text: &str, name: &str) -> Result<BufferId, Error> {
        let mut buffer = Buffer::new(text, &self.settings.settings().files)?;
        buffer.name = name.into();
        buffer.transient = true;
        buffer.text_byte_limit = Some(crate::git::MAX_COMMIT_BYTES);
        buffer.bom = false;
        buffer.default_newline = crate::config::Newline::Lf;
        buffer.mode_choice = Some(Mode::Text);
        Ok(self.insert(buffer))
    }

    /// Installs only fully validated ingress as a fresh clean canonical owner.
    pub fn load(&mut self, loaded: crate::file::Loaded) -> Result<BufferId, Error> {
        let buffer = Buffer::from_loaded(loaded)?;
        Ok(self.insert(buffer))
    }

    pub(crate) fn insert(&mut self, mut buffer: Buffer) -> BufferId {
        buffer.bookmarks = self.bookmarks.clone();
        buffer.locations = self.locations.clone();
        buffer.apply_settings(&self.settings);
        buffer.name = self.label(&buffer.name);
        let id = buffer.id();
        self.entries.insert(id, buffer);
        id
    }

    /// Mutable access exposes checked operations, never the underlying rope.
    pub fn get_mut(&mut self, id: BufferId) -> Result<&mut Buffer, Error> {
        self.entries.get_mut(&id).ok_or(Error::Missing(id))
    }

    pub fn get(&self, id: BufferId) -> Result<&Buffer, Error> {
        self.entries.get(&id).ok_or(Error::Missing(id))
    }
}

fn bounds(range: &Range<usize>, len: usize) -> Result<(), Error> {
    if range.start > range.end || range.end > len {
        return Err(range_error(range, len));
    }
    Ok(())
}

fn range_error(range: &Range<usize>, len: usize) -> Error {
    Error::Bounds {
        start: range.start,
        end: range.end,
        len,
    }
}

#[cfg(test)]
mod capture_tests;

impl Buffers {
    /// Canonical collection shared with every live buffer's publication owner.
    pub fn bookmarks(&self) -> crate::bookmark::Collection<'_> {
        crate::bookmark::Collection(self.bookmarks.lock().expect("bookmark owner poisoned"))
    }
    pub(crate) fn bookmarks_mut(&self) -> std::sync::MutexGuard<'_, crate::bookmark::Bookmarks> {
        self.bookmarks.lock().expect("bookmark owner poisoned")
    }
}
