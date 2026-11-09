use super::{Completeness, Group, Head, Observation, OperationId, PathChange, budget::Lease};
use std::{ops::Range, sync::Arc};

mod patch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Baseline {
    Empty,
    Head(super::ObjectId),
    Index,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffKind {
    Text,
    Binary,
    Rename,
    TypeChange,
    Submodule,
    Unmerged,
    Metadata,
    Malformed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub count: usize,
}

/// Identifies one unit in an immutable comparison, never a presentation row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HunkId(OperationId, usize);

#[derive(Debug)]
pub struct Hunk {
    pub id: HunkId,
    pub old: LineRange,
    pub new: LineRange,
    pub patch: Range<usize>,
}

/// Patch tokens are classified before display escaping or 2048-byte chunking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Metadata,
    Header,
    Hunk,
    Context,
    Added,
    Removed,
}

/// Worker-indexed, bounded display chunks. Bytes remain in the parent evidence;
/// a line label cannot reconstruct a pathname or become a changing target.
#[derive(Debug)]
pub struct DiffLine {
    pub bytes: Range<usize>,
    pub hunk: Option<HunkId>,
    pub kind: DiffLineKind,
}

/// Exact read-only comparison tied to its originating Status observation.
/// Complete patch evidence is retained for later revalidation; accepting a
/// parsed hunk alone does not establish that this comparison is still current.
#[derive(Debug)]
pub struct Diff {
    pub id: OperationId,
    pub observation: Arc<Observation>,
    pub group: Group,
    pub change: PathChange,
    pub baseline: Baseline,
    pub kind: DiffKind,
    pub binary: bool,
    pub completeness: Completeness,
    pub verified: bool,
    pub evidence: Vec<u8>,
    pub lines: Vec<DiffLine>,
    pub hunks: Vec<Hunk>,
    pub(super) admission: Lease,
}

impl Diff {
    pub fn supports_hunks(&self) -> bool {
        self.verified
            && matches!(self.group, Group::Staged | Group::Unstaged)
            && self.kind == DiffKind::Text
            && self.completeness == Completeness::Complete
            && !self.hunks.is_empty()
    }
    pub fn hunk(&self, id: HunkId) -> Option<&Hunk> {
        // The private ordinal belongs to this immutable comparison. Checking
        // the complete ID prevents another comparison from aliasing its slot.
        self.hunks.get(id.1).filter(|hunk| hunk.id == id)
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.evidence.capacity()
            + self.lines.capacity() * std::mem::size_of::<DiffLine>()
            + self.hunks.capacity() * std::mem::size_of::<Hunk>()
            + self.change.bytes()
            + match &self.kind {
                DiffKind::Malformed(reason) => reason.capacity(),
                _ => 0,
            }
    }
}

pub(super) struct Target {
    pub observation: Arc<Observation>,
    pub group: Group,
    pub change: PathChange,
}

pub(super) struct Prepared {
    pub target: Target,
    pub bytes: Vec<u8>,
    pub verified: bool,
    pub completeness: Completeness,
}

pub(super) fn prepare(
    id: OperationId,
    prepared: Prepared,
    mut admission: Lease,
    cancel: &crate::process::Cancellation,
) -> Result<Diff, String> {
    let Prepared {
        target,
        bytes,
        verified,
        mut completeness,
    } = prepared;
    let baseline = target.baseline();
    let binary = bytes
        .split(|b| *b == b'\n')
        .any(|line| line.starts_with(b"Binary files ") || line == b"GIT binary patch");
    let mut kind = classify(&target, binary);
    let mut lines = Vec::new();
    let mut hunks = Vec::new();
    if let Some((admitted_lines, admitted_hunks)) = admit_lines(&bytes, &mut admission) {
        lines = admitted_lines;
        hunks = admitted_hunks;
        let parsed = if completeness == Completeness::Complete && kind != DiffKind::Unmerged {
            patch::parse(id, &bytes, &mut lines, &mut hunks, cancel)
        } else {
            raw_lines(&bytes, &mut lines);
            Ok(())
        };
        if let Err(error) = parsed {
            if cancel.is_cancelled() {
                return Err(error);
            }
            kind = DiffKind::Malformed(error);
        }
    } else {
        completeness = Completeness::Truncated {
            reason: "Git session detail admission",
        };
    }
    if kind == DiffKind::Text && hunks.is_empty() {
        kind = DiffKind::Metadata;
    }
    let verified = verified && !matches!(kind, DiffKind::Malformed(_));
    let mut diff = Diff {
        id,
        observation: target.observation,
        group: target.group,
        change: target.change,
        baseline,
        kind,
        binary,
        completeness,
        verified,
        evidence: bytes,
        lines,
        hunks,
        admission,
    };
    diff.admission.retain(diff.bytes());
    Ok(diff)
}

