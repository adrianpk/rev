use super::*;
impl Session {
    pub(super) fn capture_prompt(&mut self, prompt: Prompt) -> Result<(), String> {
        let (label, initial) = match &prompt {
            Prompt::Template => ("Capture template key: ", ""),
            Prompt::File => ("Capture destination file: ", ""),
            Prompt::Expansion(prompt) => (prompt.label.as_str(), prompt.default.as_str()),
        };
        self.input
            .argument(label, initial)
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Capture(prompt));
        self.complete_capture();
        Ok(())
    }
    pub(in crate::session) fn capture_argument(&mut self, prompt: Prompt, value: &str) -> Action {
        let Some(state) = self.capture.take() else {
            return self.error("Capture is no longer active");
        };
        if !self.capture_current(&state.origin) {
            return self.error("Capture origin changed before prompt completion");
        }
        let State { origin, stage } = state;
        match self.capture_answer(&origin, stage, &prompt, value) {
            Ok(stage) => {
                self.capture = Some(State { origin, stage });
                if matches!(
                    self.capture.as_ref().map(|s| &s.stage),
                    Some(Stage::File(_))
                ) && let Err(e) = self.capture_prompt(Prompt::File)
                {
                    return self.error(&e);
                }
                self.advance_capture();
                Action::Redraw
            }
            Err(e) => self.error(&e),
        }
    }
    fn capture_answer(
        &mut self,
        origin: &Origin,
        stage: Stage,
        prompt: &Prompt,
        value: &str,
    ) -> Result<Stage, String> {
        match (stage, prompt) {
            (Stage::Template, Prompt::Template) => self.select_capture(origin, value),
            (Stage::File(mut choice), Prompt::File) => {
                choice.enter_file(value, &origin.directory)?;
                let work = Expansion::new(&choice.template, origin.context.clone())?;
                Ok(Stage::Expanding(choice, Box::new(work)))
            }
            (Stage::Expanding(choice, mut work), Prompt::Expansion(_)) => {
                work.answer(value)?;
                Ok(Stage::Expanding(choice, work))
            }
            _ => Err("Capture prompt does not own this stage".into()),
        }
    }
    fn select_capture(&self, origin: &Origin, value: &str) -> Result<Stage, String> {
        let key = if value.is_empty() { "t" } else { value };
        let choice = Choice::configured(self.buffers.snapshot(), key)?;
        // Parse before file entry or IO, without scanning substituted values.
        let expansion = Expansion::new(&choice.template, origin.context.clone())?;
        if choice.file.is_none() {
            return Ok(Stage::File(choice));
        }
        Ok(Stage::Expanding(choice, Box::new(expansion)))
    }
    pub(in crate::session) fn complete_capture(&mut self) -> Action {
        let notice = match &self.pending {
            Some(Pending::Capture(Prompt::Template)) => template_choices(self.buffers.snapshot()),
            Some(Pending::Capture(Prompt::File)) => {
                "Enter one literal local file; C-g cancels".into()
            }
            Some(Pending::Capture(Prompt::Expansion(prompt))) => {
                if prompt.completions.is_empty() {
                    "Enter text; C-g cancels capture".into()
                } else {
                    format!(
                        "Choices: {}; C-g cancels capture",
                        prompt.completions.join(" | ")
                    )
                }
            }
            _ => "No capture argument is active".into(),
        };
        self.message(&notice)
    }
}
fn template_choices(snapshot: &crate::config::Snapshot) -> String {
    let templates = &snapshot.settings().org.capture.templates;
    if templates.is_empty() {
        return "t Task (Enter selects t); C-g cancels".into();
    }
    templates
        .iter()
        .map(|t| format!("{} {}", t.key, t.description))
        .collect::<Vec<_>>()
        .join(" | ")
}
