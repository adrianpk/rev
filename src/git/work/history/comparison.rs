use super::*;

/// One immutable comparison target retains its history and endpoint authority.
pub(super) struct Target {
    pub history: Arc<History>,
    pub object: ObjectId,
    pub parent: Option<ObjectId>,
    pub range: bool,
}
struct ChangedPaths {
    paths: Vec<RevisionPath>,
    path_bytes: usize,
    completeness: Completeness,
}
impl Target {
    pub(super) fn read(
        self,
        runner: &mut Runner,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let Self {
            history,
            object,
            parent,
            range,
        } = self;
        let metadata = runner.command(
            &["--no-replace-objects", "cat-file", "commit", &object.hex()],
            Phase::Checking,
            false,
        )?;
        if metadata.len() > runner.policy.max_detail_bytes {
            return Err("Revision metadata exceeds configured detail bound".into());
        }
        let mut args: Vec<OsString> = [
            "--no-replace-objects",
            "diff-tree",
            "--root",
            "--no-commit-id",
            "-r",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--ignore-submodules=none",
            "--find-renames",
            "--name-status",
            "-z",
        ]
        .iter()
        .map(OsString::from)
        .collect();
        if let Some(parent) = parent {
            args.push(parent.hex().into());
        }
        args.extend([object.hex().into(), "--".into()]);
        let (names, truncated) = runner.command_os(
            &args,
            Phase::Refreshing,
            false,
            Some(runner.policy.max_detail_bytes),
        )?;
        admission.grow(
            names.len() * 3
                + runner.policy.max_status_paths.min(names.len())
                    * std::mem::size_of::<RevisionPath>()
                    * 2
                + 4096,
        )?;
        let mut changed = ChangedPaths::read(&names, truncated, &runner.policy)?;
        let position = args
            .iter()
            .position(|v| v == "--name-status")
            .expect("revision diff mode");
        args[position] = "--patch".into();
        args[position + 1] = "--full-index".into();
        let (patch, truncated) = runner.command_os(
            &args,
            Phase::Refreshing,
            false,
            Some(
                runner
                    .policy
                    .max_detail_bytes
                    .saturating_sub(metadata.len())
                    .max(1),
            ),
        )?;
        if truncated {
            changed.completeness = Completeness::Truncated {
                reason: "Revision diff bytes",
            };
        }
        let chunks = patch
            .split_inclusive(|b| *b == b'\n')
            .map(|line| line.len().div_ceil(2048))
            .sum::<usize>();
        let bytes = std::mem::size_of::<RevisionDetail>()
            + metadata.capacity()
            + patch.capacity()
            + changed.paths.capacity() * std::mem::size_of::<RevisionPath>()
            + changed
                .paths
                .iter()
                .map(|p| {
                    p.path.capacity()
                        + p.old_path.as_ref().map_or(0, PathBuf::capacity)
                        + p.status.capacity()
                })
                .sum::<usize>()
            + chunks * std::mem::size_of::<std::ops::Range<usize>>()
            + 4096;
        admission.grow(bytes)?;
        let lines = patch_lines(&patch, chunks);
        Ok(ResultData::Show(RevisionDetail {
            id: runner
                .progress
                .lock()
                .expect("Git progress owner")
                .operation,
            history,
            object,
            parent,
            range,
            metadata,
            paths: changed.paths,
            patch,
            completeness: changed.completeness,
            lines,
            _admission: admission.split(bytes),
        }))
    }
}
impl ChangedPaths {
    fn read(names: &[u8], truncated: bool, policy: &config::Git) -> Result<Self, String> {
        let mut selection = Self {
            paths: Vec::new(),
            path_bytes: 0,
            completeness: if truncated {
                Completeness::Truncated {
                    reason: "Revision changed-path output limit",
                }
            } else {
                Completeness::Complete
            },
        };
        let mut fields = names
            .split_inclusive(|b| *b == 0)
            .map(|v| v.strip_suffix(b"\0").unwrap_or(v));
        if !truncated {
            while let Some(status) = fields.next() {
                if !selection.record(status, &mut fields, policy)? {
                    break;
                }
            }
        }
        Ok(selection)
    }
    fn record<'a>(
        &mut self,
        status: &[u8],
        fields: &mut impl Iterator<Item = &'a [u8]>,
        policy: &config::Git,
    ) -> Result<bool, String> {
        if self.paths.len() >= policy.max_status_paths {
            self.completeness = Completeness::Truncated {
                reason: "Revision path count",
            };
            return Ok(false);
        }
        let first = fields.next().ok_or("Incomplete revision path record")?;
        let rename = matches!(status.first(), Some(b'R' | b'C'));
        let second = if rename {
            fields.next().ok_or("Incomplete historical rename")?
        } else {
            first
        };
        self.path_bytes += first.len() + if rename { second.len() } else { 0 };
        if self.path_bytes > policy.max_path_bytes {
            self.completeness = Completeness::Truncated {
                reason: "Revision path bytes",
            };
            return Ok(false);
        }
        validate_path(first)?;
        validate_path(second)?;
        self.paths.push(RevisionPath {
            path: PathBuf::from(OsString::from_vec(second.to_vec())),
            old_path: rename.then(|| PathBuf::from(OsString::from_vec(first.to_vec()))),
            status: status.to_vec(),
        });
        Ok(true)
    }
}
fn patch_lines(patch: &[u8], capacity: usize) -> Vec<std::ops::Range<usize>> {
    let mut lines = Vec::with_capacity(capacity);
    let mut offset = 0;
    for line in patch.split_inclusive(|b| *b == b'\n') {
        for start in (0..line.len()).step_by(2048) {
            lines.push(offset + start..offset + (start + 2048).min(line.len()));
        }
        offset += line.len();
    }
    lines
}
