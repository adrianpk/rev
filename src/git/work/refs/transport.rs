use super::*;
mod push;
use crate::git::PushIntent;

impl Runner {
    // Keep the complete effective configuration private, including URL rewrites,
    // helpers and refspecs. Output already has the shared aggregate admission;
    // every retained key/value also observes the 4096-byte metadata bound.
    pub(in crate::git::work) fn transport_config(
        &mut self,
        admission: &mut Lease,
    ) -> Result<Vec<u8>, String> {
        let bytes = self.command(&["config", "--null", "--list"], Phase::Checking, false)?;
        if !bytes.is_empty() && bytes.last() != Some(&0) {
            return Err("Transport configuration is incomplete".into());
        }
        for record in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let mut fields = record.splitn(2, |b| *b == b'\n');
            if fields.any(|field| field.len() > 4096) {
                return Err("Transport configuration exceeds metadata bounds".into());
            }
        }
        admission.grow(bytes.len())?;
        Ok(bytes)
    }
    fn configured_transport(config: &[u8], remote: &[u8]) -> Result<(), String> {
        let key = [b"remote.".as_slice(), remote, b".url"].concat();
        if !config.split(|b| *b == 0).any(|record| {
            record
                .strip_prefix(key.as_slice())
                .is_some_and(|rest| rest.starts_with(b"\n") && rest.len() > 1)
        }) {
            return Err(
                "Selected configured remote has no transport URL; no path fallback is admitted"
                    .into(),
            );
        }
        Ok(())
    }
    pub(super) fn fetch(
        &mut self,
        refs: &Arc<Refs>,
        remote: &[u8],
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        self.check_ref_snapshot(refs, admission)?;
        let config = self.transport_config(admission)?;
        Self::configured_transport(&config, remote)?;
        self.check_ref_snapshot(refs, admission)?;
        if self.transport_config(admission)? != config {
            return Err(
                "Transport/refspec configuration changed before fetch; select and retry".into(),
            );
        }
        // An explicit configured name uses Git's observed fetch refspecs. There
        // is no pull, merge, rebase, --all or --update-head-ok request.
        let args = [
            "fetch".into(),
            "--".into(),
            OsString::from_vec(remote.into()),
        ];
        let (executed, facts) = self.execute_change(&args, None);
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Fetch cleanup remains owned; tracking effects require inspection".into());
        }
        let result = (|| {
            let after = self.refresh_ref_result(refs, admission)?;
            let unchanged_head = after.head == refs.head && after.head_ref == refs.head_ref;
            let unchanged_config = self.transport_config(admission)? == config;
            if executed.is_ok() && unchanged_head && unchanged_config {
                self.effects = Effects::Verified;
            } else {
                self.failure_verified = executed.is_err() && after.same(refs) && unchanged_config;
                self.operation_failure = Some("Fetch failed or HEAD/configuration changed; checked local facts refreshed; no automatic retry".into());
            }
            Ok(Candidate::Refs {
                refs: after,
                changed: true,
            })
        })();
        self.restore_execution(facts, executed.is_err());
        if result.is_err() {
            self.operation_failure = Some(
                "Fetch tracking effects remain unverified; refresh required; no automatic retry"
                    .into(),
            );
        }
        result
    }
}
