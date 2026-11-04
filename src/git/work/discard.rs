use super::*;
use std::{
    fs,
    io::Read,
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
};

impl Runner {
    pub(super) fn discard(
        &mut self,
        diff: Arc<Diff>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let targets = [IndexTarget::Path(diff.clone())];
        self.check_index_comparisons(&targets, admission)?;
        let before = self.discard_index()?;
        admission.grow(2 * before.len() + 16384)?;
        let object = diff
            .change
            .objects
            .get(1)
            .ok_or("Missing checked index object")?;
        let mode = *diff
            .change
            .modes
            .get(1)
            .ok_or("Missing checked index mode")?;
        let mut args: Vec<OsString> = vec!["cat-file".into()];
        if mode == 0o120000 {
            args.push("blob".into());
        } else {
            args.push("--filters".into());
            let mut path = OsString::from("--path=");
            path.push(&diff.change.path);
            args.push(path);
        }
        args.push(object.hex().into());
        let (expected, limited) = self.command_os(
            &args,
            Phase::Checking,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        if limited {
            return Err("Discard checkout content exceeded complete-detail admission".into());
        }
        admission.grow(2 * expected.len() + 4096)?;
        self.check_index_comparisons(&targets, admission)?;
        if self.discard_index()? != before {
            return Err("Index changed during discard preparation; refresh required".into());
        }
        let args = [
            "--literal-pathspecs".into(),
            "restore".into(),
            "--worktree".into(),
            "--".into(),
            diff.change.path.as_os_str().to_owned(),
        ];
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err(
                "Discard child cleanup remains owned; effects require later inspection".into(),
            );
        }
        // Both proofs are attempted after a failed child; no mutation is replayed.
        let result = (|| {
            let (identity, status) = self.observe(admission)?;
            let index = self.discard_index()?;
            let verified = if identity != diff.observation.identity
                || status.as_ref().map(|s| &s.head)
                    != diff.observation.status.as_ref().map(|s| &s.head)
                || index != before
            {
                Err("Discard index/repository postcondition mismatch; inspect actual state".into())
            } else {
                disk_matches(&self.directory.join(&diff.change.path), mode, &expected)
            };
            Ok((Candidate::Status(identity, status), verified))
        })();
        self.restore_execution(facts, executed.is_err());
        match result {
            Ok((candidate, verified)) => {
                match (executed, verified) {
                    (Ok(_), Ok(())) => self.effects = Effects::Verified,
                    (Err(error), _) => {
                        self.operation_failure = Some(format!(
                            "Discard child failed after possible effects: {error}"
                        ))
                    }
                    (_, Err(error)) => self.operation_failure = Some(error),
                }
                Ok(candidate)
            }
            Err(error) => {
                self.operation_failure = Some(format!(
                    "Discard effects remain unverified; refresh required: {error}"
                ));
                Err(error)
            }
        }
    }
    fn discard_index(&mut self) -> Result<Vec<u8>, String> {
        let (bytes, limited) = self.command_os(
            &["ls-files".into(), "--stage".into(), "-z".into()],
            Phase::Verifying,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        if limited {
            return Err("Complete index entries exceed discard admission".into());
        }
        Ok(bytes)
    }
}

fn disk_matches(path: &Path, mode: u32, expected: &[u8]) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|e| format!("Discard file verification: {e}"))?;
    if mode == 0o120000 {
        if !metadata.file_type().is_symlink()
            || fs::read_link(path)
                .map_err(|e| e.to_string())?
                .as_os_str()
                .as_bytes()
                != expected
        {
            return Err("Discard symlink content/type mismatch".into());
        }
        let after = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        return if same_metadata(&metadata, &after) {
            Ok(())
        } else {
            Err("Discard symlink identity changed during verification".into())
        };
    }
    if !metadata.is_file()
        || metadata.len() != expected.len() as u64
        || (metadata.mode() & 0o111 != 0) != (mode == 0o100755)
    {
        return Err("Discard file type, size or executable-mode mismatch".into());
    }
    let descriptor = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| e.to_string())?;
    let file = fs::File::from(descriptor);
    let opened = file.metadata().map_err(|e| e.to_string())?;
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return Err("Discard destination changed during file verification".into());
    }
    let mut actual = Vec::with_capacity(expected.len() + 1);
    file.take(expected.len() as u64 + 1)
        .read_to_end(&mut actual)
        .map_err(|e| e.to_string())?;
    let after = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if actual != expected || !same_metadata(&metadata, &after) {
        return Err("Discard file bytes or identity changed during verification".into());
    }
    Ok(())
}

fn same_metadata(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mode() == b.mode()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}
