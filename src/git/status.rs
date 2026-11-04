use super::{Identity, ObjectFormat, ObjectId};
use crate::config::Git;
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Head {
    Unborn { branch: Vec<u8> },
    Branch { name: Vec<u8>, object: ObjectId },
    Detached { object: ObjectId },
}

/// A path in a layer is a semantic target, independent of a rendered row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Unmerged,
    Untracked,
    Unstaged,
    Staged,
}

impl Group {
    pub const ALL: [Self; 4] = [
        Self::Unmerged,
        Self::Untracked,
        Self::Unstaged,
        Self::Staged,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Unmerged => "Unmerged",
            Self::Untracked => "Untracked",
            Self::Unstaged => "Unstaged",
            Self::Staged => "Staged",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathChange {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub index: u8,
    pub worktree: u8,
    pub unmerged: bool,
    pub submodule: [u8; 4],
    pub modes: Vec<u32>,
    pub objects: Vec<ObjectId>,
}

impl PathChange {
    pub fn in_group(&self, group: Group) -> bool {
        match group {
            Group::Unmerged => self.unmerged,
            Group::Untracked => self.index == b'?',
            Group::Unstaged => !self.unmerged && self.index != b'?' && self.worktree != b'.',
            Group::Staged => !self.unmerged && self.index != b'?' && self.index != b'.',
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Completeness {
    Complete,
    Truncated { reason: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub head: Head,
    pub upstream: Option<Vec<u8>>,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub paths: Vec<PathChange>,
    pub completeness: Completeness,
}

impl Status {
    pub fn group(&self, group: Group) -> impl Iterator<Item = &PathChange> {
        self.paths.iter().filter(move |path| path.in_group(group))
    }

    pub(super) fn bytes(&self) -> usize {
        let head = match &self.head {
            Head::Unborn { branch } => branch.capacity(),
            Head::Branch { name, .. } => name.capacity(),
            Head::Detached { .. } => 0,
        };
        head + self.upstream.as_ref().map_or(0, Vec::capacity)
            + self.paths.capacity() * std::mem::size_of::<PathChange>()
            + self
                .paths
                .iter()
                .map(|p| {
                    p.path.capacity()
                        + p.old_path.as_ref().map_or(0, PathBuf::capacity)
                        + p.modes.capacity() * std::mem::size_of::<u32>()
                        + p.objects.capacity() * std::mem::size_of::<ObjectId>()
                })
                .sum::<usize>()
    }
}

pub(super) fn parse(
    bytes: &[u8],
    identity: &Identity,
    policy: &Git,
    cancel: &crate::process::Cancellation,
) -> Result<Status, String> {
    if bytes.last() != Some(&0) {
        return Err("Git status lacks its final NUL record boundary".into());
    }
    let mut records = bytes[..bytes.len() - 1].split(|b| *b == 0);
    let mut object = None;
    let mut branch = None;
    let mut upstream = None;
    let mut counts = None;
    let mut paths = Vec::new();
    let mut path_bytes = 0usize;
    let mut completeness = Completeness::Complete;
    for_record(&mut records, |record, records| {
        if cancel.is_cancelled() {
            return Err("Git status parsing cancelled or timed out".into());
        }
        if let Some(value) = record.strip_prefix(b"# branch.oid ") {
            if object
                .replace(if value == b"(initial)" {
                    None
                } else {
                    Some(ObjectId::parse(value, identity.format)?)
                })
                .is_some()
            {
                return Err("Duplicate Git HEAD object".into());
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.head ") {
            if branch.replace(metadata(value)?.to_vec()).is_some() {
                return Err("Duplicate Git HEAD name".into());
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.upstream ") {
            if upstream.replace(metadata(value)?.to_vec()).is_some() {
                return Err("Duplicate Git upstream".into());
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.ab ") {
            let split = value
                .iter()
                .position(|b| *b == b' ')
                .ok_or("Malformed Git tracking counts")?;
            let (ahead, behind) = (&value[..split], &value[split + 1..]);
            let number = |value: Option<&[u8]>| -> Result<usize, String> {
                std::str::from_utf8(value.ok_or("Malformed Git tracking count")?)
                    .map_err(|e| e.to_string())?
                    .parse()
                    .map_err(|_| "Malformed Git tracking count".into())
            };
            if counts
                .replace((
                    number(ahead.strip_prefix(b"+"))?,
                    number(behind.strip_prefix(b"-"))?,
                ))
                .is_some()
            {
                return Err("Duplicate Git tracking counts".into());
            }
        } else if record.starts_with(b"# ") {
            return Err("Unsupported Git status header".into());
        } else {
            let path = change(record, records, identity.format)?;
            let size = path.path.as_os_str().as_encoded_bytes().len()
                + path
                    .old_path
                    .as_ref()
                    .map_or(0, |p| p.as_os_str().as_encoded_bytes().len());
            if paths.len() >= policy.max_status_paths {
                completeness = Completeness::Truncated {
                    reason: "max_status_paths",
                };
            } else if path_bytes + size > policy.max_path_bytes {
                completeness = Completeness::Truncated {
                    reason: "max_path_bytes",
                };
            } else if completeness == Completeness::Complete {
                path_bytes += size;
                paths.push(path);
            }
        }
        Ok(())
    })?;
    let object = object.ok_or("Git status lacks HEAD object")?;
    let branch = branch.ok_or("Git status lacks HEAD name")?;
    let head = match (object, branch.as_slice()) {
        (None, b"(detached)") => return Err("Detached Git HEAD lacks an object".into()),
        (None, _) => Head::Unborn { branch },
        (Some(object), b"(detached)") => Head::Detached { object },
        (Some(object), _) => Head::Branch {
            name: branch,
            object,
        },
    };
    if counts.is_some() && upstream.is_none() {
        return Err("Git tracking counts lack an upstream".into());
    }
    Ok(Status {
        head,
        upstream,
        ahead: counts.map(|n| n.0),
        behind: counts.map(|n| n.1),
        paths,
        completeness,
    })
}

// The iterator also supplies the second NUL record of a rename/copy, whose
// original pathname can contain any byte except NUL.
fn for_record<'a>(
    records: &mut impl Iterator<Item = &'a [u8]>,
    mut visit: impl FnMut(&'a [u8], &mut dyn Iterator<Item = &'a [u8]>) -> Result<(), String>,
) -> Result<(), String> {
    while let Some(record) = records.next() {
        visit(record, records)?;
    }
    Ok(())
}

fn change(
    record: &[u8],
    records: &mut dyn Iterator<Item = &[u8]>,
    format: ObjectFormat,
) -> Result<PathChange, String> {
    let (kind, fields) = match record.first() {
        Some(b'?') => (b'?', 2),
        Some(b'1') => (b'1', 9),
        Some(b'2') => (b'2', 10),
        Some(b'u') => (b'u', 11),
        _ => return Err("Unsupported Git path record".into()),
    };
    let parts: Vec<_> = record.splitn(fields, |b| *b == b' ').collect();
    if parts.len() != fields {
        return Err("Incomplete Git path record".into());
    }
    let parsed_path = path(parts[fields - 1])?;
    let mut change = PathChange {
        path: parsed_path,
        old_path: None,
        index: b'?',
        worktree: b'?',
        unmerged: false,
        submodule: *b"N...",
        modes: Vec::new(),
        objects: Vec::new(),
    };
    if kind == b'?' {
        return Ok(change);
    }
    let xy = parts[1];
    if xy.len() != 2 || !xy.iter().all(|b| b".MTADRCU".contains(b)) {
        return Err("Malformed Git path status".into());
    }
    change.index = xy[0];
    change.worktree = xy[1];
    change.unmerged = kind == b'u';
    let sub = parts[2];
    if sub != b"N..."
        && !(sub.len() == 4
            && sub[0] == b'S'
            && b".C".contains(&sub[1])
            && b".M".contains(&sub[2])
            && b".U".contains(&sub[3]))
    {
        return Err("Malformed Git submodule status".into());
    }
    change.submodule.copy_from_slice(sub);
    let mode_end = if kind == b'u' { 7 } else { 6 };
    for mode in &parts[3..mode_end] {
        if mode.len() != 6 || !mode.iter().all(|b| (b'0'..=b'7').contains(b)) {
            return Err("Malformed Git mode".into());
        }
        change.modes.push(
            u32::from_str_radix(std::str::from_utf8(mode).expect("ASCII mode"), 8)
                .map_err(|e| e.to_string())?,
        );
    }
    let object_end = if kind == b'u' { 10 } else { 8 };
    for object in &parts[mode_end..object_end] {
        change.objects.push(ObjectId::parse(object, format)?);
    }
    if kind == b'2' {
        let score = parts[8];
        if score.len() < 2
            || !b"RC".contains(&score[0])
            || !score[1..].iter().all(u8::is_ascii_digit)
            || std::str::from_utf8(&score[1..])
                .expect("ASCII score")
                .parse::<u32>()
                .map_or(true, |n| n > 100)
        {
            return Err("Malformed Git rename/copy score".into());
        }
        change.old_path = Some(path(
            records.next().ok_or("Git rename lacks its original path")?,
        )?);
    }
    Ok(change)
}

fn metadata(bytes: &[u8]) -> Result<&[u8], String> {
    if bytes.is_empty() || bytes.len() > 4096 {
        return Err("Git metadata exceeds its 4096-byte bound".into());
    }
    Ok(bytes)
}

fn path(bytes: &[u8]) -> Result<PathBuf, String> {
    metadata(bytes)?;
    let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err("Git path is not an exact repository-relative path".into());
    }
    Ok(path)
}
