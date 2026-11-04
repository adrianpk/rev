use super::*;
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    path: PathBuf,
    mode: Option<u32>,
    object: Option<ObjectId>,
}
struct Prepared {
    args: Vec<OsString>,
    input: Option<fs::File>,
    expected: Vec<Entry>,
}
struct Execution {
    argv: Vec<OsString>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit: Option<std::process::ExitStatus>,
    truncated: bool,
    end: capture::End,
}

impl Runner {
    pub(super) fn change_index(
        &mut self,
        request: super::super::index::Request,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let diff = request.targets[0].diff();
        self.check_index_comparisons(&request.targets, admission)?;
        let Prepared {
            args,
            input,
            expected,
        } = match &request.targets[0] {
            IndexTarget::Path(_) => {
                self.path_group(request.direction, &request.targets, admission)?
            }
            IndexTarget::Hunk { .. } => {
                let hunks: Vec<_> = request
                    .targets
                    .iter()
                    .filter_map(|t| match t {
                        IndexTarget::Hunk { hunk, .. } => Some(*hunk),
                        _ => None,
                    })
                    .collect();
                self.hunk_action(request.direction, diff, &hunks, admission)?
            }
        };
        // All captured comparisons are checked again after hashing/filtering
        // and patch preflight. No subset has effects while another is checked.
        self.check_index_comparisons(&request.targets, admission)?;
        let executed = self.command_input(&args, Phase::Executing, false, None, input);
        self.effects = if self.exit.is_some() || self.retained.is_some() {
            Effects::Possible
        } else {
            Effects::NotStarted
        };
        let facts = Execution {
            argv: std::mem::take(&mut self.argv),
            stdout: std::mem::take(&mut self.stdout),
            stderr: std::mem::take(&mut self.stderr),
            exit: self.exit.take(),
            truncated: self.truncated,
            end: self.end,
        };
        // A failed/interrupted changing child is never replayed. Refresh is
        // attempted under the remaining original deadline and cancellation.
        let result = self.verify_index_result(diff, &expected, admission);
        self.argv = facts.argv;
        self.stdout = facts.stdout;
        self.stderr = facts.stderr;
        self.exit = facts.exit;
        self.truncated = facts.truncated;
        if executed.is_err() {
            self.end = facts.end;
        }
        match result {
            Ok((candidate, verified)) => {
                match (executed, verified) {
                    (Ok(_), Ok(())) => self.effects = Effects::Verified,
                    (Err(error), _) => {
                        self.operation_failure = Some(format!(
                            "Index child failed after possible effects: {error}"
                        ))
                    }
                    (_, Err(error)) => self.operation_failure = Some(error),
                }
                Ok(candidate)
            }
            Err(error) => {
                self.operation_failure = Some(format!(
                    "Index effects remain unverified; refresh required: {error}"
                ));
                Err(error)
            }
        }
    }

    fn check_index_comparisons(
        &mut self,
        targets: &[IndexTarget],
        admission: &mut Lease,
    ) -> Result<(), String> {
        self.check_index_status(targets, admission)?;
        for (i, selected) in targets.iter().enumerate() {
            let diff = selected.diff();
            if targets[..i].iter().any(|t| t.diff().id == diff.id) {
                continue;
            }
            let target = diff::Target {
                observation: diff.observation.clone(),
                group: diff.group,
                change: diff.change.clone(),
            };
            let (bytes, limited) = self.command_os(
                &self.diff_args(&target),
                Phase::Checking,
                target.group == Group::Untracked,
                Some(self.policy.max_detail_bytes),
            )?;
            if limited || bytes != diff.evidence {
                return Err(
                    "Index group comparison changed or is incomplete; refresh required".into(),
                );
            }
        }
        self.check_index_status(targets, admission)
    }

    fn check_index_status(
        &mut self,
        targets: &[IndexTarget],
        admission: &mut Lease,
    ) -> Result<(), String> {
        let (identity, status) = self.observe(admission)?;
        let current = status.as_ref().ok_or("Working-tree Status unavailable")?;
        for target in targets {
            let diff = target.diff();
            let prior = diff
                .observation
                .status
                .as_ref()
                .ok_or("Captured Status unavailable")?;
            if identity != diff.observation.identity
                || current.head != prior.head
                || current.completeness != Completeness::Complete
                || current
                    .group(diff.group)
                    .find(|p| p.path == diff.change.path)
                    != Some(&diff.change)
            {
                return Err(
                    "Index group path, layer or repository changed; refresh required".into(),
                );
            }
        }
        Ok(())
    }

