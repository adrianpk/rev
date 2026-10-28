use super::*;
use crate::{org::time::TimerDuration, session::Pending};
pub(in crate::session) struct Prompt {
    generation: u64,
    default: String,
    stage: Stage,
}
enum Stage {
    Duration,
    Replace(TimerDuration),
}
impl Session {
    pub(super) fn open_countdown(&mut self) -> Action {
        if self.timer.mode() == Some(TimerMode::Relative) {
            return self.message("Relative timer is running; stop it first");
        }
        let prompt = Prompt {
            generation: self.timer.generation(),
            default: self.buffers.snapshot().settings().org.timer.default.clone(),
            stage: Stage::Duration,
        };
        match self.countdown_prompt(prompt, "") {
            Ok(()) => Action::Redraw,
            Err(error) => self.error(&error),
        }
    }
    fn countdown_prompt(&mut self, prompt: Prompt, draft: &str) -> Result<(), String> {
        let label = match prompt.stage {
            Stage::Duration => "Org countdown (minutes or H:MM:SS): ",
            Stage::Replace(_) => "Replace Org countdown (yes/no): ",
        };
        self.input
            .argument(label, draft)
            .map_err(|e| e.to_string())?;
        self.message(&format!(
            "Default {}; positive duration, at most 365 days; C-g cancels",
            prompt.default
        ));
        self.pending = Some(Pending::Timer(prompt));
        Ok(())
    }
    pub(in crate::session) fn timer_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if self.timer.generation() != prompt.generation {
            return self.message("Org timer changed; countdown request cancelled");
        }
        let result = self.countdown_argument(&prompt, draft);
        match result {
            Ok(Some(value)) => self.accept_countdown(prompt, value),
            Ok(None) => self.message("Org countdown unchanged"),
            Err(error) => {
                if let Err(reopen) = self.countdown_prompt(prompt, draft) {
                    return self.error(&reopen);
                }
                self.error(&error)
            }
        }
    }
    fn countdown_argument(
        &self,
        prompt: &Prompt,
        draft: &str,
    ) -> Result<Option<TimerDuration>, String> {
        match prompt.stage {
            Stage::Duration => {
                let value = TimerDuration::parse(if draft.trim().is_empty() {
                    &prompt.default
                } else {
                    draft
                })?;
                if value.seconds() == 0 {
                    return Err("Countdown requires a positive duration".into());
                }
                Ok(Some(value))
            }
            Stage::Replace(value) => match draft.trim() {
                "yes" | "y" => Ok(Some(value)),
                "no" | "n" => Ok(None),
                _ => Err("Enter yes or no; C-g cancels".into()),
            },
        }
    }
    fn accept_countdown(&mut self, mut prompt: Prompt, value: TimerDuration) -> Action {
        if matches!(prompt.stage, Stage::Duration)
            && self.timer.mode() == Some(TimerMode::Countdown)
        {
            prompt.stage = Stage::Replace(value);
            return match self.countdown_prompt(prompt, "") {
                Ok(()) => Action::Redraw,
                Err(error) => self.error(&error),
            };
        }
        match self.timer.countdown(self.timer_now(), value) {
            Ok(()) => self.message("Org countdown started"),
            Err(error) => self.error(&error),
        }
    }
}
