mod ancestry;
mod comparison;
mod inspection;
use super::*;
use crate::git::history::{self, Request, ResultData};
pub(super) use inspection::unquote_path;

impl Runner {
    pub(super) fn history(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let status = self.history_source(request.observation(), admission)?;
        let prior = request.ancestry_owner();
        if let Some(history) = &prior {
            self.verify_ancestry(history)?;
        }
        let result = self.history_result(request, admission)?;
        if let Some(history) = &prior {
            self.verify_ancestry(history)?;
        }
        if let Some(status) = &status {
            self.verify_head(status)?;
        }
        Ok(result)
    }
    fn history_source(
        &mut self,
        observation: &Observation,
        admission: &mut Lease,
    ) -> Result<Option<Status>, String> {
        let (identity, status) = self.observe(admission)?;
        if identity != observation.identity
            || status.as_ref().map(|s| &s.head) != observation.status.as_ref().map(|s| &s.head)
        {
            return Err(
                "Git HEAD or repository changed; refresh Status before inspecting History".into(),
            );
        }
        Ok(status)
    }
    fn history_result(
        &mut self,
        request: Request,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        match request {
            Request::Log(request) => {
                self.ancestry(request, self.policy.history_page_size, admission)
            }
            Request::Show {
                history,
                object,
                parent,
            } => comparison::Target {
                history,
                object,
                parent,
                range: false,
            }
            .read(self, admission),
            Request::Range { history, base, end } => comparison::Target {
                history,
                object: end,
                parent: Some(base),
                range: true,
            }
            .read(self, admission),
            Request::Reflog(observation) => self.read_reflog(observation, admission),
            Request::Blame { observation, path } => self.read_blame(observation, path, admission),
            Request::File(request) => self.read_revision_file(request, admission),
            Request::ShowObject {
                observation,
                object,
            } => self.history_object(observation, object, admission),
        }
    }
    fn history_object(
        &mut self,
        observation: Arc<Observation>,
        object: ObjectId,
        admission: &mut Lease,
    ) -> Result<ResultData, String> {
        let ResultData::Log(history) = self.ancestry(
            history::Ancestry {
                observation,
                root: Some(object.hex()),
                prior: None,
                next: false,
            },
            1,
            admission,
        )?
        else {
            unreachable!("ancestry result");
        };
        let parent = history
            .revision(object)
            .ok_or("Exact inspection object is unavailable")?
            .parents
            .first()
            .copied();
        comparison::Target {
            history: Arc::new(history),
            object,
            parent,
            range: false,
        }
        .read(self, admission)
    }

    pub(super) fn verify_ancestry(&mut self, history: &History) -> Result<(), String> {
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
