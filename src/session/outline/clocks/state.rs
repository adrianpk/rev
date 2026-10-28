mod restore;
use super::*;
use crate::org::io::{ClockState, ClockStore};
use std::path::Path;

impl Session {
    /// Explicit native-host initialization. With persistence off this performs
    /// no state IO. Library constructors never infer an environment state root.
    pub fn restore_clock_state(&mut self, path: &Path) -> Result<Action, String> {
        if !self.buffers.snapshot().settings().org.clock.persist {
            return Ok(Action::Wait);
        }
        let store = ClockStore::open(path)?;
        let clock = store.clock().cloned();
        self.clock_store = Some(store);
        if let Some(record) = clock {
            return self.ask_clock(Prompt::Stored(record), "Org saved clock: ", "recover = visit and verify recorded source; keep = ignore saved state; C-g cancels");
        }
        Ok(Action::Wait)
    }
    pub(in crate::session) fn startup_clock_state(&mut self) {
        if !self.buffers.snapshot().settings().org.clock.persist {
            return;
        }
        let path = crate::config::Roots::from_env()
            .map(|r| r.state.join("org-clock.toml"))
            .map_err(|e| e.to_string());
        let result = path.and_then(|path| self.restore_clock_state(&path));
        if let Err(error) = result {
            self.notice(&format!("Org clock state: {error}"));
        }
    }
    pub(super) fn checkpoint_clock(&mut self) -> Result<(), String> {
        if !self.buffers.snapshot().settings().org.clock.persist {
            return Ok(());
        }
        let value = self
            .active_clock
            .as_ref()
            .or(self.retained_clock.as_ref())
            .map(|active| self.clock_state_record(active))
            .transpose()?;
        let store = self
            .clock_store
            .as_mut()
            .ok_or("Clock source/tracking retained; Org clock state path is not initialized")?;
        store.write(value).map_err(|e| {
            format!("Clock source/tracking retained; Org clock state not acknowledged: {e}")
        })
    }
    pub(in crate::session) fn checkpoint_saved_clock(&mut self, path: &Path) -> Result<(), String> {
        let active = self.active_clock.as_ref().or(self.retained_clock.as_ref());
        let matches = active
            .and_then(|a| self.buffers.get(a.source.stamp.buffer).ok())
            .and_then(|b| b.file())
            .is_some_and(|f| f.path() == path);
        if matches {
            self.checkpoint_clock()?;
        }
        Ok(())
    }
    fn clock_state_record(&self, active: &Active) -> Result<ClockState, String> {
        let buffer = self
            .buffers
            .get(active.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let source = buffer
            .slice(&active.source)
            .map_err(|e| format!("Clock state source changed: {e}; resolve explicitly"))?
            .to_string();
        let path = buffer
            .file()
            .ok_or("Save the clock source to a named file before persisting it")?
            .path();
        let file = crate::file::observe(path)
            .map_err(|e| e.to_string())?
            .ok_or("Clock source file does not exist; save it first")?;
        let record = ClockState {
            path: path.into(),
            file,
            heading: active.owner.position.0,
            heading_source: heading_source(buffer, active.owner.position.0)?,
            line_start: active.source.range.start,
            line_end: active.source.range.end,
            source: source.trim_end().into(),
            start_seconds: active.interval.started().unix_seconds(),
            acknowledged_seconds: active.acknowledged.unix_seconds(),
        };
        record.validate()?;
        Ok(record)
    }
}
fn heading_source(buffer: &crate::buffer::Buffer, at: usize) -> Result<String, String> {
    let span = buffer
        .span(at..at.saturating_add(4096).min(buffer.len_chars()))
        .map_err(|e| e.to_string())?;
    let text = buffer.slice(&span).map_err(|e| e.to_string())?.to_string();
    let line = text.lines().next().ok_or("Clock heading is missing")?;
    if line.len() > 4096 {
        return Err("Clock heading exceeds 4096 UTF-8 bytes".into());
    }
    Ok(line.into())
}
