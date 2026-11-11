use super::Session;
use crate::{display::Frame, view::Error};

impl Session {
    /// Native activities share the active pane's finite status space. Reading
    /// labels schedules no work and introduces no clock or post-job deadline.
    pub(super) fn activity_frame(&self, frame: &mut Frame) -> Result<(), Error> {
        let timer = self.timer_status();
        let shell = self.shell_status();
        let macro_status = self.macro_activity_label();
        if timer.is_none() && shell.is_none() && macro_status.is_none() {
            return Ok(());
        }
        let buffer = self.buffer()?;
        let mut label = buffer.mode().to_owned();
        if buffer.external_change_notice().is_some() {
            label.push_str(" disk: revert-buffer");
        }
        for activity in timer
            .as_deref()
            .into_iter()
            .chain(shell)
            .chain(macro_status.as_deref())
        {
            label.push(' ');
            label.push_str(activity);
        }
        frame.activity_status(&label)?;
        Ok(())
    }

    fn macro_activity_label(&self) -> Option<String> {
        match self.macro_wait() {
            Some(crate::macros::Wait::Query) => Some("Macro?".to_owned()),
            Some(crate::macros::Wait::Editing) => {
                Some(format!("Macro edit {}/4", self.macros.pauses.len()))
            }
            _ => match self.macro_status() {
                crate::macros::Status::Recording => Some("Def".to_owned()),
                crate::macros::Status::Playing => Some("Macro".to_owned()),
                crate::macros::Status::Idle => None,
            },
        }
    }
}
