use super::*;
use crate::git::integration::SequenceFile;

impl Runner {
    fn sequence_files(
        &mut self,
        refs: &Refs,
        admission: &mut Lease,
    ) -> Result<Vec<SequenceFile>, String> {
        let root = &refs.observation.identity.git_directory;
        let mut paths = Vec::new();
        let mut directories = Vec::new();
        for name in [
            "BISECT_START",
            "BISECT_LOG",
            "BISECT_TERMS",
            "BISECT_NAMES",
            "BISECT_EXPECTED_REV",
            "BISECT_ANCESTORS_OK",
            "BISECT_HEAD",
            "MERGE_HEAD",
            "MERGE_AUTOSTASH",
            "MERGE_MSG",
            "MERGE_MODE",
            "ORIG_HEAD",
            "REBASE_HEAD",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
        ] {
            match fs::symlink_metadata(root.join(name)) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("Observe Git sequence marker: {e}")),
                Ok(_) => paths.push(PathBuf::from(name)),
            }
        }
        for directory in ["rebase-merge", "rebase-apply", "sequencer"] {
            let path = root.join(directory);
            match fs::symlink_metadata(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(format!("Observe Git sequence directory: {e}")),
                Ok(m) if !m.is_dir() || m.file_type().is_symlink() => {
                    return Err("Git sequence directory is not an ordinary owned directory".into());
                }
                _ => {}
            }
            directories.push(format!("{directory}/").into_bytes());
            for entry in fs::read_dir(path).map_err(|e| format!("Read sequence entries: {e}"))? {
                if paths.len() == 64 {
                    return Err("Sequence metadata exceeds 64-file admission".into());
                }
                paths.push(
                    PathBuf::from(directory).join(
                        entry
                            .map_err(|e| format!("Read sequence entry: {e}"))?
                            .file_name(),
                    ),
                );
            }
        }
        paths.sort();
        admission.grow(directories.iter().map(|d| d.capacity() + 128).sum())?;
        let mut files = Vec::with_capacity(paths.len() + directories.len());
        files.extend(directories.into_iter().map(|name| SequenceFile {
            name,
            bytes: Vec::new(),
        }));
        let mut remaining = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        for path in paths {
            if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
                return Err("Sequence observation cancelled or timed out".into());
            }
            let full = root.join(&path);
            let metadata =
                fs::symlink_metadata(&full).map_err(|e| format!("Read sequence metadata: {e}"))?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err("Sequence metadata is not an ordinary file".into());
            }
            let length = usize::try_from(metadata.len())
                .map_err(|_| "Sequence file length exceeds admission")?;
            if length > remaining {
                return Err("Sequence metadata exceeds bounded detail/scratch admission".into());
            }
            admission.grow(length + path.as_os_str().len() + 128)?;
            let mut bytes = Vec::with_capacity(length);
            fs::File::open(full)
                .map_err(|e| format!("Open sequence metadata: {e}"))?
                .take(remaining as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("Read sequence metadata bytes: {e}"))?;
            if bytes.len() > remaining {
                return Err("Sequence metadata grew beyond detail/scratch admission".into());
            }
            remaining -= bytes.len();
            files.push(SequenceFile {
                name: path.as_os_str().as_bytes().into(),
                bytes,
            });
        }
        if files.iter().any(|f| f.name == b"BISECT_START") {
            let refs = self.command(
                &[
                    "for-each-ref",
                    "--format=%(refname) %(objectname) %(objecttype) %(symref)",
                    "refs/bisect/",
                ],
                Phase::Checking,
                false,
            )?;
            if refs.len() > remaining {
                return Err("Bisect refs exceed bounded metadata admission".into());
            }
            admission.grow(refs.len() + 128)?;
            files.push(SequenceFile {
                name: b"refs/bisect/".to_vec(),
                bytes: refs,
            });
        }
        Ok(files)
    }
    pub(in crate::git::work) fn read_sequence(
        &mut self,
        refs: Arc<Refs>,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        let files = self.sequence_files(&refs, admission)?;
        let value = |name: &[u8]| {
            files
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.bytes.as_slice())
        };
        let has = |prefix: &[u8]| files.iter().any(|f| f.name.starts_with(prefix));
        let family = if (value(b"BISECT_START").is_some()
            && (has(b"rebase-")
                || has(b"sequencer/")
                || value(b"MERGE_HEAD").is_some()
                || value(b"CHERRY_PICK_HEAD").is_some()
                || value(b"REVERT_HEAD").is_some()))
            || (has(b"rebase-merge/") && has(b"rebase-apply/"))
            || (value(b"CHERRY_PICK_HEAD").is_some() && value(b"REVERT_HEAD").is_some())
            || (value(b"MERGE_HEAD").is_some()
                && (value(b"CHERRY_PICK_HEAD").is_some() || value(b"REVERT_HEAD").is_some()))
        {
            Some(SequenceFamily::Unsupported)
        } else if value(b"BISECT_START").is_some() {
            Some(SequenceFamily::Bisect)
        } else if has(b"rebase-merge/") {
            Some(SequenceFamily::Rebase)
        } else if has(b"rebase-apply/") {
            Some(if value(b"rebase-apply/applying").is_some() {
                SequenceFamily::Mailbox
            } else {
                SequenceFamily::Rebase
            })
        } else if value(b"MERGE_HEAD").is_some() {
            Some(SequenceFamily::Merge)
        } else if value(b"CHERRY_PICK_HEAD").is_some() {
            Some(SequenceFamily::CherryPick)
        } else if value(b"REVERT_HEAD").is_some() {
            Some(SequenceFamily::Revert)
        } else if has(b"sequencer/") {
            Some(SequenceFamily::Unsupported)
        } else {
            None
        };
        let prefix = if has(b"rebase-merge/") {
            b"rebase-merge/".as_slice()
        } else {
            b"rebase-apply/"
        };
        let field = |name: &[u8]| value(&[prefix, name].concat());
        let format = refs.observation.identity.format;
        let object = |bytes: Option<&[u8]>| -> Result<Option<ObjectId>, String> {
            bytes
                .map(|v| ObjectId::parse(v.strip_suffix(b"\n").unwrap_or(v), format))
                .transpose()
        };
        let mut targets = Vec::new();
        if family == Some(SequenceFamily::Merge)
            && let Some(bytes) = value(b"MERGE_HEAD")
        {
            let count = bytes
                .split(|b| *b == b'\n')
                .filter(|line| !line.is_empty())
                .count();
            if count > 256 {
                return Err("Merge sequence exceeds 256-target admission".into());
            }
            admission.grow(count * std::mem::size_of::<ObjectId>())?;
            for line in bytes.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
                targets.push(ObjectId::parse(line, format)?);
            }
        }

        let original = if family == Some(SequenceFamily::Rebase) {
            object(field(b"orig-head"))?
        } else if family == Some(SequenceFamily::Merge) {
            object(value(b"ORIG_HEAD"))?
        } else {
            None
        };
        let current = if family == Some(SequenceFamily::Bisect) {
            refs.head
        } else if family == Some(SequenceFamily::Rebase) {
            object(
                field(b"stopped-sha")
                    .or_else(|| value(b"REBASE_HEAD"))
                    .or_else(|| field(b"original-commit")),
            )?
        } else if family == Some(SequenceFamily::CherryPick) {
            object(value(b"CHERRY_PICK_HEAD"))?
        } else if family == Some(SequenceFamily::Revert) {
            object(value(b"REVERT_HEAD"))?
        } else if targets.len() == 1 {
            targets.first().copied()
        } else {
            None
        };
        let onto = if family == Some(SequenceFamily::Rebase) {
            object(field(b"onto"))?
        } else {
            None
        };
        let branch = field(b"head-name")
            .unwrap_or(&[])
            .strip_suffix(b"\n")
            .unwrap_or(field(b"head-name").unwrap_or(&[]));
        if branch.len() > 4096 {
            return Err("Sequence branch metadata exceeds 4096 bytes".into());
        }
        let number = |bytes: Option<&[u8]>| -> Result<Option<usize>, String> {
            bytes
                .map(|v| {
                    std::str::from_utf8(v)
                        .ok()
                        .and_then(|s| s.trim().parse().ok())
                        .ok_or("Sequence step is malformed".into())
                })
                .transpose()
        };
        let mut sequence = Sequence {
            id: self.progress.lock().expect("Git progress owner").operation,
            refs: refs.clone(),
            family,
            current,
            targets,
            original,
            onto,
            branch: branch.into(),
            step: number(field(b"msgnum").or_else(|| field(b"next")))?,
            total: number(field(b"end").or_else(|| field(b"last")))?,
            conflicts: Vec::new(),
            application: None,
            bisect: None,
            files: Vec::new(),
            index: Vec::new(),
            worktree: Vec::new(),
            _admission: admission.split(0),
        };
        if let Some(status) = &refs.observation.status {
            admission.grow(
                status.paths.len() * std::mem::size_of::<PathBuf>()
                    + status
                        .group(Group::Unmerged)
                        .map(|p| p.path.capacity())
                        .sum::<usize>(),
            )?;
            sequence.conflicts = status
                .group(Group::Unmerged)
                .map(|p| p.path.clone())
                .collect();
        }
        if family.is_some() || !sequence.conflicts.is_empty() {
            sequence.index =
                self.command(&["ls-files", "--stage", "-z"], Phase::Checking, false)?;
            sequence.worktree = self.command(
                &[
                    "diff",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "--binary",
                    "--",
                ],
                Phase::Checking,
                false,
            )?;
            admission.grow(sequence.index.len() + sequence.worktree.len())?;
        }
        if files != self.sequence_files(&refs, admission)?
            || self.ref_head(format)? != (refs.head_ref.clone(), refs.head)
        {
            return Err("Git sequence or HEAD changed during observation".into());
        }
        sequence.files = files;
        if family == Some(SequenceFamily::Bisect) {
            sequence.bisect = self.read_bisect(&sequence, admission)?.map(Box::new);
            if sequence.files != self.sequence_files(&refs, admission)? {
                return Err("Bisect changed during candidate verification".into());
            }
        }
        admission.grow(sequence.bytes())?;
        sequence._admission = admission.split(sequence.bytes());
        Ok(sequence)
    }
}
