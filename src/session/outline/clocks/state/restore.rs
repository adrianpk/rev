use super::*;

impl Session {
    pub(in crate::session::outline::clocks) fn restore_stored_clock(
        &mut self,
        record: ClockState,
        draft: &str,
    ) -> Result<Action, String> {
        match draft.trim() {
            "keep" => Ok(self.message("Saved Org clock remains unresolved; source not visited")),
            "recover" => self.recover_stored_source(record),
            _ => self.ask_clock(
                Prompt::Stored(record),
                "Org saved clock: ",
                "Enter recover or keep; C-g cancels",
            ),
        }
    }
    fn recover_stored_source(&mut self, record: ClockState) -> Result<Action, String> {
        record.validate()?;
        if self.active_clock.is_some() {
            return Err("Resolve the running Org clock before restoring saved state".into());
        }
        current_file(&record)?;
        self.load(&record.path).map_err(|e| e.to_string())?;
        self.verify_stored_source(&record)?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let owner = Point {
            stamp: buffer.stamp(),
            position: crate::coordinates::Scalar(record.heading),
        };
        self.place_point(owner).map_err(|e| e.to_string())?;
        self.clock_recovery_pin = Some(record);
        Ok(self.org_command(OrgCommand::ResolveClocks))
    }
    pub(in crate::session::outline::clocks) fn verify_stored_source(
        &self,
        record: &ClockState,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if !buffer.is_org() || buffer.file().is_none_or(|f| f.path() != record.path) {
            return Err("Saved clock requires its canonical Org source buffer".into());
        }
        current_source(buffer, record)?;
        current_file(record)?;
        Ok(())
    }
}

fn current_file(record: &ClockState) -> Result<(), String> {
    let file = crate::file::observe(&record.path).map_err(|e| e.to_string())?;
    if file.as_ref() != Some(&record.file) {
        return Err("Saved Org clock source file changed; state and source retained".into());
    }
    if record.file.length > 1048576 {
        return Err(
            "Saved Org clock source exceeds 1048576 bytes; select and inspect it explicitly".into(),
        );
    }
    Ok(())
}
fn current_source(buffer: &crate::buffer::Buffer, record: &ClockState) -> Result<(), String> {
    let span = buffer
        .span(record.line_start..record.line_end)
        .map_err(|e| e.to_string())?;
    let source = buffer.slice(&span).map_err(|e| e.to_string())?.to_string();
    if source.trim_end() != record.source
        || heading_source(buffer, record.heading)? != record.heading_source
    {
        return Err("Saved Org clock source text differs; state and source retained".into());
    }
    Ok(())
}
