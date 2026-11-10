use super::*;

impl Runner {
    pub(super) fn verify_index_result(
        &mut self,
        diff: &Diff,
        expected: &[Entry],
        admission: &mut Lease,
    ) -> Result<(Candidate, Result<(), String>), String> {
        let (identity, status) = self.observe(admission)?;
        let actual = self.observed_index_entries(diff, expected)?;
        let valid = actual
            .iter()
            .all(|entry| expected.iter().any(|e| e.path == entry.path))
            && actual
                .iter()
                .enumerate()
                .all(|(i, entry)| actual[..i].iter().all(|e| e.path != entry.path))
            && identity == diff.observation.identity
            && status.as_ref().map(|s| &s.head)
                == diff.observation.status.as_ref().map(|s| &s.head)
            && expected.iter().all(|expected| expected.matches(&actual));
        Ok((
            Candidate::Status(identity, status),
            if valid {
                Ok(())
            } else {
                Err("Index postcondition differs from the captured request; effects remain uncertain".into())
            },
        ))
    }
    fn observed_index_entries(
        &mut self,
        diff: &Diff,
        expected: &[Entry],
    ) -> Result<Vec<Entry>, String> {
        let mut actual = Vec::with_capacity(expected.len());
        // ls-files has no input manifest. Bounded read-only batches preserve
        // the 64-token argv bound without breaking the changing action apart.
        for batch in expected.chunks(12) {
            let mut args: Vec<OsString> = vec![
                "--literal-pathspecs".into(),
                "ls-files".into(),
                "--stage".into(),
                "-z".into(),
                "--".into(),
            ];
            args.extend(batch.iter().map(|e| e.path.as_os_str().to_owned()));
            let (bytes, _) = self.command_os(&args, Phase::Verifying, false, None)?;
            actual.extend(entries(
                &bytes,
                diff.observation.identity.format,
                true,
                batch.len(),
            )?);
        }
        Ok(actual)
    }
}

impl Entry {
    fn matches(&self, actual: &[Entry]) -> bool {
        match self.object {
            None => !actual.iter().any(|e| e.path == self.path),
            Some(object) => actual.iter().any(|e| {
                e.path == self.path
                    && e.object == Some(object)
                    && self.mode.is_none_or(|mode| e.mode == Some(mode))
            }),
        }
    }
}
