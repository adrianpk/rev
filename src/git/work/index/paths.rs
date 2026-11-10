use super::*;

impl Runner {
    pub(super) fn path_group(
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
        let args = group_arguments(direction, targets[0].diff())?;
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
        let mode = self.worktree_mode(diff, &metadata)?;
        let bytes = self.worktree_object(path, &absolute, &metadata, admission)?;
        Ok(Entry {
            path: path.to_owned(),
            mode,
            object: Some(ObjectId::parse(
                bytes.strip_suffix(b"\n").unwrap_or(&bytes),
                diff.observation.identity.format,
            )?),
        })
    }

    pub(super) fn hash_input(
        &mut self,
        bytes: &[u8],
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        admission.grow(bytes.len())?;
        let input = crate::process::input::stage(Some(bytes))?;
        self.command_argv(
            self.git_invocation(&["hash-object".into(), "--stdin".into()], input),
            Phase::Checking,
            false,
            None,
        )
        .map(|(bytes, _)| bytes)
    }

    fn worktree_mode(
        &mut self,
        diff: &Diff,
        metadata: &fs::Metadata,
    ) -> Result<Option<u32>, String> {
        let mode = if diff.group == Group::Untracked {
            Some(self.untracked_mode(metadata)?)
        } else {
            diff.change.modes.get(2).copied()
        };
        Ok(mode)
    }

    fn untracked_mode(&mut self, metadata: &fs::Metadata) -> Result<u32, String> {
        Ok(if metadata.is_dir() {
            0o160000
        } else if metadata.file_type().is_symlink() {
            0o120000
        } else {
            let executable = self.executable_mode(metadata)?;
            if executable { 0o100755 } else { 0o100644 }
        })
    }

    fn executable_mode(&mut self, metadata: &fs::Metadata) -> Result<bool, String> {
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
        Ok(executable)
    }

    fn worktree_object(
        &mut self,
        path: &Path,
        absolute: &Path,
        metadata: &fs::Metadata,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let bytes = if metadata.is_dir() {
            self.gitlink_object(path)?
        } else if metadata.file_type().is_symlink() {
            let target = fs::read_link(absolute).map_err(|e| format!("Read index symlink: {e}"))?;
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
        Ok(bytes)
    }

    fn gitlink_object(&mut self, path: &Path) -> Result<Vec<u8>, String> {
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
        Ok(self.command_os(&args, Phase::Checking, false, None)?.0)
    }
}

fn group_arguments(direction: IndexDirection, diff: &Diff) -> Result<Vec<OsString>, String> {
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
    Ok(args)
}
