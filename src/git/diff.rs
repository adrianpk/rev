use super::{Completeness, Group, Head, Observation, OperationId, PathChange, budget::Lease};
use std::{ops::Range, sync::Arc};

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

/// Worker-indexed, bounded display chunks. Bytes remain in the parent evidence;
/// a line label cannot reconstruct a pathname or become a changing target.
#[derive(Debug)]
pub struct DiffLine {
    pub bytes: Range<usize>,
    pub hunk: Option<HunkId>,
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
        self.hunks.iter().find(|hunk| hunk.id == id)
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
    let baseline = match target.group {
        Group::Untracked => Baseline::Empty,
        Group::Staged => match target.observation.status.as_ref().map(|s| &s.head) {
            Some(Head::Branch { object, .. } | Head::Detached { object }) => {
                Baseline::Head(*object)
            }
            _ => Baseline::Empty,
        },
        _ => Baseline::Index,
    };
    let binary = bytes
        .split(|b| *b == b'\n')
        .any(|line| line.starts_with(b"Binary files ") || line == b"GIT binary patch");
    let mut kind = classify(&target, binary);
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
    let mut lines = Vec::new();
    let mut hunks = Vec::new();
    if reserved.is_some_and(|bytes| admission.grow(bytes).is_ok()) {
        lines = Vec::with_capacity(chunks);
        hunks = Vec::with_capacity(count);
        let parsed = if completeness == Completeness::Complete && kind != DiffKind::Unmerged {
            parse(id, &bytes, &mut lines, &mut hunks, cancel)
        } else {
            let mut offset = 0;
            for line in bytes.split_inclusive(|b| *b == b'\n') {
                for start in (0..line.len()).step_by(2048) {
                    lines.push(DiffLine {
                        bytes: offset + start..offset + (start + 2048).min(line.len()),
                        hunk: None,
                    });
                }
                offset += line.len();
            }
            Ok(())
        };
        if let Err(error) = parsed {
            if cancel.is_cancelled() {
                return Err(error);
            }
            kind = DiffKind::Malformed(error);
            hunks.clear();
            for line in &mut lines {
                line.hunk = None;
            }
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

fn parse(
    operation: OperationId,
    bytes: &[u8],
    lines: &mut Vec<DiffLine>,
    hunks: &mut Vec<Hunk>,
    cancel: &crate::process::Cancellation,
) -> Result<(), String> {
    let mut offset = 0;
    let mut remaining = (0usize, 0usize);
    let mut active = None;
    let mut failure = None;
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        if cancel.is_cancelled() {
            return Err("Git diff parsing cancelled or timed out".into());
        }
        let content = line.strip_suffix(b"\n").unwrap_or(line);
        if content.starts_with(b"@@ ") {
            if remaining != (0, 0) {
                failure = Some("Git hunk body does not match its declared ranges");
            }
            match ranges(content) {
                Ok((old, new)) => {
                    let id = HunkId(operation, hunks.len());
                    remaining = (old.count, new.count);
                    hunks.push(Hunk {
                        id,
                        old,
                        new,
                        patch: offset..offset + line.len(),
                    });
                    active = Some(id);
                }
                Err(_) => {
                    failure = Some("Malformed Git unified hunk ranges");
                    active = None;
                }
            }
        } else if active.is_some()
            && (remaining != (0, 0) || content == b"\\ No newline at end of file")
        {
            let used = match content.first() {
                Some(b' ') => (1, 1),
                Some(b'-') => (1, 0),
                Some(b'+') => (0, 1),
                Some(b'\\') if content == b"\\ No newline at end of file" => (0, 0),
                _ => {
                    failure = Some("Malformed Git hunk body");
                    (0, 0)
                }
            };
            if used.0 > remaining.0 || used.1 > remaining.1 {
                failure = Some("Git hunk exceeds its declared ranges");
            }
            remaining = (
                remaining.0.saturating_sub(used.0),
                remaining.1.saturating_sub(used.1),
            );
            if let Some(hunk) = hunks.last_mut() {
                hunk.patch.end = offset + line.len();
            }
        } else {
            active = None;
            // Every unified comparison comes from one native Git request;
            // unexpected extra payload must not acquire semantic text targets.
            if !content.is_empty()
                && ![
                    b"diff ".as_slice(),
                    b"index ",
                    b"--- ",
                    b"+++ ",
                    b"old mode ",
                    b"new mode ",
                    b"new file mode ",
                    b"deleted file mode ",
                    b"similarity index ",
                    b"dissimilarity index ",
                    b"rename from ",
                    b"rename to ",
                    b"copy from ",
                    b"copy to ",
                    b"Binary files ",
                    b"Submodule ",
                    b"GIT binary patch",
                    b"@@@ ",
                ]
                .iter()
                .any(|prefix| content.starts_with(prefix))
            {
                failure = Some("Unsupported Git diff payload");
            }
        }
        for start in (0..line.len()).step_by(2048) {
            lines.push(DiffLine {
                bytes: offset + start..offset + (start + 2048).min(line.len()),
                hunk: active,
            });
        }
        offset += line.len();
    }
    if remaining != (0, 0) {
        failure = Some("Incomplete Git unified hunk body");
    }
    failure.map_or(Ok(()), |message| Err(message.into()))
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
    let number = |bytes: &[u8]| -> Result<usize, String> {
        if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
            return Err("Invalid unified range number".into());
        }
        std::str::from_utf8(bytes)
            .map_err(|e| e.to_string())?
            .parse::<usize>()
            .map_err(|e| e.to_string())
    };
    let start = number(parts.next().ok_or("Missing range start")?)?;
    let count = parts.next().map_or(Ok(1), number)?;
    if parts.next().is_some() || start.checked_add(count).is_none() || (start == 0 && count != 0) {
        return Err("Unsupported unified range".into());
    }
    Ok(LineRange { start, count })
}
