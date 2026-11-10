mod blame;
mod quoted;
use super::*;
use crate::git::{Attribution, Blame, Reflog, ReflogEntry, RevisionFile};
pub(in crate::git::work) use quoted::unquote_path;

impl Runner {
    pub(super) fn read_reflog(
        &mut self,
        observation: Arc<Observation>,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let (bytes, truncated) = if matches!(
            observation.status.as_ref().map(|s| &s.head),
            Some(Head::Unborn { .. })
        ) {
            (Vec::new(), false)
        } else {
            self.command_os(
                &[
                    "--no-replace-objects".into(),
                    "reflog".into(),
                    "show".into(),
                    "--date=iso-strict".into(),
                    "--format=%H%x00%gD%x00%gs%x00%gN%x00%gE".into(),
                    "-z".into(),
                    // Each record needs a full hash and five NUL separators.
                    // Read one beyond the byte-derived ceiling so a capped
                    // traversal cannot masquerade as a complete reflog.
                    format!(
                        "--max-count={}",
                        self.policy.max_detail_bytes
                            / match observation.identity.format {
                                ObjectFormat::Sha1 => 45,
                                ObjectFormat::Sha256 => 69,
                            }
                            + 1
                    )
                    .into(),
                    "HEAD".into(),
                    "--".into(),
                ],
                Phase::Refreshing,
                false,
                Some(self.policy.max_detail_bytes),
            )?
        };
        let bytes_charge = bytes.len() * 3
            + (bytes.iter().filter(|b| **b == 0).count() / 5 + 1)
                * std::mem::size_of::<ReflogEntry>()
                * 2
            + 4096;
        admission.grow(bytes_charge)?;
        let entries = reflog_entries(&bytes, truncated, observation.identity.format)?;
        let complete = !truncated;
        let retained = std::mem::size_of::<Reflog>()
            + entries.capacity() * std::mem::size_of::<ReflogEntry>()
            + entries
                .iter()
                .map(|e| {
                    e.selector.capacity()
                        + e.subject.capacity()
                        + e.author.capacity()
                        + e.email.capacity()
                })
                .sum::<usize>();
        admission.grow(retained)?;
        Ok(ResultData::Reflog(Reflog {
            id: self.progress.lock().expect("Git progress owner").operation,
            observation,
            entries,
            completeness: if complete {
                Completeness::Complete
            } else {
                Completeness::Truncated {
                    reason: "Reflog detail bound",
                }
            },
            _admission: admission.split(retained),
        }))
    }

