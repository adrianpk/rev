use super::*;

impl Runner {
    pub(in crate::git::work) fn read_bisect(
        &mut self,
        sequence: &Sequence,
        admission: &mut Lease,
    ) -> Result<Option<BisectState>, String> {
        let value = |name: &[u8]| {
            sequence
                .files
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.bytes.as_slice())
        };
        if value(b"BISECT_TERMS") != Some(b"bad\ngood\n")
            || value(b"BISECT_NAMES") != Some(b"\n")
            || value(b"BISECT_HEAD").is_some()
            || !sequence.refs.head_ref.is_empty()
        {
            return Ok(None);
        }
        let format = sequence.refs.observation.identity.format;
        let expected = value(b"BISECT_EXPECTED_REV")
            .map(|b| ObjectId::parse(b.strip_suffix(b"\n").unwrap_or(b), format))
            .transpose()?;
        if expected.is_none() || expected != sequence.refs.head {
            return Ok(None);
        }
        let bytes = value(b"refs/bisect/").ok_or("Bisect refs are unavailable")?;
        let count = bytes
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .count();
        if count > 256 || count > self.policy.max_refs {
            return Err("Bisect refs exceed complete 256-ref admission".into());
        }
        admission.grow(2 * 256 * std::mem::size_of::<ObjectId>())?;
        let mut good = Vec::with_capacity(256);
        let mut skipped = Vec::with_capacity(256);
        let mut bad = None;
        for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
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
            if name == b"refs/bisect/bad" {
                bad = Some(id);
            } else if name.strip_prefix(b"refs/bisect/good-") == Some(object) {
                good.push(id);
            } else if name.strip_prefix(b"refs/bisect/skip-") == Some(object) {
                skipped.push(id);
            } else {
                return Ok(None);
            }
        }
        let Some(bad) = bad.filter(|_| !good.is_empty()) else {
            return Ok(None);
        };
        let count_bytes = self.bisect_range(bad, &good, true, admission)?;
        let remaining = std::str::from_utf8(&count_bytes)
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
            .ok_or("Bisect range is empty or unavailable")?;
        let bytes = self.bisect_range(bad, &good, false, admission)?;
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
        let result = if remaining == 1 && ids == [bad] {
            BisectResult::Found(bad)
        } else if remaining <= 256 && ids.iter().all(|id| *id == bad || skipped.contains(id)) {
            BisectResult::Ambiguous
        } else {
            BisectResult::Searching
        };
        if result == BisectResult::Searching {
            let head = expected.ok_or("Bisect expected candidate is unavailable")?;
            let member = if remaining <= 256 {
                ids.contains(&head)
            } else {
                self.bisect_ancestor(head, bad)?
                    && self.bisect_range(head, &good, true, admission)? != b"0\n"
            };
            if !member || head == bad || skipped.contains(&head) {
                return Err(
                    "Bisect current HEAD is not an unclassified candidate in the actual range"
                        .into(),
                );
            }
        }
        Ok(Some(BisectState {
            plan: None,
            bad,
            good,
            skipped,
            remaining,
            result,
        }))
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