    fn path_group(
        &mut self,
        direction: IndexDirection,
        targets: &[IndexTarget],
        admission: &mut Lease,
    ) -> Result<Prepared, String> {
        if targets.len() == 1 {
            return self.path_action(direction, targets[0].diff(), admission);
        }
        let count = targets
            .iter()
            .map(|t| {
                if t.diff().change.renamed_in(t.diff().group) {
                    2
                } else {
                    1
                }
            })
            .sum::<usize>();
        let path_bytes = targets
            .iter()
            .map(|t| {
                let d = t.diff();
                d.change.path.as_os_str().len()
                    + 1
                    + d.change
                        .old_path
                        .as_ref()
                        .filter(|_| d.change.renamed_in(d.group))
                        .map_or(0, |p| p.as_os_str().len() + 1)
            })
            .sum::<usize>();
        if path_bytes > budget::SCRATCH_BYTES {
            return Err("Index group path manifest exceeds worker scratch admission".into());
        }
        admission.grow(2 * path_bytes + count * (std::mem::size_of::<Entry>() + 128))?;
        let mut expected = Vec::with_capacity(count);
        for target in targets {
            expected.extend(
                self.path_action(direction, target.diff(), admission)?
                    .expected,
            );
        }
        let diff = targets[0].diff();
        let mut args: Vec<OsString> = vec!["--literal-pathspecs".into()];
        match direction {
            IndexDirection::Stage => args.extend([
                "add".into(),
                "--pathspec-from-file=-".into(),
                "--pathspec-file-nul".into(),
            ]),
            IndexDirection::Unstage => match &diff
                .observation
                .status
                .as_ref()
                .ok_or("Missing Status")?
                .head
            {
                Head::Unborn { .. } => args.extend([
                    "update-index".into(),
                    "--force-remove".into(),
                    "-z".into(),
                    "--stdin".into(),
                ]),
                Head::Branch { object, .. } | Head::Detached { object } => args.extend([
                    "restore".into(),
                    "--staged".into(),
                    format!("--source={}", object.hex()).into(),
                    "--pathspec-from-file=-".into(),
                    "--pathspec-file-nul".into(),
                ]),
            },
        }
        let mut manifest = Vec::with_capacity(path_bytes);
        for entry in &expected {
            manifest.extend_from_slice(entry.path.as_os_str().as_bytes());
            manifest.push(0);
        }
        let input = crate::process::input::stage(Some(&manifest))?;
        Ok(Prepared {
            args,
            input,
            expected,
        })
    }

    fn path_action(
        &mut self,
        direction: IndexDirection,
        diff: &Diff,
        admission: &mut Lease,
    ) -> Result<Prepared, String> {
        let mut paths = Vec::new();
        if diff.change.renamed_in(diff.group)
            && let Some(old) = &diff.change.old_path
        {
            paths.push(old.clone());
        }
        paths.push(diff.change.path.clone());
        let mut args: Vec<OsString> = vec!["--literal-pathspecs".into()];
        let expected = match direction {
            IndexDirection::Stage => {
                args.push("add".into());
                paths
                    .iter()
                    .map(|path| self.worktree_entry(diff, path, admission))
                    .collect::<Result<Vec<_>, _>>()?
            }
            IndexDirection::Unstage => match &diff
                .observation
                .status
                .as_ref()
                .ok_or("Missing Status")?
                .head
            {
                Head::Unborn { .. } => {
                    args.extend(["update-index".into(), "--force-remove".into()]);
                    paths
                        .iter()
                        .map(|path| Entry {
                            path: path.clone(),
                            mode: None,
                            object: None,
                        })
                        .collect()
                }
                Head::Branch { object, .. } | Head::Detached { object } => {
                    args.extend([
                        "restore".into(),
                        "--staged".into(),
                        format!("--source={}", object.hex()).into(),
                    ]);
                    let mut tree_args: Vec<OsString> = vec![
                        "--literal-pathspecs".into(),
                        "ls-tree".into(),
                        "-z".into(),
                        "--full-tree".into(),
                        object.hex().into(),
                        "--".into(),
                    ];
                    tree_args.extend(paths.iter().map(|p| p.as_os_str().to_owned()));
                    let (bytes, _) = self.command_os(&tree_args, Phase::Checking, false, None)?;
                    let entries =
                        entries(&bytes, diff.observation.identity.format, false, paths.len())?;
                    paths
                        .iter()
                        .map(|path| {
                            entries
                                .iter()
                                .find(|e| e.path == *path)
                                .cloned()
                                .unwrap_or_else(|| Entry {
                                    path: path.clone(),
                                    mode: None,
                                    object: None,
                                })
                        })
                        .collect()
                }
            },
        };
        args.push("--".into());
        args.extend(paths.iter().map(|p| p.as_os_str().to_owned()));
        Ok(Prepared {
            args,
            input: None,
            expected,
        })
    }

