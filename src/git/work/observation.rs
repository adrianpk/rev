use super::*;

impl Runner {
    pub(super) fn observe(
        &mut self,
        admission: &mut Lease,
    ) -> Result<(Identity, Option<Status>), String> {
        self.capture_directory()?;
        let identity = self.observe_identity()?;
        let status = if identity.worktree.is_none() {
            None
        } else {
            Some(self.observe_status(&identity, admission)?)
        };
        Ok((identity, status))
    }
    fn capture_directory(&mut self) -> Result<(), String> {
        let origin = std::fs::canonicalize(&self.directory)
            .map_err(|e| format!("Resolve Git directory: {e}"))?;
        if !origin.is_dir() {
            return Err("Git entry requires a captured directory".into());
        }
        self.directory = origin;
        Ok(())
    }
    fn observe_identity(&mut self) -> Result<Identity, String> {
        let git_directory = self.directory(&["rev-parse", "--absolute-git-dir"])?;
        let common_directory =
            self.directory(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
        let bare = self.observe_bare()?;
        let format = self.observe_format()?;
        let worktree = if bare {
            None
        } else {
            Some(self.directory(&["rev-parse", "--show-toplevel"])?)
        };
        let identity = Identity {
            worktree,
            git_directory,
            common_directory,
            format,
        };
        if identity
            .worktree
            .as_ref()
            .is_some_and(|root| !self.directory.starts_with(root))
        {
            return Err("Git resolved a different working tree from the captured directory".into());
        }
        Ok(identity)
    }
    fn observe_bare(&mut self) -> Result<bool, String> {
        let bare = self.command(
            &["rev-parse", "--is-bare-repository"],
            Phase::Checking,
            false,
        )?;
        match bare.as_slice() {
            b"true\n" => Ok(true),
            b"false\n" => Ok(false),
            _ => Err("Git lacks a machine-readable bare repository capability".into()),
        }
    }
    fn observe_format(&mut self) -> Result<ObjectFormat, String> {
        let format = self.command(
            &["rev-parse", "--show-object-format"],
            Phase::Checking,
            false,
        )?;
        match format.as_slice() {
            b"sha1\n" => Ok(ObjectFormat::Sha1),
            b"sha256\n" => Ok(ObjectFormat::Sha256),
            _ => Err("Git object format is unavailable or unsupported".into()),
        }
    }
    fn observe_status(
        &mut self,
        identity: &Identity,
        admission: &mut Lease,
    ) -> Result<Status, String> {
        let bytes = self.command(
            &[
                "status",
                "--porcelain=v2",
                "--branch",
                "--ahead-behind",
                "--untracked-files=all",
                "-z",
            ],
            Phase::Refreshing,
            false,
        )?;
        let records = bytes
            .iter()
            .filter(|b| **b == 0)
            .count()
            .min(self.policy.max_status_paths);
        admission.grow(
            records * (2 * std::mem::size_of::<PathChange>() + 256)
                + 2 * bytes.len().min(self.policy.max_path_bytes),
        )?;
        let status = status::parse(
            &bytes,
            identity,
            &self.policy,
            &self.cancel.bounded(self.deadline),
        )?;
        drop(bytes);
        self.verify_head(&status)?;
        Ok(status)
    }
    pub(super) fn verify_head(&mut self, status: &Status) -> Result<(), String> {
        let reference =
            self.command(&["symbolic-ref", "--quiet", "HEAD"], Phase::Verifying, true)?;
        let reference = reference.strip_suffix(b"\n").unwrap_or(&reference);
        let object = self.command(
            &["rev-parse", "--verify", "--quiet", "HEAD"],
            Phase::Verifying,
            true,
        )?;
        let object = object.strip_suffix(b"\n").unwrap_or(&object);
        if !head_reference_matches(&status.head, reference) {
            return Err("Git HEAD changed during Status observation; refresh required".into());
        }
        match &status.head {
            Head::Unborn { .. } if object.is_empty() => Ok(()),
            Head::Branch {
                object: expected, ..
            }
            | Head::Detached { object: expected }
                if object == expected.hex().as_bytes() =>
            {
                Ok(())
            }
            _ => Err("Git HEAD changed during Status observation; refresh required".into()),
        }
    }

    pub(super) fn directory(&mut self, args: &[&str]) -> Result<PathBuf, String> {
        let bytes = self.command(args, Phase::Checking, false)?;
        let bytes = bytes
            .strip_suffix(b"\n")
            .ok_or("Git directory response lacks a final newline")?;
        if bytes.is_empty() || bytes.len() > 4096 || bytes.contains(&0) {
            return Err("Git directory exceeds its exact 4096-byte path bound".into());
        }
        let path = PathBuf::from(OsString::from_vec(bytes.to_vec()));
        if !path.is_absolute() {
            return Err("Git directory response is not absolute".into());
        }
        let path =
            std::fs::canonicalize(path).map_err(|e| format!("Validate Git directory: {e}"))?;
        if !path.is_dir() || path.as_os_str().as_encoded_bytes().len() > 4096 {
            return Err("Git returned an unavailable or overlong directory".into());
        }
        Ok(path)
    }
}

fn head_reference_matches(head: &Head, reference: &[u8]) -> bool {
    match head {
        Head::Branch { name, .. } | Head::Unborn { branch: name } => {
            reference.strip_prefix(b"refs/heads/") == Some(name.as_slice())
        }
        Head::Detached { .. } => reference.is_empty(),
    }
}
