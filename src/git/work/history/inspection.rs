use super::*;
use crate::git::{Attribution, Blame, Reflog, ReflogEntry, RevisionFile};

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
        if !truncated && !bytes.is_empty() && bytes.last() != Some(&0) {
            return Err("Reflog lacks a terminated machine record".into());
        }
        let mut fields = nul_fields(&bytes);
        let mut entries = Vec::new();
        let complete = !truncated;
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
                object: ObjectId::parse(object, observation.identity.format)?,
                selector: selector.to_vec(),
                subject: subject.to_vec(),
                author: author.to_vec(),
                email: email.to_vec(),
            });
        }
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
        let mut records = bytes
            .split_inclusive(|b| *b == b'\n')
            .filter_map(|l| l.strip_suffix(b"\n"));
        let mut lines = Vec::new();
        let mut path_bytes = 0;
        let mut complete = !truncated;
        while let Some(header) = records.next() {
            if self.cancel.bounded(self.deadline).is_cancelled() {
                return Err("Blame parsing cancelled".into());
            }
            let mut parts = header.split(|b| *b == b' ');
            let object = ObjectId::parse(
                parts.next().ok_or("Blame lacks an object")?,
                observation.identity.format,
            )?;
            let original_line = number(parts.next().ok_or("Blame lacks an original line")?)?;
            let final_line = number(parts.next().ok_or("Blame lacks a final line")?)?;
            if final_line != lines.len() + 1 {
                return Err("Blame source lines are not a complete ordered prefix".into());
            }
            if let Some(count) = parts.next() {
                number(count)?;
            }
            if parts.next().is_some() {
                return Err("Malformed blame header".into());
            }
            let mut author = None;
            let mut email = None;
            let mut time = None;
            let mut subject = None;
            let mut filename = None;
            let mut text = None;
            for record in records.by_ref() {
                if let Some(value) = record.strip_prefix(b"\t") {
                    text = Some(value);
                    break;
                }
                let Some((key, value)) = record
                    .iter()
                    .position(|b| *b == b' ')
                    .map(|at| (&record[..at], &record[at + 1..]))
                else {
                    if record == b"boundary" {
                        continue;
                    }
                    return Err("Malformed blame metadata".into());
                };
                if key == b"filename" {
                    filename = Some(unquote_path(value)?);
                } else {
                    metadata_field(value)?;
                    match key {
                        b"author" => author = Some(value),
                        b"author-mail" => email = Some(value),
                        b"author-time" => time = Some(value),
                        b"summary" => subject = Some(value),
                        _ => {}
                    }
                }
            }
            let Some(text) = text else {
                if truncated {
                    break;
                }
                return Err("Incomplete blame source record".into());
            };
            let Some(filename) = filename else {
                return Err("Blame lacks an exact original filename".into());
            };
            path_bytes += filename.capacity();
            if path_bytes > self.policy.max_path_bytes {
                complete = false;
                break;
            }
            lines.push(Attribution {
                object,
                path: filename,
                original_line,
                final_line,
                author: author.ok_or("Blame lacks author metadata")?.to_vec(),
                email: email.ok_or("Blame lacks author mail")?.to_vec(),
                time: time.ok_or("Blame lacks author time")?.to_vec(),
                subject: subject.ok_or("Blame lacks summary")?.to_vec(),
                text: text.to_vec(),
            });
        }
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
        observation: Arc<Observation>,
        object: ObjectId,
        path: PathBuf,
        line: usize,
        current_path: Option<PathBuf>,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
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

// Git's C-quoted filename grammar preserves whitespace, quotes, backslashes
// and non-UTF-8 bytes. Escaped display strings never become path arguments.
pub(in crate::git::work) fn unquote_path(value: &[u8]) -> Result<PathBuf, String> {
    let bytes = if let Some(inner) = value.strip_prefix(b"\"") {
        let inner = inner
            .strip_suffix(b"\"")
            .ok_or("Unterminated quoted blame path")?;
        if inner.len() > 16384 {
            return Err("Quoted blame path exceeds its bound".into());
        }
        let mut bytes = Vec::with_capacity(inner.len().min(4096));
        let mut at = 0;
        while at < inner.len() {
            let mut b = inner[at];
            at += 1;
            if b == b'\\' {
                b = *inner.get(at).ok_or("Incomplete blame path escape")?;
                at += 1;
                b = match b {
                    b'\\' | b'"' => b,
                    b'a' => 7,
                    b'b' => 8,
                    b't' => b'\t',
                    b'n' => b'\n',
                    b'v' => 11,
                    b'f' => 12,
                    b'r' => b'\r',
                    b'0'..=b'7' => {
                        let mut v = u16::from(b - b'0');
                        for _ in 0..2 {
                            if let Some(c @ b'0'..=b'7') = inner.get(at) {
                                v = v * 8 + u16::from(c - b'0');
                                at += 1;
                            } else {
                                break;
                            }
                        }
                        u8::try_from(v).map_err(|_| "Invalid blame path octal escape")?
                    }
                    _ => return Err("Unsupported blame path escape".into()),
                };
            }
            if bytes.len() >= 4096 {
                return Err("Blame path exceeds 4096 bytes".into());
            }
            bytes.push(b);
        }
        bytes
    } else {
        if value.len() > 4096 {
            return Err("Blame path exceeds 4096 bytes".into());
        }
        value.to_vec()
    };
    validate_path(&bytes)?;
    Ok(PathBuf::from(OsString::from_vec(bytes)))
}