    fn worktree_entry(
        &mut self,
        diff: &Diff,
        path: &Path,
        admission: &mut Lease,
    ) -> Result<Entry, String> {
        let absolute = self.directory.join(path);
        let metadata = match fs::symlink_metadata(&absolute) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Entry {
                    path: path.to_owned(),
                    mode: None,
                    object: None,
                });
            }
            Err(error) => return Err(format!("Read index source metadata: {error}")),
        };
        let mode = if diff.group == Group::Untracked {
            Some(if metadata.is_dir() {
                0o160000
            } else if metadata.file_type().is_symlink() {
                0o120000
            } else {
                let configured = self.command(
                    &["config", "--bool", "--get", "core.filemode"],
                    Phase::Checking,
                    true,
                )?;
                let executable = match configured.as_slice() {
                    b"false\n" => false,
                    b"true\n" | b"" => metadata.permissions().mode() & 0o111 != 0,
                    _ => return Err("Git file-mode policy is unavailable".into()),
                };
                if executable { 0o100755 } else { 0o100644 }
            })
        } else {
            diff.change.modes.get(2).copied()
        };
        let bytes = if metadata.is_dir() {
            let prefix = self
                .command_os(
                    &[
                        "-C".into(),
                        path.as_os_str().to_owned(),
                        "rev-parse".into(),
                        "--show-prefix".into(),
                    ],
                    Phase::Checking,
                    false,
                    None,
                )?
                .0;
            if prefix != b"\n" {
                return Err("Selected gitlink has no available working repository".into());
            }
            let args = [
                OsString::from("-C"),
                path.as_os_str().to_owned(),
                "rev-parse".into(),
                "--verify".into(),
                "HEAD".into(),
            ];
            self.command_os(&args, Phase::Checking, false, None)?.0
        } else if metadata.file_type().is_symlink() {
            let target =
                fs::read_link(&absolute).map_err(|e| format!("Read index symlink: {e}"))?;
            self.hash_input(target.as_os_str().as_bytes(), admission)?
        } else {
            let mut option = OsString::from("--path=");
            option.push(path);
            self.command_os(
                &[
                    "hash-object".into(),
                    option,
                    "--".into(),
                    path.as_os_str().to_owned(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0
        };
        Ok(Entry {
            path: path.to_owned(),
            mode,
            object: Some(ObjectId::parse(
                bytes.strip_suffix(b"\n").unwrap_or(&bytes),
                diff.observation.identity.format,
            )?),
        })
    }

    fn hash_input(&mut self, bytes: &[u8], admission: &mut Lease) -> Result<Vec<u8>, String> {
        admission.grow(bytes.len())?;
        let input = crate::process::input::stage(Some(bytes))?;
        self.command_input(
            &["hash-object".into(), "--stdin".into()],
            Phase::Checking,
            false,
            None,
            input,
        )
        .map(|(bytes, _)| bytes)
    }

    fn hunk_action(
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
            let hash = self.hash_input(&content, admission)?;
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
        let header = diff
            .hunks
            .first()
            .ok_or("Missing comparison header")?
            .patch
            .start;
        let mut patch =
            Vec::with_capacity(header + hunks.iter().map(|h| h.patch.len()).sum::<usize>());
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
        let input = crate::process::input::stage(Some(&patch))?;
        self.command_input(&check, Phase::Checking, false, None, input)?;
        let input = crate::process::input::stage(Some(&patch))?;
        Ok(Prepared {
            args,
            input,
            expected: vec![expected],
        })
    }

    fn verify_index_result(
        &mut self,
        diff: &Diff,
        expected: &[Entry],
        admission: &mut Lease,
    ) -> Result<(Candidate, Result<(), String>), String> {
        let (identity, status) = self.observe(admission)?;
        let mut actual = Vec::with_capacity(expected.len());
        // ls-files has no input manifest. Bounded read-only batches preserve
        // the 64-token argv bound without breaking the changing action apart.
        for batch in expected.chunks(12) {
            let mut args: Vec<OsString> = vec![
                "--literal-pathspecs".into(),
                "ls-files".into(),
                "--stage".into(),
                "-z".into(),
                "--".into(),
            ];
            args.extend(batch.iter().map(|e| e.path.as_os_str().to_owned()));
            let (bytes, _) = self.command_os(&args, Phase::Verifying, false, None)?;
            actual.extend(entries(
                &bytes,
                diff.observation.identity.format,
                true,
                batch.len(),
            )?);
        }
        let valid = actual
            .iter()
            .all(|entry| expected.iter().any(|e| e.path == entry.path))
            && actual
                .iter()
                .enumerate()
                .all(|(i, entry)| actual[..i].iter().all(|e| e.path != entry.path))
            && identity == diff.observation.identity
            && status.as_ref().map(|s| &s.head)
                == diff.observation.status.as_ref().map(|s| &s.head)
            && expected.iter().all(|expected| match expected.object {
                None => !actual.iter().any(|e| e.path == expected.path),
                Some(object) => actual.iter().any(|e| {
                    e.path == expected.path
                        && e.object == Some(object)
                        && expected.mode.is_none_or(|mode| e.mode == Some(mode))
                }),
            });
        Ok((
            Candidate::Status(identity, status),
            if valid {
                Ok(())
            } else {
                Err("Index postcondition differs from the captured request; effects remain uncertain".into())
            },
        ))
    }
}

fn entries(
    bytes: &[u8],
    format: ObjectFormat,
    index: bool,
    limit: usize,
) -> Result<Vec<Entry>, String> {
    if bytes.iter().filter(|b| **b == 0).count() > limit {
        return Err("Index entries exceed the exact target count".into());
    }
    if !bytes.is_empty() && bytes.last() != Some(&0) {
        return Err("Index entries lack their NUL boundary".into());
    }
    bytes
        .split(|b| *b == 0)
        .filter(|part| !part.is_empty())
        .map(|record| {
            let tab = record
                .iter()
                .position(|b| *b == b'\t')
                .ok_or("Index entry lacks its path boundary")?;
            let mut fields = record[..tab].split(|b| *b == b' ');
            let parts = [
                fields.next().ok_or("Missing index mode")?,
                fields.next().ok_or("Missing index object")?,
                fields.next().ok_or("Missing index stage/type")?,
            ];
            if fields.next().is_some() || (index && parts[2] != b"0") {
                return Err("Index entry is malformed or unmerged".into());
            }
            let mode = std::str::from_utf8(parts[0])
                .ok()
                .and_then(|s| u32::from_str_radix(s, 8).ok())
                .ok_or("Invalid index mode")?;
            Ok(Entry {
                path: PathBuf::from(OsString::from_vec(record[tab + 1..].to_vec())),
                mode: Some(mode),
                object: Some(ObjectId::parse(parts[if index { 1 } else { 2 }], format)?),
            })
        })
        .collect()
}

fn hunk_result(base: &[u8], diff: &Diff, hunk: &Hunk, reverse: bool) -> Result<Vec<u8>, String> {
    let mut old = Vec::new();
    let mut new = Vec::new();
    let mut previous = 0;
    for line in diff.evidence[hunk.patch.clone()]
        .split_inclusive(|b| *b == b'\n')
        .skip(1)
    {
        let prefix = *line.first().ok_or("Empty hunk record")?;
        match prefix {
            b' ' => {
                old.extend_from_slice(&line[1..]);
                new.extend_from_slice(&line[1..]);
            }
            b'-' => old.extend_from_slice(&line[1..]),
            b'+' => new.extend_from_slice(&line[1..]),
            b'\\' => {
                if matches!(previous, b' ' | b'-') && old.last() == Some(&b'\n') {
                    old.pop();
                }
                if matches!(previous, b' ' | b'+') && new.last() == Some(&b'\n') {
                    new.pop();
                }
                continue;
            }
            _ => return Err("Unsupported hunk record".into()),
        }
        previous = prefix;
    }
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
