use super::*;
impl Snapshot {
    /// Pure canonical checks on the editor owner. Newly visited sources replace
    /// disk authority even when their bytes have not yet changed.
    pub fn buffers_current(&self, buffers: &Buffers) -> Result<(), String> {
        for source in &self.sources {
            source.buffer_current(buffers)?;
        }
        Ok(())
    }
    /// Perform disk rechecks inside the ordinary owned worker before any later
    /// renderer publication. This check grants no destination/write authority.
    pub fn files_current(&self, cancel: &Cancellation) -> Result<(), String> {
        let budget = Budget {
            cancel,
            deadline: Instant::now() + Duration::from_secs(self.job_seconds as u64),
        };
        for source in &self.sources {
            budget.check()?;
            if let SourceVersion::Disk(observed) = &source.version {
                let path = source
                    .path
                    .as_deref()
                    .ok_or("Disk export source has no path")?;
                if crate::file::observe(path)
                    .map_err(|e| e.to_string())?
                    .as_ref()
                    != Some(observed)
                {
                    return Err("Org export disk source changed; prepare a new snapshot".into());
                }
            }
        }
        Ok(())
    }
}
impl Evidence {
    pub(crate) fn buffer_current(&self, buffers: &Buffers) -> Result<(), String> {
        match &self.version {
            SourceVersion::Buffer(stamp) => {
                let b = buffers.get(stamp.buffer).map_err(|e| e.to_string())?;
                if b.stamp() != *stamp || b.file().map(|f| f.path()) != self.path.as_deref() {
                    return Err(
                        "Org export canonical source changed; prepare a new snapshot".into(),
                    );
                }
            }
            SourceVersion::Disk(observed) => {
                let mut count = 0;
                for file in buffers.iter().filter_map(|b| b.file()) {
                    count += 1;
                    if count > 4096 {
                        return Err("Export freshness exceeds 4096 open files".into());
                    }
                    if Some(file.path()) == self.path.as_deref()
                        || file.observation().is_some_and(|o| {
                            o.device == observed.device && o.inode == observed.inode
                        })
                    {
                        return Err(
                            "Org export source is now canonical; prepare a new snapshot".into()
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
