//! An explicit successful instruction becomes data through the root publisher.
use crate::config::{AiCommand, Roots, SettingsEdit};
use crate::input::Focus;
use crate::session::{Action, Pending, Session};

pub(in crate::session) struct Prompt {
    edit: SettingsEdit,
    definition: AiCommand,
}
impl Session {
    pub(in crate::session) fn ai_save_command(&mut self) -> Action {
        let result = (|| {
            if self.input.focus() != Focus::Editing
                || self.pending.is_some()
                || self.ai.operation.is_some()
            {
                return Err(
                    "Finish the current input/AI operation before saving a command".to_owned(),
                );
            }
            let definition = self
                .ai
                .last
                .clone()
                .ok_or("No successful AI instruction to save")?;
            let root = self
                .buffers
                .snapshot()
                .root()
                .map(ToOwned::to_owned)
                .map(Ok)
                .unwrap_or_else(|| {
                    Roots::from_env()
                        .map(|r| r.config_file())
                        .map_err(|e| e.to_string())
                })?;
            let edit = SettingsEdit::capture(&root).map_err(|e| e.to_string())?;
            self.input
                .argument("AI command name (without ai-): ", "")
                .map_err(|e| e.to_string())?;
            self.pending = Some(Pending::AiSave(Box::new(Prompt { edit, definition })));
            Ok(())
        })();
        match result {Ok(())=>self.message("Save the successful literal instruction only; generated text is not the prompt; C-g cancels"),Err(error)=>self.error(&error)}
    }
    pub(in crate::session) fn ai_save_argument(
        &mut self,
        mut prompt: Prompt,
        name: &str,
    ) -> Action {
        let result = (|| {
            let identity = crate::command::AiName::new(name)?;
            if matches!(
                self.input.command_named(identity.name()),
                Some(crate::command::Command::MacroNamed(_))
            ) {
                return Err("AI name collides with a session macro command".into());
            }
            let active = &self.buffers.snapshot().settings().ai.commands;
            let captured = prompt.edit.snapshot().map_err(|e| e.to_string())?;
            if active.contains_key(name) || captured.settings().ai.commands.contains_key(name) {
                return Err("AI definition already exists; choose another name".to_owned());
            }
            if active.len() >= 64 {
                return Err("AI permits at most 64 active definitions".into());
            }
            prompt.edit.ai_command(name, &prompt.definition)?;
            let state = Roots::from_env().map_err(|e| e.to_string())?.state;
            let published = prompt.edit.publish(&state).map_err(|e| e.to_string())?;
            self.buffers.apply_ai_definition(name, &published.snapshot);
            self.input.admit_ai_command(identity);
            Ok(published.warning)
        })();
        match result {
            Ok(Some(warning)) => self.message(&warning),
            Ok(None) => self.message(
                "AI command saved and callable now; new configured keybindings require restart",
            ),
            Err(error) => self.error(&error),
        }
    }
}
