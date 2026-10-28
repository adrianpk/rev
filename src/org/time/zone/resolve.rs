use super::super::{CivilDateTime, OperationTime};
use super::Zone;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedTime {
    pub unix_seconds: i64,
    pub offset_seconds: i32,
}
/// A fold retains every valid offset choice; a gap requires another civil time.
/// Unknown zone coverage is an error, never disguised as a nonexistent time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    Unique(ResolvedTime),
    Ambiguous(Vec<ResolvedTime>),
    Nonexistent,
}
impl Resolution {
    pub fn candidates(&self) -> &[ResolvedTime] {
        match self {
            Self::Unique(value) => std::slice::from_ref(value),
            Self::Ambiguous(values) => values,
            Self::Nonexistent => &[],
        }
    }
    /// Select only an observed valid offset. A gap cannot be repaired by forcing
    /// an offset that the zone does not actually observe at that instant.
    pub fn choose_offset(&self, offset_seconds: i32) -> Result<ResolvedTime, String> {
        self.candidates()
            .iter()
            .find(|c| c.offset_seconds == offset_seconds)
            .copied()
            .ok_or_else(|| "Choose an observed offset or enter a valid local time".into())
    }
}
impl Zone {
    pub fn civil_at(&self, unix_seconds: i64) -> Result<CivilDateTime, String> {
        let offset = self.offset_at(unix_seconds)?;
        OperationTime {
            seconds: unix_seconds,
        }
        .civil_at_offset(offset)
    }
    /// At most 258 offsets are checked against the same immutable observation.
    /// Candidates are ordered by UTC instant, independently of input type order.
    pub fn resolve(&self, local: CivilDateTime) -> Result<Resolution, String> {
        let offsets = self.candidate_offsets();
        if offsets.is_empty() {
            return Err("TZif has no known offset candidates".into());
        }
        let mut candidates = Vec::new();
        for offset in offsets {
            let unix_seconds = local.seconds() - i64::from(offset);
            if self.offset_at(unix_seconds)? == offset {
                candidates.push(ResolvedTime {
                    unix_seconds,
                    offset_seconds: offset,
                });
            }
        }
        candidates.sort_by_key(|c| c.unix_seconds);
        Ok(match candidates.len() {
            0 => Resolution::Nonexistent,
            1 => Resolution::Unique(candidates[0]),
            _ => Resolution::Ambiguous(candidates),
        })
    }
    fn candidate_offsets(&self) -> BTreeSet<i32> {
        let mut values: BTreeSet<_> = self.offsets.iter().flatten().copied().collect();
        if let Some(future) = &self.future {
            values.insert(future.standard);
            if let Some(dst) = &future.daylight {
                values.insert(dst.offset);
            }
        }
        values
    }
}
