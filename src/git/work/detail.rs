use super::*;

impl Runner {
    pub(super) fn compare(
        &mut self,
        target: diff::Target,
        admission: &mut Lease,
    ) -> Result<diff::Prepared, String> {
        self.check_target(&target, admission)?;
        let args = self.diff_args(&target);
        let (bytes, limited) = self.command_os(
            &args,
            Phase::Executing,
            target.group == Group::Untracked,
            Some(self.policy.max_detail_bytes),
        )?;
        if limited {
            return Ok(diff::Prepared {
                target,
                bytes,
                verified: false,
                completeness: Completeness::Truncated {
                    reason: "Git detail/output byte cap",
                },
            });
        }
        // A second exact comparison checks working-tree bytes as well as index
        // objects. Porcelain flags alone cannot detect edits with the same XY.
        let (verified, limited) = match self.command_os(
            &args,
            Phase::Verifying,
            target.group == Group::Untracked,
            Some(self.policy.max_detail_bytes),
        ) {
            Ok(result) => result,
            Err(_) if self.end == capture::End::OutputLimit => {
                return Ok(diff::Prepared {
                    target,
                    bytes,
                    verified: false,
                    completeness: Completeness::Truncated {
                        reason: "Git verification output cap",
                    },
                });
            }
            Err(error) => return Err(error),
        };
        if limited {
            return Ok(diff::Prepared {
                target,
                bytes,
                verified: false,
                completeness: Completeness::Truncated {
                    reason: "Git verification output cap",
                },
            });
        }
        if verified != bytes {
            return Err(
                "Git comparison changed while detail was being observed; refresh required".into(),
            );
        }
        drop(verified);
        self.check_target(&target, admission)?;
        Ok(diff::Prepared {
            target,
            bytes,
            verified: true,
            completeness: Completeness::Complete,
        })
    }

    fn check_target(&mut self, target: &diff::Target, admission: &mut Lease) -> Result<(), String> {
        let (identity, status) = self.observe(admission)?;
        let prior = target
            .observation
            .status
            .as_ref()
            .ok_or("Captured Git Status is unavailable")?;
        let current = status
            .as_ref()
            .ok_or("Working-tree Status became unavailable")?;
        if identity != target.observation.identity
            || current.head != prior.head
            || current
                .group(target.group)
                .find(|change| change.path == target.change.path)
                != Some(&target.change)
        {
            return Err(
                "Git path, layer or repository changed since Status; refresh required".into(),
            );
        }
        Ok(())
    }

    fn diff_args(&self, target: &diff::Target) -> Vec<OsString> {
        let mut args: Vec<OsString> = [
            "--literal-pathspecs",
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-color",
            "--full-index",
            "--no-relative",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "--unified=3",
            "--diff-algorithm=myers",
            "--no-indent-heuristic",
            "--submodule=short",
            "--ignore-submodules=none",
            "--find-renames=50%",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        if target.group == Group::Staged {
            args.push("--cached".into());
        }
        if target.group == Group::Untracked {
            args.push("--no-index".into());
            args.push("--".into());
            args.push("/dev/null".into());
        } else {
            args.push("--".into());
            if let Some(old) = &target.change.old_path
                && target.change.renamed_in(target.group)
            {
                args.push(old.as_os_str().to_owned());
            }
        }
        args.push(target.change.path.as_os_str().to_owned());
        args
    }
}
