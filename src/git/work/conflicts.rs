use super::*;
use crate::git::{Conflict, ConflictRole, ConflictSide};
use std::os::unix::ffi::OsStrExt;

impl Runner {
    pub(super) fn conflict(
        &mut self,
        request: super::super::conflicts::Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let sequence = request.sequence;
        self.checked_sequence(&sequence, admission)?;
        let change = sequence
            .refs
            .observation
            .status
            .as_ref()
            .and_then(|s| s.group(Group::Unmerged).find(|c| c.path == request.path))
            .ok_or("Exact unmerged path disappeared")?
            .clone();
        let mut sides: [Option<ConflictSide>; 3] = [None, None, None];
        let mut remaining = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        for record in sequence.index.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let at = record
                .iter()
                .position(|b| *b == b'\t')
                .ok_or("Index record lacks exact path")?;
            if &record[at + 1..] != request.path.as_os_str().as_bytes() {
                continue;
            }
            let fields: Vec<_> = record[..at].split(|b| *b == b' ').collect();
            if fields.len() != 3 || fields[2].len() != 1 || !(b'1'..=b'3').contains(&fields[2][0]) {
                return Err("Conflict index has an unsupported stage record".into());
            }
            let stage = usize::from(fields[2][0] - b'1');
            if sides[stage].is_some() {
                return Err("Conflict index has duplicate stages".into());
            }
            let mode = u32::from_str_radix(
                std::str::from_utf8(fields[0]).map_err(|_| "Invalid index mode")?,
                8,
            )
            .map_err(|_| "Invalid index mode")?;
            let object = ObjectId::parse(fields[1], sequence.refs.observation.identity.format)?;
            let ordinary = matches!(mode, 0o100644 | 0o100755);
            let (bytes, truncated) = if ordinary && remaining != 0 {
                self.command_os(
                    &[
                        "--no-replace-objects".into(),
                        "cat-file".into(),
                        "blob".into(),
                        object.hex().into(),
                    ],
                    Phase::Refreshing,
                    false,
                    Some(remaining),
                )?
            } else {
                (Vec::new(), ordinary)
            };
            remaining = remaining.saturating_sub(bytes.len());
            let binary = bytes.contains(&0);
            let text = ordinary && !truncated && !binary && std::str::from_utf8(&bytes).is_ok();
            let chunks =
                bytes.len().div_ceil(2048) + bytes.iter().filter(|b| **b == b'\n').count() + 1;
            admission.grow(
                bytes.capacity() + chunks * std::mem::size_of::<std::ops::Range<usize>>() + 512,
            )?;
            let mut lines = Vec::with_capacity(chunks);
            let mut offset = 0;
            for line in bytes.split_inclusive(|b| *b == b'\n') {
                let content = line.strip_suffix(b"\n").unwrap_or(line);
                if content.is_empty() {
                    lines.push(offset..offset);
                }
                for start in (0..content.len()).step_by(2048) {
                    lines.push(offset + start..offset + (start + 2048).min(content.len()));
                }
                offset += line.len();
            }
            sides[stage] = Some(ConflictSide {
                mode,
                object,
                bytes,
                binary,
                text,
                lines,
                completeness: if truncated {
                    Completeness::Truncated {
                        reason: "Conflict side detail bound",
                    }
                } else {
                    Completeness::Complete
                },
            });
        }
        if sides.iter().all(Option::is_none) {
            return Err("Selected conflict lacks actual unmerged index stages".into());
        }
        self.checked_sequence(&sequence, admission)?;
        let completeness = if sequence.refs.completeness == Completeness::Complete
            && sequence
                .refs
                .observation
                .status
                .as_ref()
                .is_some_and(|s| s.completeness == Completeness::Complete)
            && ConflictRole::ALL.into_iter().all(|r| {
                sides[r.stage() - 1]
                    .as_ref()
                    .is_none_or(|s| s.completeness == Completeness::Complete)
            }) {
            Completeness::Complete
        } else {
            Completeness::Truncated {
                reason: "Conflict observation or side bound",
            }
        };
        let retained = std::mem::size_of::<Conflict>()
            + change.bytes()
            + sides
                .iter()
                .flatten()
                .map(|s| {
                    s.bytes.capacity()
                        + s.lines.capacity() * std::mem::size_of::<std::ops::Range<usize>>()
                })
                .sum::<usize>();
        admission.grow(retained)?;
        Ok(Candidate::Conflict(Conflict {
            id: self.progress.lock().expect("Git progress owner").operation,
            sequence,
            change,
            sides,
            completeness,
            _admission: admission.split(retained),
        }))
    }
}
