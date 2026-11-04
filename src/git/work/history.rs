use super::*;
use crate::git::history::{self, Request, ResultData};

impl Runner {
    pub(super) fn history(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let observation = match &request {
            Request::Log { observation, .. } => observation,
            Request::Show { history, .. } => &history.observation,
        };
        let (identity, status) = self.observe(admission)?;
        if identity != observation.identity
            || status.as_ref().map(|s| &s.head) != observation.status.as_ref().map(|s| &s.head)
        {
            return Err(
                "Git HEAD or repository changed; refresh Status before inspecting History".into(),
            );
        }
        let prior = match &request {
            Request::Log { prior, .. } => prior.clone(),
            Request::Show { history, .. } => Some(history.clone()),
        };
        if let Some(history) = &prior {
            self.verify_ancestry(history)?;
        }
        let result = match request {
            Request::Log {
                observation,
                root,
                prior,
                next,
            } => self.ancestry(observation, root, prior, next, admission),
            Request::Show {
                history,
                object,
                parent,
            } => self.revision(history, object, parent, admission),
        }?;
        if let Some(history) = &prior {
            self.verify_ancestry(history)?;
        }
        if let Some(status) = &status {
            self.verify_head(status)?;
        }
        Ok(result)
    }

    fn verify_ancestry(&mut self, history: &History) -> Result<(), String> {
        let Some(root) = history.root else {
            return Ok(());
        };
        let count = history.revisions().count();
        let bytes = self.command(
            &[
                "--no-replace-objects",
                "rev-list",
                "--topo-order",
                &format!("--max-count={count}"),
                &root.hex(),
                "--",
            ],
            Phase::Verifying,
            false,
        )?;
        let mut objects = bytes.split(|b| *b == b'\n').filter(|s| !s.is_empty());
        for revision in history.revisions() {
            if objects.next() != Some(revision.object.hex().as_bytes()) {
                return Err(
                    "Git ancestry changed before its exact continuation; reopen History".into(),
                );
            }
        }
        if objects.next().is_some() {
            return Err("Git ancestry exceeds the captured continuation".into());
        }
        Ok(())
    }

    fn ancestry(
        &mut self,
        observation: Arc<Observation>,
        root: Option<String>,
        prior: Option<Arc<History>>,
        next: bool,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let root = if let Some(prior) = &prior {
            prior.root
        } else if let Some(root) = root {
            let resolved = self.command(
                &[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("{root}^{{commit}}"),
                ],
                Phase::Checking,
                false,
            )?;
            Some(ObjectId::parse(
                resolved
                    .strip_suffix(b"\n")
                    .ok_or("Malformed history root")?,
                observation.identity.format,
            )?)
        } else {
            match observation.status.as_ref().map(|s| &s.head) {
                Some(Head::Branch { object, .. } | Head::Detached { object }) => Some(*object),
                Some(Head::Unborn { .. }) => None,
                None => return Err("Bare history requires an explicit revision root".into()),
            }
        };
        let id = self.progress.lock().expect("Git progress owner").operation;
        if !next && let Some(prior) = prior {
            if let Some(root) = root {
                self.command(
                    &[
                        "--no-replace-objects",
                        "cat-file",
                        "-e",
                        &format!("{}^{{commit}}", root.hex()),
                    ],
                    Phase::Checking,
                    false,
                )?;
            }
            let bytes = std::mem::size_of::<History>()
                + (self.policy.max_history_pages + 1) * std::mem::size_of::<Arc<HistoryPage>>() * 2
                + 4096;
            admission.grow(bytes)?;
            let more = prior.more;
            return Ok(ResultData::Log(history::log_result(
                id,
                observation,
                root,
                Some(prior),
                history::PageResult {
                    revisions: Vec::new(),
                    more,
                },
                self.policy.max_history_pages,
                admission.split(bytes),
            )));
        }
        let retained = prior;
        let skip = retained.as_ref().map_or(0, |h| h.revisions().count());
        let bytes = if let Some(root) = root {
            self.command(
                &[
                    "--no-replace-objects",
                    "log",
                    "--topo-order",
                    "--no-decorate",
                    "--no-show-signature",
                    "--format=%H%x00%P%x00%an%x00%ae%x00%aI%x00%cn%x00%ce%x00%cI%x00%s",
                    "-z",
                    &format!("--max-count={}", self.policy.history_page_size + 1),
                    &format!("--skip={skip}"),
                    &root.hex(),
                    "--",
                ],
                Phase::Refreshing,
                false,
            )?
        } else {
            Vec::new()
        };
        if bytes.len() > budget::SCRATCH_BYTES {
            return Err("History page exceeds its 1 MiB parser scratch bound".into());
        }
        let records = bytes.iter().filter(|b| **b == 0).count() / 9;
        admission.grow(
            bytes.len() * 3 + (records + 1) * (std::mem::size_of::<Revision>() * 2 + 256) + 4096,
        )?;
        let mut fields = bytes
            .split_inclusive(|b| *b == 0)
            .map(|v| v.strip_suffix(b"\0").unwrap_or(v));
        let mut revisions = Vec::new();
        while let Some(object) = fields.next() {
            if self.cancel.bounded(self.deadline).is_cancelled() {
                return Err("History parsing cancelled".into());
            }
            let mut field = || fields.next().ok_or("Incomplete Git history record");
            let parents = field()?;
            if parents.len() > 4096 {
                return Err("Revision parent metadata exceeds 4096 bytes".into());
            }
            let parents = if parents.is_empty() {
                Vec::new()
            } else {
                parents
                    .split(|b| *b == b' ')
                    .map(|p| ObjectId::parse(p, observation.identity.format))
                    .collect::<Result<Vec<_>, _>>()?
            };
            let mut value = || -> Result<Vec<u8>, String> {
                let v = field()?;
                if v.len() > 4096 {
                    return Err("Revision metadata exceeds 4096 bytes".into());
                }
                Ok(v.to_vec())
            };
            revisions.push(Revision {
                object: ObjectId::parse(object, observation.identity.format)?,
                parents,
                author: value()?,
                author_email: value()?,
                authored: value()?,
                committer: value()?,
                committer_email: value()?,
                committed: value()?,
                subject: value()?,
            });
        }
        if records != revisions.len() {
            return Err("Malformed terminated Git history records".into());
        }
        let more = revisions.len() > self.policy.history_page_size;
        if more {
            revisions.pop();
        }
        let bytes = std::mem::size_of::<History>()
            + (self.policy.max_history_pages + 1) * std::mem::size_of::<Arc<HistoryPage>>() * 2
            + std::mem::size_of::<HistoryPage>()
            + revisions.capacity() * std::mem::size_of::<Revision>()
            + revisions.iter().map(|r| r.bytes()).sum::<usize>()
            + 4096;
        admission.grow(bytes)?;
        Ok(ResultData::Log(history::log_result(
            id,
            observation,
            root,
            retained,
            history::PageResult { revisions, more },
            self.policy.max_history_pages,
            admission.split(bytes),
        )))
    }

