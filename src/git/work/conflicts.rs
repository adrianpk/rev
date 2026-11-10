use super::*;
mod explanation;
mod resolution;
mod stages;
use crate::git::{Conflict, ConflictRole, ConflictSide};
use std::os::unix::ffi::OsStrExt;

impl Runner {
    pub(super) fn conflict(
        &mut self,
        request: super::super::conflicts::Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let sequence = &request.sequence;
        self.checked_sequence(sequence, admission)?;
        let change = sequence
            .refs
            .observation
            .status
            .as_ref()
            .and_then(|s| s.group(Group::Unmerged).find(|c| c.path == request.path))
            .ok_or("Exact unmerged path disappeared")?
            .clone();
        let sides = stages::read(self, &request, admission)?;
        self.checked_sequence(sequence, admission)?;
        let completeness = stages::completeness(sequence, &sides);
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
            sequence: request.sequence,
            change,
            sides,
            completeness,
            _admission: admission.split(retained),
        }))
    }
}
