use super::*;
use crate::git::conflicts::Request;
use std::os::unix::ffi::OsStrExt;

/// Stage reads share one cumulative detail allowance for this selected path.
struct Stages<'a> {
    request: &'a Request,
    sides: [Option<ConflictSide>; 3],
    remaining: usize,
}
struct StageRecord<'a> {
    ordinal: usize,
    mode: &'a [u8],
    object: &'a [u8],
}
struct Stage {
    mode: u32,
    object: ObjectId,
}
pub(super) fn read(
    runner: &mut Runner,
    request: &Request,
    admission: &mut Lease,
) -> Result<[Option<ConflictSide>; 3], String> {
    let mut stages = Stages {
        request,
        sides: [None, None, None],
        remaining: runner.policy.max_detail_bytes.min(budget::SCRATCH_BYTES),
    };
    for record in request
        .sequence
        .index
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
    {
        stages.record(record, runner, admission)?;
    }
    if stages.sides.iter().all(Option::is_none) {
        return Err("Selected conflict lacks actual unmerged index stages".into());
    }
    Ok(stages.sides)
}
impl Stages<'_> {
    fn record(
        &mut self,
        bytes: &[u8],
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let Some(record) = StageRecord::parse(bytes, &self.request.path)? else {
            return Ok(());
        };
        if self.sides[record.ordinal].is_some() {
            return Err("Conflict index has duplicate stages".into());
        }
        let stage = record.decode(self.request.sequence.refs.observation.identity.format)?;
        let side = stage.read(runner, self.remaining, admission)?;
        self.remaining = self.remaining.saturating_sub(side.bytes.len());
        self.sides[record.ordinal] = Some(side);
        Ok(())
    }
}
impl StageRecord<'_> {
    fn parse<'a>(record: &'a [u8], path: &Path) -> Result<Option<StageRecord<'a>>, String> {
        let at = record
            .iter()
            .position(|b| *b == b'\t')
            .ok_or("Index record lacks exact path")?;
        if &record[at + 1..] != path.as_os_str().as_bytes() {
            return Ok(None);
        }
        let fields: Vec<_> = record[..at].split(|b| *b == b' ').collect();
        if fields.len() != 3 || fields[2].len() != 1 || !(b'1'..=b'3').contains(&fields[2][0]) {
            return Err("Conflict index has an unsupported stage record".into());
        }
        Ok(Some(StageRecord {
            ordinal: usize::from(fields[2][0] - b'1'),
            mode: fields[0],
            object: fields[1],
        }))
    }
    fn decode(&self, format: ObjectFormat) -> Result<Stage, String> {
        let mode = u32::from_str_radix(
            std::str::from_utf8(self.mode).map_err(|_| "Invalid index mode")?,
            8,
        )
        .map_err(|_| "Invalid index mode")?;
        let object = ObjectId::parse(self.object, format)?;
        Ok(Stage { mode, object })
    }
}
impl Stage {
    fn read(
        self,
        runner: &mut Runner,
        remaining: usize,
        admission: &mut Lease,
    ) -> Result<ConflictSide, String> {
        let ordinary = matches!(self.mode, 0o100644 | 0o100755);
        let (bytes, truncated) = if ordinary && remaining != 0 {
            runner.command_os(
                &[
                    "--no-replace-objects".into(),
                    "cat-file".into(),
                    "blob".into(),
                    self.object.hex().into(),
                ],
                Phase::Refreshing,
                false,
                Some(remaining),
            )?
        } else {
            (Vec::new(), ordinary)
        };
        let binary = bytes.contains(&0);
        let text = ordinary && !truncated && !binary && std::str::from_utf8(&bytes).is_ok();
        let chunks = bytes.len().div_ceil(2048) + bytes.iter().filter(|b| **b == b'\n').count() + 1;
        admission.grow(
            bytes.capacity() + chunks * std::mem::size_of::<std::ops::Range<usize>>() + 512,
        )?;
        let lines = side_lines(&bytes, chunks);
        Ok(ConflictSide {
            mode: self.mode,
            object: self.object,
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
        })
    }
}
fn side_lines(bytes: &[u8], capacity: usize) -> Vec<std::ops::Range<usize>> {
    let mut lines = Vec::with_capacity(capacity);
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
    lines
}
pub(super) fn completeness(sequence: &Sequence, sides: &[Option<ConflictSide>; 3]) -> Completeness {
    if sequence.refs.completeness == Completeness::Complete
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
        })
    {
        Completeness::Complete
    } else {
        Completeness::Truncated {
            reason: "Conflict observation or side bound",
        }
    }
}