    pub(super) fn read_blame(
        &mut self,
        observation: Arc<Observation>,
        path: PathBuf,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let root = match observation.status.as_ref().map(|s| &s.head) {
            Some(Head::Branch { object, .. } | Head::Detached { object }) => *object,
            _ => return Err("Blame requires an existing exact HEAD revision".into()),
        };
        let (bytes, truncated) = self.command_os(
            &[
                "--no-replace-objects".into(),
                "--literal-pathspecs".into(),
                "-c".into(),
                "core.quotePath=false".into(),
                "blame".into(),
                "--line-porcelain".into(),
                "--no-textconv".into(),
                "--root".into(),
                "--no-progress".into(),
                root.hex().into(),
                "--".into(),
                path.as_os_str().to_owned(),
            ],
            Phase::Refreshing,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        let record_count = bytes
            .split(|b| *b == b'\n')
            .filter(|line| line.starts_with(b"\t"))
            .count();
        admission.grow(
            bytes.len() * 4 + (record_count + 1) * std::mem::size_of::<Attribution>() * 2 + 8192,
        )?;
        let parsed = blame::Attributions::read(
            &bytes,
            observation.identity.format,
            &self.policy,
            &self.cancel.bounded(self.deadline),
            truncated,
        )?;
        let lines = parsed.lines;
        let complete = parsed.complete;
        let retained = std::mem::size_of::<Blame>()
            + path.capacity()
            + lines.capacity() * std::mem::size_of::<Attribution>()
            + lines
                .iter()
                .map(|l| {
                    l.path.capacity()
                        + l.author.capacity()
                        + l.email.capacity()
                        + l.time.capacity()
                        + l.subject.capacity()
                        + l.text.capacity()
                })
                .sum::<usize>();
        admission.grow(retained)?;
        Ok(ResultData::Blame(Blame {
            id: self.progress.lock().expect("Git progress owner").operation,
            observation,
            root,
            path,
            lines,
            completeness: if complete {
                Completeness::Complete
            } else {
                Completeness::Truncated {
                    reason: "Blame path or detail bound",
                }
            },
            _admission: admission.split(retained),
        }))
    }

    pub(super) fn read_revision_file(
        &mut self,
        request: crate::git::history::FileRequest,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let crate::git::history::FileRequest {
            observation,
            object,
            path,
            line,
            current_path,
        } = request;
        validate_path(path.as_os_str().as_encoded_bytes())?;
        let mut name = object.hex().into_bytes();
        name.push(b':');
        name.extend_from_slice(path.as_os_str().as_encoded_bytes());
        let (bytes, truncated) = self.command_os(
            &[
                "--no-replace-objects".into(),
                "cat-file".into(),
                "blob".into(),
                OsString::from_vec(name),
            ],
            Phase::Refreshing,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        let chunks = bytes
            .split_inclusive(|b| *b == b'\n')
            .map(|l| {
                l.strip_suffix(b"\n")
                    .unwrap_or(l)
                    .len()
                    .div_ceil(2048)
                    .max(1)
            })
            .sum::<usize>();
        let retained = std::mem::size_of::<RevisionFile>()
            + path.capacity()
            + current_path.as_ref().map_or(0, PathBuf::capacity)
            + bytes.capacity()
            + chunks * std::mem::size_of::<std::ops::Range<usize>>();
        admission.grow(retained)?;
        let mut lines = Vec::with_capacity(chunks);
        let mut offset = 0;
        for part in bytes.split_inclusive(|b| *b == b'\n') {
            let text = part.strip_suffix(b"\n").unwrap_or(part);
            if text.is_empty() {
                lines.push(offset..offset);
            } else {
                for start in (0..text.len()).step_by(2048) {
                    lines.push(offset + start..offset + (start + 2048).min(text.len()));
                }
            }
            offset += part.len();
        }
        let binary = bytes.contains(&0);
        Ok(ResultData::File(RevisionFile {
            id: self.progress.lock().expect("Git progress owner").operation,
            observation,
            object,
            path,
            current_path,
            bytes,
            lines,
            completeness: if truncated {
                Completeness::Truncated {
                    reason: "Historical file detail bound",
                }
            } else {
                Completeness::Complete
            },
            binary,
            original_line: line,
            _admission: admission.split(retained),
        }))
    }
}

fn nul_fields(bytes: &[u8]) -> impl Iterator<Item = &[u8]> {
    bytes
        .split_inclusive(|b| *b == 0)
        .filter_map(|p| p.strip_suffix(b"\0"))
}
fn metadata_field(value: &[u8]) -> Result<(), String> {
    if value.len() > 4096 || value.contains(&0) {
        Err("Git inspection metadata exceeds 4096 bytes".into())
    } else {
        Ok(())
    }
}
fn number(value: &[u8]) -> Result<usize, String> {
    let n = std::str::from_utf8(value)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0);
    n.ok_or("Git blame line/count is invalid".into())
}

fn reflog_entries(
    bytes: &[u8],
    truncated: bool,
    format: ObjectFormat,
) -> Result<Vec<ReflogEntry>, String> {
    if !truncated && !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err("Reflog lacks a terminated machine record".into());
    }
    let mut fields = nul_fields(bytes);
    let mut entries = Vec::new();
    while let Some(object) = fields.next() {
        let mut values = [None; 4];
        for value in &mut values {
            *value = fields.next();
        }
        let [Some(selector), Some(subject), Some(author), Some(email)] = values else {
            if truncated {
                break;
            }
            return Err("Incomplete reflog metadata record".into());
        };
        for field in [selector, subject, author, email] {
            metadata_field(field)?;
        }
        entries.push(ReflogEntry {
            object: ObjectId::parse(object, format)?,
            selector: selector.to_vec(),
            subject: subject.to_vec(),
            author: author.to_vec(),
            email: email.to_vec(),
        });
    }
    Ok(entries)
}
