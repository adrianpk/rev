use super::*;

impl Repositories {
    /// Fetch one explicitly selected configured remote. The worker captures and
    /// rechecks private transport/refspec configuration; no integration is run.
    pub fn fetch(&mut self, refs: Arc<Refs>, remote: &[u8]) -> Result<OperationId, String> {
        self.check_refs(&refs)?;
        if refs.completeness != Completeness::Complete {
            return Err("Fetch requires complete current Refs".into());
        }
        if remote.is_empty() || remote.len() > 4096 || !refs.remotes.iter().any(|r| r == remote) {
            return Err("Fetch requires one exact selected configured remote".into());
        }
        let directory = self.local_ref_directory(&refs)?;
        let admission = self.budget.reserve(remote.len() + 4096)?;
        self.start(
            directory,
            Some(refs.observation.context),
            work::Request::Refs(Request::Fetch {
                refs,
                remote: remote.into(),
                _admission: admission,
            }),
        )
    }
}