    fn revision(
        &mut self,
        history: Arc<History>,
        object: ObjectId,
        parent: Option<ObjectId>,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let metadata = self.command(
            &["--no-replace-objects", "cat-file", "commit", &object.hex()],
            Phase::Checking,
            false,
        )?;
        if metadata.len() > self.policy.max_detail_bytes {
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
        let (names, truncated) = self.command_os(
            &args,
            Phase::Refreshing,
            false,
            Some(self.policy.max_detail_bytes),
        )?;
        let mut completeness = if truncated {
            Completeness::Truncated {
                reason: "Revision changed-path output limit",
            }
        } else {
            Completeness::Complete
        };
        admission.grow(
            names.len() * 3
                + self.policy.max_status_paths.min(names.len())
                    * std::mem::size_of::<RevisionPath>()
                    * 2
                + 4096,
        )?;
        let mut paths = Vec::new();
        let mut fields = names
            .split_inclusive(|b| *b == 0)
            .map(|v| v.strip_suffix(b"\0").unwrap_or(v));
        let mut path_bytes = 0;
        if !truncated {
            while let Some(status) = fields.next() {
                if paths.len() >= self.policy.max_status_paths {
                    completeness = Completeness::Truncated {
                        reason: "Revision path count",
                    };
                    break;
                }
                let first = fields.next().ok_or("Incomplete revision path record")?;
                let rename = matches!(status.first(), Some(b'R' | b'C'));
                let second = if rename {
                    fields.next().ok_or("Incomplete historical rename")?
                } else {
                    first
                };
                path_bytes += first.len() + if rename { second.len() } else { 0 };
                if path_bytes > self.policy.max_path_bytes {
                    completeness = Completeness::Truncated {
                        reason: "Revision path bytes",
                    };
                    break;
                }
                validate_path(first)?;
                validate_path(second)?;
                paths.push(RevisionPath {
                    path: PathBuf::from(OsString::from_vec(second.to_vec())),
                    old_path: rename.then(|| PathBuf::from(OsString::from_vec(first.to_vec()))),
                    status: status.to_vec(),
                });
            }
        }
        let position = args
            .iter()
            .position(|v| v == "--name-status")
            .expect("revision diff mode");
        args[position] = "--patch".into();
        args[position + 1] = "--full-index".into();
        let (patch, truncated) = self.command_os(
            &args,
            Phase::Refreshing,
            false,
            Some(
                self.policy
                    .max_detail_bytes
                    .saturating_sub(metadata.len())
                    .max(1),
            ),
        )?;
        if truncated {
            completeness = Completeness::Truncated {
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
            + paths.capacity() * std::mem::size_of::<RevisionPath>()
            + paths
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
        let mut lines = Vec::with_capacity(chunks);
        let mut offset = 0;
        for line in patch.split_inclusive(|b| *b == b'\n') {
            for start in (0..line.len()).step_by(2048) {
                lines.push(offset + start..offset + (start + 2048).min(line.len()));
            }
            offset += line.len();
        }
        Ok(ResultData::Show(RevisionDetail {
            id: self.progress.lock().expect("Git progress owner").operation,
            history,
            object,
            parent,
            metadata,
            paths,
            patch,
            completeness,
            lines,
            _admission: admission.split(bytes),
        }))
    }
}

fn validate_path(path: &[u8]) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 4096
        || path[0] == b'/'
        || path
            .split(|b| *b == b'/')
            .any(|p| p == b".." || p.is_empty())
    {
        return Err("Historical path exceeds exact relative-path admission".into());
    }
    Ok(())
}
