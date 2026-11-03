use super::Session;
use crate::{display::Frame, view::Error};

impl Session {
    /// Native activities share the active pane's finite status space. Reading
    /// labels schedules no work and introduces no clock or post-job deadline.
    pub(super) fn activity_frame(&self, frame: &mut Frame) -> Result<(), Error> {
        let timer = self.timer_status();
        let shell = self.shell_status();
        if timer.is_none() && shell.is_none() {
            return Ok(());
        }
        let buffer = self.buffer()?;
        let mut label = buffer.mode().to_owned();
        if buffer.external_change_notice().is_some() {
            label.push_str(" disk: revert-buffer");
        }
        for activity in timer.as_deref().into_iter().chain(shell) {
            label.push(' ');
            label.push_str(activity);
        }
        frame.activity_status(&label)?;
        Ok(())
    }
}
