use super::*;

impl Runner {
    pub(super) fn hunk_action(
        &mut self,
        direction: IndexDirection,
        diff: &Diff,
        ids: &[HunkId],
        admission: &mut Lease,
    ) -> Result<Prepared, String> {
        let mut hunks = ids
            .iter()
            .map(|id| diff.hunk(*id).ok_or("Hunk identity expired"))
            .collect::<Result<Vec<_>, _>>()?;
        hunks.sort_by_key(|h| h.patch.start);
        let base = self.hunk_base(diff)?;
        admission.grow(3 * base.len() + 4 * diff.evidence.len() + 16384)?;
        let reverse = direction == IndexDirection::Unstage;
        let mut content = base;
        // Descending ranges preserve the original index offsets, including
        // insertions/deletions that change the size of later selected hunks.
        for hunk in hunks.iter().rev() {
            content = hunk_result(&content, diff, hunk, reverse)?;
        }
        if content.len() > self.policy.max_detail_bytes {
            return Err("Expected index content exceeds detail admission".into());
        }
        let expected = self.expected_hunk_entry(diff, reverse, &content, admission)?;
        let patch = hunk_patch(diff, &hunks)?;
        self.checked_hunk_patch(&patch, reverse, expected)
    }

    fn checked_hunk_patch(
        &mut self,
        patch: &[u8],
        reverse: bool,
        expected: Entry,
    ) -> Result<Prepared, String> {
        let mut args: Vec<OsString> = vec![
            "apply".into(),
            "--cached".into(),
            "--whitespace=nowarn".into(),
        ];
        if reverse {
            args.push("--reverse".into());
        }
        let mut check = args.clone();
        check.push("--check".into());
        let input = crate::process::input::stage(Some(patch))?;
        self.command_argv(
            self.git_invocation(&check, input),
            Phase::Checking,
            false,
            None,
        )?;
        let input = crate::process::input::stage(Some(patch))?;
        Ok(Prepared {
            args,
            input,
            expected: vec![expected],
        })
    }

    fn hunk_base(&mut self, diff: &Diff) -> Result<Vec<u8>, String> {
        let object = diff
            .change
            .objects
            .get(1)
            .ok_or("Index blob identity unavailable")?;
        let base = if diff.change.modes.get(1) == Some(&0) {
            Vec::new()
        } else {
            let (bytes, limited) = self.command_os(
                &["cat-file".into(), "blob".into(), object.hex().into()],
                Phase::Checking,
                false,
                Some(self.policy.max_detail_bytes),
            )?;
            if limited {
                return Err("Index base blob exceeded its complete-detail cap".into());
            }
            bytes
        };
        Ok(base)
    }

    fn expected_hunk_entry(
        &mut self,
        diff: &Diff,
        reverse: bool,
        content: &[u8],
        admission: &mut Lease,
    ) -> Result<Entry, String> {
        let deleting = if reverse {
            diff.change.modes.first() == Some(&0)
        } else {
            diff.change.modes.get(2) == Some(&0)
        };
        let expected = if deleting {
            Entry {
                path: diff.change.path.clone(),
                mode: None,
                object: None,
            }
        } else {
            let hash = self.hash_input(content, admission)?;
            Entry {
                path: diff.change.path.clone(),
                mode: diff
                    .change
                    .modes
                    .get(1)
                    .copied()
                    .filter(|m| *m != 0)
                    .or_else(|| diff.change.modes.get(if reverse { 0 } else { 2 }).copied())
                    .filter(|mode| *mode != 0),
                object: Some(ObjectId::parse(
                    hash.strip_suffix(b"\n").unwrap_or(&hash),
                    diff.observation.identity.format,
                )?),
            }
        };
        Ok(expected)
    }
}

fn hunk_patch(diff: &Diff, hunks: &[&Hunk]) -> Result<Vec<u8>, String> {
    let header = diff
        .hunks
        .first()
        .ok_or("Missing comparison header")?
        .patch
        .start;
    let mut patch = Vec::with_capacity(header + hunks.iter().map(|h| h.patch.len()).sum::<usize>());
    // An executable-bit change is whole-path metadata, not part of a
    // selected text hunk. Creation/deletion mode headers remain necessary.
    for line in diff.evidence[..header].split_inclusive(|b| *b == b'\n') {
        if !line.starts_with(b"old mode ") && !line.starts_with(b"new mode ") {
            patch.extend_from_slice(line);
        }
    }
    for hunk in hunks {
        patch.extend_from_slice(&diff.evidence[hunk.patch.clone()]);
    }
    Ok(patch)
}

fn hunk_result(base: &[u8], diff: &Diff, hunk: &Hunk, reverse: bool) -> Result<Vec<u8>, String> {
    let HunkContent { old, new, .. } = HunkContent::parse(diff, hunk)?;
    let (range, before, replacement) = if reverse {
        (hunk.new, new, old)
    } else {
        (hunk.old, old, new)
    };
    let line = if range.count == 0 {
        range.start
    } else {
        range.start.checked_sub(1).ok_or("Invalid hunk start")?
    };
    let count = base.split_inclusive(|b| *b == b'\n').count();
    if line > count || range.count > count - line {
        return Err("Hunk range no longer matches index content".into());
    }
    let start: usize = base
        .split_inclusive(|b| *b == b'\n')
        .take(line)
        .map(<[u8]>::len)
        .sum();
    let end = start
        + base[start..]
            .split_inclusive(|b| *b == b'\n')
            .take(range.count)
            .map(<[u8]>::len)
            .sum::<usize>();
    if base[start..end] != before {
        return Err("Hunk bytes no longer match index content".into());
    }
    let mut result = Vec::with_capacity(base.len() - before.len() + replacement.len());
    result.extend_from_slice(&base[..start]);
    result.extend_from_slice(&replacement);
    result.extend_from_slice(&base[end..]);
    Ok(result)
}

/// The two exact hunk sides share content-prefix state for newline markers.
struct HunkContent {
    old: Vec<u8>,
    new: Vec<u8>,
    previous: u8,
}

impl HunkContent {
    fn parse(diff: &Diff, hunk: &Hunk) -> Result<Self, String> {
        let mut content = Self {
            old: Vec::new(),
            new: Vec::new(),
            previous: 0,
        };
        for line in diff.evidence[hunk.patch.clone()]
            .split_inclusive(|b| *b == b'\n')
            .skip(1)
        {
            content.record(line)?;
        }
        Ok(content)
    }

    fn record(&mut self, line: &[u8]) -> Result<(), String> {
        let prefix = *line.first().ok_or("Empty hunk record")?;
        match prefix {
            b' ' => {
                self.old.extend_from_slice(&line[1..]);
                self.new.extend_from_slice(&line[1..]);
            }
            b'-' => self.old.extend_from_slice(&line[1..]),
            b'+' => self.new.extend_from_slice(&line[1..]),
            b'\\' => {
                self.no_newline();
                return Ok(());
            }
            _ => return Err("Unsupported hunk record".into()),
        }
        self.previous = prefix;
        Ok(())
    }

    fn no_newline(&mut self) {
        if matches!(self.previous, b' ' | b'-') && self.old.last() == Some(&b'\n') {
            self.old.pop();
        }
        if matches!(self.previous, b' ' | b'+') && self.new.last() == Some(&b'\n') {
            self.new.pop();
        }
    }
}