fn classify(target: &Target, binary: bool) -> DiffKind {
    let change = &target.change;
    if change.unmerged {
        return DiffKind::Unmerged;
    }
    let modes = match target.group {
        Group::Staged => change.modes.get(..2),
        Group::Unstaged => change.modes.get(1..3),
        _ => None,
    };
    if modes.is_some_and(|pair| {
        pair.iter().all(|mode| *mode != 0) && pair[0] & 0o170000 != pair[1] & 0o170000
    }) {
        return DiffKind::TypeChange;
    }
    if modes.is_some_and(|pair| pair.contains(&0o160000)) {
        return DiffKind::Submodule;
    }
    if change.renamed_in(target.group) {
        return DiffKind::Rename;
    }
    if binary {
        DiffKind::Binary
    } else {
        DiffKind::Text
    }
}

fn ranges(header: &[u8]) -> Result<(LineRange, LineRange), String> {
    let mut parts = header.split(|byte| *byte == b' ');
    if parts.next() != Some(b"@@".as_slice()) {
        return Err("Missing unified header".into());
    }
    let old = line_range(parts.next(), b'-')?;
    let new = line_range(parts.next(), b'+')?;
    if parts.next() != Some(b"@@".as_slice()) {
        return Err("Incomplete unified header".into());
    }
    Ok((old, new))
}
fn line_range(bytes: Option<&[u8]>, prefix: u8) -> Result<LineRange, String> {
    let bytes = bytes
        .and_then(|value| value.strip_prefix(&[prefix]))
        .ok_or("Missing unified range")?;
    let mut parts = bytes.split(|b| *b == b',');
    let start = range_number(parts.next().ok_or("Missing range start")?)?;
    let count = parts.next().map_or(Ok(1), range_number)?;
    if parts.next().is_some() || start.checked_add(count).is_none() || (start == 0 && count != 0) {
        return Err("Unsupported unified range".into());
    }
    Ok(LineRange { start, count })
}

fn range_number(bytes: &[u8]) -> Result<usize, String> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err("Invalid unified range number".into());
    }
    std::str::from_utf8(bytes)
        .map_err(|e| e.to_string())?
        .parse::<usize>()
        .map_err(|e| e.to_string())
}

impl Target {
    fn baseline(&self) -> Baseline {
        match self.group {
            Group::Untracked => Baseline::Empty,
            Group::Staged => match self.observation.status.as_ref().map(|s| &s.head) {
                Some(Head::Branch { object, .. } | Head::Detached { object }) => {
                    Baseline::Head(*object)
                }
                _ => Baseline::Empty,
            },
            _ => Baseline::Index,
        }
    }
}

/// Reserve all indexed chunks and hunks before allocating their vectors.
fn admit_lines(bytes: &[u8], admission: &mut Lease) -> Option<(Vec<DiffLine>, Vec<Hunk>)> {
    let chunks = bytes
        .split_inclusive(|b| *b == b'\n')
        .map(|line| line.len().div_ceil(2048))
        .sum::<usize>();
    let count = bytes
        .split(|b| *b == b'\n')
        .filter(|line| line.starts_with(b"@@ "))
        .count();
    let reserved = chunks
        .checked_mul(2 * std::mem::size_of::<DiffLine>())
        .and_then(|lines| {
            count
                .checked_mul(2 * std::mem::size_of::<Hunk>())
                .and_then(|hunks| lines.checked_add(hunks))
        })
        .and_then(|bytes| bytes.checked_add(32768));
    let reserved = reserved?;
    admission.grow(reserved).ok()?;
    Some((Vec::with_capacity(chunks), Vec::with_capacity(count)))
}

fn raw_lines(bytes: &[u8], lines: &mut Vec<DiffLine>) {
    let mut offset = 0;
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        for start in (0..line.len()).step_by(2048) {
            lines.push(DiffLine {
                bytes: offset + start..offset + (start + 2048).min(line.len()),
                hunk: None,
                kind: DiffLineKind::Context,
            });
        }
        offset += line.len();
    }
}
