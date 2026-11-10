use super::*;
use crate::git::integration::SequenceFile;

impl Runner {
    fn sequence_files(
        &mut self,
        refs: &Refs,
        admission: &mut Lease,
    ) -> Result<Vec<SequenceFile>, String> {
        let root = &refs.observation.identity.git_directory;
        let (mut paths, directories) = sequence_paths(root)?;
        paths.sort();
        admission.grow(directories.iter().map(|d| d.capacity() + 128).sum())?;
        let mut files = Vec::with_capacity(paths.len() + directories.len());
        files.extend(directories.into_iter().map(|name| SequenceFile {
            name,
            bytes: Vec::new(),
        }));
        let mut remaining = self.policy.max_detail_bytes.min(budget::SCRATCH_BYTES);
        for path in paths {
            let bytes = self.sequence_file(root, &path, remaining, admission)?;
            remaining -= bytes.len();
            files.push(SequenceFile {
                name: path.as_os_str().as_bytes().into(),
                bytes,
            });
        }
        if files.iter().any(|f| f.name == b"BISECT_START") {
            self.append_bisect_refs(&mut files, remaining, admission)?;
        }
        Ok(files)
    }
    pub(in crate::git::work) fn read_sequence(
        &mut self,
        refs: &Arc<Refs>,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        let files = self.sequence_files(refs, admission)?;
        let metadata = Metadata::new(&files, refs.observation.identity.format);
        let mut sequence = metadata.decode(self, refs, admission)?;
        capture_conflicts(&mut sequence, admission)?;
        self.capture_sequence_changes(&mut sequence, admission)?;
        self.check_sequence_capture(&mut sequence, files, admission)?;
        admission.grow(sequence.bytes())?;
        sequence._admission = admission.split(sequence.bytes());
        Ok(sequence)
    }
    fn sequence_file(
        &mut self,
        root: &Path,
        path: &Path,
        remaining: usize,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        if self.cancel.is_cancelled() || Instant::now() >= self.deadline {
            return Err("Sequence observation cancelled or timed out".into());
        }
        let length = sequence_file_length(root, path)?;
        if length > remaining {
            return Err("Sequence metadata exceeds bounded detail/scratch admission".into());
        }
        admission.grow(length + path.as_os_str().len() + 128)?;
        read_sequence_bytes(&root.join(path), length, remaining)
    }
    fn append_bisect_refs(
        &mut self,
        files: &mut Vec<SequenceFile>,
        remaining: usize,
        admission: &mut Lease,
    ) -> Result<(), String> {
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
        Ok(())
    }
    fn capture_sequence_changes(
        &mut self,
        sequence: &mut Sequence,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if sequence.family.is_some() || !sequence.conflicts.is_empty() {
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
        Ok(())
    }
    fn check_sequence_capture(
        &mut self,
        sequence: &mut Sequence,
        files: Vec<SequenceFile>,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if files != self.sequence_files(&sequence.refs, admission)?
            || self.ref_head(sequence.refs.observation.identity.format)?
                != (sequence.refs.head_ref.clone(), sequence.refs.head)
        {
            return Err("Git sequence or HEAD changed during observation".into());
        }
        sequence.files = files;
        if sequence.family == Some(SequenceFamily::Bisect) {
            sequence.bisect = self.read_bisect(sequence, admission)?.map(Box::new);
            if sequence.files != self.sequence_files(&sequence.refs, admission)? {
                return Err("Bisect changed during candidate verification".into());
            }
        }
        Ok(())
    }
}

fn sequence_paths(root: &Path) -> Result<(Vec<PathBuf>, Vec<Vec<u8>>), String> {
    let mut paths = Vec::new();
    let mut directories = Vec::new();
    sequence_markers(root, &mut paths)?;
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
        sequence_directory_entries(&path, directory, &mut paths)?;
    }
    Ok((paths, directories))
}
fn sequence_markers(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
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
    Ok(())
}
fn sequence_directory_entries(
    path: &Path,
    directory: &str,
    paths: &mut Vec<PathBuf>,
) -> Result<(), String> {
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
    Ok(())
}
fn sequence_file_length(root: &Path, path: &Path) -> Result<usize, String> {
    let full = root.join(path);
    let metadata =
        fs::symlink_metadata(&full).map_err(|e| format!("Read sequence metadata: {e}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Sequence metadata is not an ordinary file".into());
    }
    let length =
        usize::try_from(metadata.len()).map_err(|_| "Sequence file length exceeds admission")?;
    Ok(length)
}
fn read_sequence_bytes(path: &Path, length: usize, remaining: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(length);
    fs::File::open(path)
        .map_err(|e| format!("Open sequence metadata: {e}"))?
        .take(remaining as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Read sequence metadata bytes: {e}"))?;
    if bytes.len() > remaining {
        return Err("Sequence metadata grew beyond detail/scratch admission".into());
    }
    Ok(bytes)
}
fn capture_conflicts(sequence: &mut Sequence, admission: &mut Lease) -> Result<(), String> {
    if let Some(status) = &sequence.refs.observation.status {
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
    Ok(())
}

/// Borrowed complete sequence snapshot; decoding never performs an additional read.
struct Metadata<'a> {
    files: &'a [SequenceFile],
    family: Option<SequenceFamily>,
    prefix: &'static [u8],
    format: ObjectFormat,
}
impl<'a> Metadata<'a> {
    fn new(files: &'a [SequenceFile], format: ObjectFormat) -> Self {
        let mut metadata = Self {
            files,
            family: None,
            prefix: b"rebase-apply/",
            format,
        };
        metadata.family = metadata.classify();
        if metadata.has(b"rebase-merge/") {
            metadata.prefix = b"rebase-merge/";
        }
        metadata
    }
    fn decode(
        &self,
        runner: &Runner,
        refs: &Arc<Refs>,
        admission: &mut Lease,
    ) -> Result<Sequence, String> {
        let family = self.family;
        let targets = self.targets(admission)?;
        let original = self.original()?;
        let current = self.current(refs, &targets)?;
        let onto = self.onto()?;
        let branch = self.branch()?;
        let sequence = Sequence {
            id: runner
                .progress
                .lock()
                .expect("Git progress owner")
                .operation,
            refs: std::sync::Arc::clone(refs),
            family,
            current,
            targets,
            original,
            onto,
            branch: branch.into(),
            step: sequence_number(self.field(b"msgnum").or_else(|| self.field(b"next")))?,
            total: sequence_number(self.field(b"end").or_else(|| self.field(b"last")))?,
            conflicts: Vec::new(),
            application: None,
            bisect: None,
            files: Vec::new(),
            index: Vec::new(),
            worktree: Vec::new(),
            _admission: admission.split(0),
        };
        Ok(sequence)
    }
    fn value(&self, name: &[u8]) -> Option<&'a [u8]> {
        self.files
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.bytes.as_slice())
    }
    fn has(&self, prefix: &[u8]) -> bool {
        self.files.iter().any(|f| f.name.starts_with(prefix))
    }
    fn field(&self, name: &[u8]) -> Option<&'a [u8]> {
        self.value(&[self.prefix, name].concat())
    }
    fn object(&self, bytes: Option<&[u8]>) -> Result<Option<ObjectId>, String> {
        bytes
            .map(|v| ObjectId::parse(v.strip_suffix(b"\n").unwrap_or(v), self.format))
            .transpose()
    }
    fn mixed_bisect(&self) -> bool {
        self.value(b"BISECT_START").is_some()
            && (self.has(b"rebase-")
                || self.has(b"sequencer/")
                || self.value(b"MERGE_HEAD").is_some()
                || self.value(b"CHERRY_PICK_HEAD").is_some()
                || self.value(b"REVERT_HEAD").is_some())
    }
    fn conflicting(&self) -> bool {
        self.mixed_bisect()
            || (self.has(b"rebase-merge/") && self.has(b"rebase-apply/"))
            || (self.value(b"CHERRY_PICK_HEAD").is_some() && self.value(b"REVERT_HEAD").is_some())
            || (self.value(b"MERGE_HEAD").is_some()
                && (self.value(b"CHERRY_PICK_HEAD").is_some()
                    || self.value(b"REVERT_HEAD").is_some()))
    }
    fn classify(&self) -> Option<SequenceFamily> {
        if self.conflicting() {
            return Some(SequenceFamily::Unsupported);
        }
        if self.value(b"BISECT_START").is_some() {
            return Some(SequenceFamily::Bisect);
        }
        if self.has(b"rebase-merge/") {
            return Some(SequenceFamily::Rebase);
        }
        if self.has(b"rebase-apply/") {
            return Some(if self.value(b"rebase-apply/applying").is_some() {
                SequenceFamily::Mailbox
            } else {
                SequenceFamily::Rebase
            });
        }
        self.marker_family()
    }
    fn marker_family(&self) -> Option<SequenceFamily> {
        if self.value(b"MERGE_HEAD").is_some() {
            return Some(SequenceFamily::Merge);
        }
        if self.value(b"CHERRY_PICK_HEAD").is_some() {
            return Some(SequenceFamily::CherryPick);
        }
        if self.value(b"REVERT_HEAD").is_some() {
            return Some(SequenceFamily::Revert);
        }
        if self.has(b"sequencer/") {
            return Some(SequenceFamily::Unsupported);
        }
        None
    }
    fn targets(&self, admission: &mut Lease) -> Result<Vec<ObjectId>, String> {
        let mut targets = Vec::new();
        if self.family == Some(SequenceFamily::Merge)
            && let Some(bytes) = self.value(b"MERGE_HEAD")
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
                targets.push(ObjectId::parse(line, self.format)?);
            }
        }

        Ok(targets)
    }
    fn original(&self) -> Result<Option<ObjectId>, String> {
        let original = if self.family == Some(SequenceFamily::Rebase) {
            self.object(self.field(b"orig-head"))?
        } else if matches!(
            self.family,
            Some(SequenceFamily::Merge | SequenceFamily::Mailbox)
        ) {
            self.object(self.value(b"ORIG_HEAD"))?
        } else {
            None
        };
        Ok(original)
    }
    fn current(&self, refs: &Refs, targets: &[ObjectId]) -> Result<Option<ObjectId>, String> {
        if self.family == Some(SequenceFamily::Bisect) {
            return Ok(refs.head);
        }
        if self.family == Some(SequenceFamily::Rebase) {
            return self.object(
                self.field(b"stopped-sha")
                    .or_else(|| self.value(b"REBASE_HEAD"))
                    .or_else(|| self.field(b"original-commit")),
            );
        }
        if self.family == Some(SequenceFamily::CherryPick) {
            return self.object(self.value(b"CHERRY_PICK_HEAD"));
        }
        if self.family == Some(SequenceFamily::Revert) {
            return self.object(self.value(b"REVERT_HEAD"));
        }
        Ok(if targets.len() == 1 {
            targets.first().copied()
        } else {
            None
        })
    }
    fn onto(&self) -> Result<Option<ObjectId>, String> {
        let onto = if self.family == Some(SequenceFamily::Rebase) {
            self.object(self.field(b"onto"))?
        } else {
            None
        };
        Ok(onto)
    }
    fn branch(&self) -> Result<&'a [u8], String> {
        let branch = self
            .field(b"head-name")
            .unwrap_or(&[])
            .strip_suffix(b"\n")
            .unwrap_or(self.field(b"head-name").unwrap_or(&[]));
        if branch.len() > 4096 {
            return Err("Sequence branch metadata exceeds 4096 bytes".into());
        }
        Ok(branch)
    }
}
fn sequence_number(bytes: Option<&[u8]>) -> Result<Option<usize>, String> {
    bytes
        .map(|v| {
            std::str::from_utf8(v)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .ok_or("Sequence step is malformed".into())
        })
        .transpose()
}
