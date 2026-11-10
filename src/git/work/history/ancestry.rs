use super::*;
use crate::git::history::Ancestry;

struct RevisionHeader {
    object: ObjectId,
    parents: Vec<ObjectId>,
}
impl Runner {
    pub(super) fn ancestry(
        &mut self,
        request: Ancestry,
        page_size: usize,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let Ancestry {
            observation,
            root,
            prior,
            next,
        } = request;
        let root = self.history_root(&observation, root, prior.as_deref())?;
        if !next && let Some(prior) = prior {
            return self.retained_ancestry(observation, root, &prior, admission);
        }
        let retained = prior;
        let skip = retained.as_ref().map_or(0, |h| h.revisions().count());
        let bytes = self.history_page(root, skip, page_size)?;
        let mut revisions =
            self.history_revisions(&bytes, observation.identity.format, admission)?;
        let more = revisions.len() > page_size;
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
        let id = self.progress.lock().expect("Git progress owner").operation;
        Ok(ResultData::Log(
            History::new(id, observation, root, admission.split(bytes)).with_page(
                retained.as_deref(),
                history::PageResult { revisions, more },
                self.policy.max_history_pages,
            ),
        ))
    }

    fn history_root(
        &mut self,
        observation: &Observation,
        root: Option<String>,
        prior: Option<&History>,
    ) -> Result<Option<ObjectId>, String> {
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
        Ok(root)
    }
    fn retained_ancestry(
        &mut self,
        observation: Arc<Observation>,
        root: Option<ObjectId>,
        prior: &History,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
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
        let id = self.progress.lock().expect("Git progress owner").operation;
        Ok(ResultData::Log(
            History::new(id, observation, root, admission.split(bytes)).with_page(
                Some(prior),
                history::PageResult {
                    revisions: Vec::new(),
                    more,
                },
                self.policy.max_history_pages,
            ),
        ))
    }
    fn history_page(
        &mut self,
        root: Option<ObjectId>,
        skip: usize,
        page_size: usize,
    ) -> Result<Vec<u8>, String> {
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
                    &format!("--max-count={}", page_size + 1),
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
        Ok(bytes)
    }
    fn history_revisions(
        &mut self,
        bytes: &[u8],
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Vec<Revision>, String> {
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
            revisions.push(revision_record(object, &mut fields, format)?);
        }
        if records != revisions.len() {
            return Err("Malformed terminated Git history records".into());
        }
        Ok(revisions)
    }
}
fn field<'a>(fields: &mut impl Iterator<Item = &'a [u8]>) -> Result<&'a [u8], String> {
    fields
        .next()
        .ok_or_else(|| "Incomplete Git history record".into())
}
fn value<'a>(fields: &mut impl Iterator<Item = &'a [u8]>) -> Result<Vec<u8>, String> {
    let value = field(fields)?;
    if value.len() > 4096 {
        return Err("Revision metadata exceeds 4096 bytes".into());
    }
    Ok(value.to_vec())
}
fn revision_header<'a>(
    object: &[u8],
    fields: &mut impl Iterator<Item = &'a [u8]>,
    format: ObjectFormat,
) -> Result<RevisionHeader, String> {
    let parents = field(fields)?;
    if parents.len() > 4096 {
        return Err("Revision parent metadata exceeds 4096 bytes".into());
    }
    let parents = if parents.is_empty() {
        Vec::new()
    } else {
        parents
            .split(|b| *b == b' ')
            .map(|p| ObjectId::parse(p, format))
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(RevisionHeader {
        object: ObjectId::parse(object, format)?,
        parents,
    })
}
fn revision_record<'a>(
    object: &[u8],
    fields: &mut impl Iterator<Item = &'a [u8]>,
    format: ObjectFormat,
) -> Result<Revision, String> {
    let header = revision_header(object, fields, format)?;
    Ok(Revision {
        object: header.object,
        parents: header.parents,
        author: value(fields)?,
        author_email: value(fields)?,
        authored: value(fields)?,
        committer: value(fields)?,
        committer_email: value(fields)?,
        committed: value(fields)?,
        subject: value(fields)?,
    })
}
