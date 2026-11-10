use super::*;

/// Complete native ref evidence separates unchecked records from boundaries.
struct References {
    bad: Option<ObjectId>,
    good: Vec<ObjectId>,
    skipped: Vec<ObjectId>,
}
struct Boundaries {
    bad: ObjectId,
    good: Vec<ObjectId>,
    skipped: Vec<ObjectId>,
}
struct BoundaryRef<'a> {
    name: &'a [u8],
    object: &'a [u8],
    id: ObjectId,
}
struct Candidates {
    remaining: usize,
    ids: Vec<ObjectId>,
}
impl Boundaries {
    fn read(
        bytes: &[u8],
        format: ObjectFormat,
        max_refs: usize,
        admission: &mut Lease,
    ) -> Result<Option<Self>, String> {
        let count = bytes
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .count();
        if count > 256 || count > max_refs {
            return Err("Bisect refs exceed complete 256-ref admission".into());
        }
        admission.grow(2 * 256 * std::mem::size_of::<ObjectId>())?;
        let mut refs = References {
            bad: None,
            good: Vec::with_capacity(256),
            skipped: Vec::with_capacity(256),
        };
        for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
            let Some(record) = BoundaryRef::parse(line, format)? else {
                return Ok(None);
            };
            if !refs.admit(&record) {
                return Ok(None);
            }
        }
        Ok(refs.finish())
    }
}
impl BoundaryRef<'_> {
    fn parse(line: &[u8], format: ObjectFormat) -> Result<Option<BoundaryRef<'_>>, String> {
        if line.len() > 4096 {
            return Err("Bisect ref metadata exceeds 4096 bytes".into());
        }
        let mut fields = line.split(|b| *b == b' ');
        let (Some(name), Some(object), Some(kind), Some(symbolic)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err("Incomplete bisect ref record".into());
        };
        if fields.next().is_some() || kind != b"commit" || !symbolic.is_empty() {
            return Ok(None);
        }
        let id = ObjectId::parse(object, format)?;
        Ok(Some(BoundaryRef { name, object, id }))
    }
}
impl References {
    fn admit(&mut self, record: &BoundaryRef<'_>) -> bool {
        if record.name == b"refs/bisect/bad" {
            self.bad = Some(record.id);
        } else if record.name.strip_prefix(b"refs/bisect/good-") == Some(record.object) {
            self.good.push(record.id);
        } else if record.name.strip_prefix(b"refs/bisect/skip-") == Some(record.object) {
            self.skipped.push(record.id);
        } else {
            return false;
        }
        true
    }
    fn finish(self) -> Option<Boundaries> {
        let bad = self.bad.filter(|_| !self.good.is_empty())?;
        Some(Boundaries {
            bad,
            good: self.good,
            skipped: self.skipped,
        })
    }
}
impl Candidates {
    fn result(&self, boundaries: &Boundaries) -> BisectResult {
        if self.remaining == 1 && self.ids == [boundaries.bad] {
            BisectResult::Found(boundaries.bad)
        } else if self.remaining <= 256
            && self
                .ids
                .iter()
                .all(|id| *id == boundaries.bad || boundaries.skipped.contains(id))
        {
            BisectResult::Ambiguous
        } else {
            BisectResult::Searching
        }
    }
}
fn value<'a>(sequence: &'a Sequence, name: &[u8]) -> Option<&'a [u8]> {
    sequence
        .files
        .iter()
        .find(|f| f.name == name)
        .map(|f| f.bytes.as_slice())
}
fn expected_candidate(sequence: &Sequence) -> Result<Option<ObjectId>, String> {
    if value(sequence, b"BISECT_TERMS") != Some(b"bad\ngood\n")
        || value(sequence, b"BISECT_NAMES") != Some(b"\n")
        || value(sequence, b"BISECT_HEAD").is_some()
        || !sequence.refs.head_ref.is_empty()
    {
        return Ok(None);
    }
    let expected = value(sequence, b"BISECT_EXPECTED_REV")
        .map(|b| {
            ObjectId::parse(
                b.strip_suffix(b"\n").unwrap_or(b),
                sequence.refs.observation.identity.format,
            )
        })
        .transpose()?;
    if expected.is_none() || expected != sequence.refs.head {
        return Ok(None);
    }
    Ok(expected)
}
impl Runner {
    pub(in crate::git::work) fn read_bisect(
        &mut self,
        sequence: &Sequence,
        admission: &mut Lease,
    ) -> Result<Option<BisectState>, String> {
        let expected = expected_candidate(sequence)?;
        if expected.is_none() {
            return Ok(None);
        }
        let bytes = value(sequence, b"refs/bisect/").ok_or("Bisect refs are unavailable")?;
        let Some(boundaries) = Boundaries::read(
            bytes,
            sequence.refs.observation.identity.format,
            self.policy.max_refs,
            admission,
        )?
        else {
            return Ok(None);
        };
        self.observe_bisect_boundaries(sequence, boundaries, expected, admission)
    }
    fn observe_bisect_boundaries(
        &mut self,
        sequence: &Sequence,
        boundaries: Boundaries,
        expected: Option<ObjectId>,
        admission: &mut Lease,
    ) -> Result<Option<BisectState>, String> {
        let remaining = self.bisect_remaining(&boundaries, admission)?;
        let candidates = self.bisect_candidates(
            &boundaries,
            remaining,
            sequence.refs.observation.identity.format,
            admission,
        )?;
        let result = candidates.result(&boundaries);
        if result == BisectResult::Searching {
            self.check_bisect_candidate(&boundaries, expected, &candidates, admission)?;
        }
        Ok(Some(BisectState {
            plan: None,
            bad: boundaries.bad,
            good: boundaries.good,
            skipped: boundaries.skipped,
            remaining,
            result,
        }))
    }
    fn bisect_remaining(
        &mut self,
        boundaries: &Boundaries,
        admission: &mut Lease,
    ) -> Result<usize, String> {
        let bytes = self.bisect_range(boundaries.bad, &boundaries.good, true, admission)?;
        std::str::from_utf8(&bytes)
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
            .ok_or_else(|| "Bisect range is empty or unavailable".into())
    }
    fn bisect_candidates(
        &mut self,
        boundaries: &Boundaries,
        remaining: usize,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Candidates, String> {
        let bytes = self.bisect_range(boundaries.bad, &boundaries.good, false, admission)?;
        admission.grow(bytes.len() * 2 + 257 * std::mem::size_of::<ObjectId>())?;
        let mut ids = Vec::with_capacity(257);
        for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
            if ids.len() == 257 {
                return Err("Bisect range exceeds finite candidate admission".into());
            }
            ids.push(ObjectId::parse(line, format)?);
        }
        if ids.len() > 257 || ids.len() != remaining.min(257) {
            return Err("Bisect range changed during observation".into());
        }
        Ok(Candidates { remaining, ids })
    }
    fn check_bisect_candidate(
        &mut self,
        boundaries: &Boundaries,
        expected: Option<ObjectId>,
        candidates: &Candidates,
        admission: &mut Lease,
    ) -> Result<(), String> {
        let head = expected.ok_or("Bisect expected candidate is unavailable")?;
        let member = if candidates.remaining <= 256 {
            candidates.ids.contains(&head)
        } else {
            self.bisect_ancestor(head, boundaries.bad)?
                && self.bisect_range(head, &boundaries.good, true, admission)? != b"0\n"
        };
        if !member || head == boundaries.bad || boundaries.skipped.contains(&head) {
            return Err(
                "Bisect current HEAD is not an unclassified candidate in the actual range".into(),
            );
        }
        Ok(())
    }
    fn bisect_range(
        &mut self,
        bad: ObjectId,
        good: &[ObjectId],
        count: bool,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let size = (good.len() + 1) * 66;
        admission.grow(size * 2)?;
        let mut input = Vec::with_capacity(size);
        input.extend_from_slice(bad.hex().as_bytes());
        input.push(b'\n');
        for id in good {
            input.push(b'^');
            input.extend_from_slice(id.hex().as_bytes());
            input.push(b'\n');
        }
        let file = crate::process::input::stage(Some(&input))?;
        self.driver_admission = Some(admission.split(size));
        let args = [
            "--no-replace-objects".into(),
            "rev-list".into(),
            if count {
                "--count".into()
            } else {
                "--max-count=257".into()
            },
            "--stdin".into(),
        ];
        let result = self.command_argv(
            self.git_invocation(&args, file),
            Phase::Verifying,
            false,
            Some(self.policy.max_detail_bytes),
        );
        if self.retained.is_none() {
            self.driver_admission = None;
        }
        let (bytes, limited) = result?;
        if limited || (count && bytes.len() > 32) {
            return Err("Bisect range exceeds admitted output".into());
        }
        Ok(bytes)
    }
    pub(super) fn bisect_ancestor(&mut self, a: ObjectId, b: ObjectId) -> Result<bool, String> {
        self.command(
            &[
                "--no-replace-objects",
                "merge-base",
                "--is-ancestor",
                &a.hex(),
                &b.hex(),
            ],
            Phase::Verifying,
            true,
        )?;
        match self.exit.and_then(|s| s.code()) {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err("Bisect ancestry check failed".into()),
        }
    }
    pub(super) fn bisect_exit_two(&self, facts: &index::Execution) -> bool {
        facts.code() == Some(2)
    }
}
