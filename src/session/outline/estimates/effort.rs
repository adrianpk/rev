use super::*;
use crate::org::Duration;

impl Session {
    pub(super) fn start_effort(&mut self, mut prompt: Prompt) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let entry = prompt.model.heading_property_entry(buffer, prompt.index)?;
        let policy = &self.buffers.snapshot().settings().org;
        let current =
            prompt
                .model
                .property_value(buffer, entry, "EFFORT", &policy.properties.inherit)?;
        if prompt.request.command == OrgCommand::InspectEffort {
            let feedback = effort_feedback(&current.text, &policy.duration.units)?;
            self.message(&feedback);
            return Ok(());
        }
        if buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let choices =
            prompt
                .model
                .property_value(buffer, entry, "EFFORT_ALL", &["EFFORT_ALL".into()])?;
        prompt.choices = effort_choices(&choices.text, &policy.duration.units)?;
        if prompt.request.command == OrgCommand::IncEffort {
            let next = next_effort(&prompt.choices, &current.text)?;
            return self.effort_argument(&prompt, &next).map(|_| ());
        }
        self.open_effort(prompt)
    }
    fn open_effort(&mut self, prompt: Prompt) -> Result<(), String> {
        self.input
            .argument("Org effort: ", "")
            .map_err(|e| e.to_string())?;
        let feedback = if prompt.choices.is_empty() {
            "Minutes, H:MM[:SS] or min/h/d/w/m/y; empty removes the local estimate".into()
        } else {
            format!(
                "Effort choices: {}; or enter an Org duration",
                prompt.choices.join(", ")
            )
        };
        self.pending = Some(Pending::Estimate(Box::new(prompt)));
        self.message(&feedback);
        Ok(())
    }
    pub(super) fn effort_argument(
        &mut self,
        prompt: &Prompt,
        draft: &str,
    ) -> Result<Action, String> {
        let draft = draft.trim();
        if !draft.is_empty() {
            Duration::parse(
                draft,
                &self.buffers.snapshot().settings().org.duration.units,
            )?;
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let entry = prompt.model.heading_property_entry(buffer, prompt.index)?;
        let current = prompt.model.property_value(buffer, entry, "EFFORT", &[])?;
        if current.text == draft {
            return Ok(self.message("Org effort unchanged"));
        }
        let edits = prompt.model.property_edit(
            buffer,
            entry,
            "EFFORT",
            (!draft.is_empty()).then_some(draft),
        )?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        Ok(self.message("Org effort changed"))
    }
}
fn effort_feedback(text: &str, units: &crate::config::DurationUnits) -> Result<String, String> {
    if text.is_empty() {
        return Ok("Org effort: absent".into());
    }
    let value = Duration::parse(text, units)?;
    Ok(format!("Org effort: {text} = {} minutes", value.minutes()))
}
fn effort_choices(text: &str, units: &crate::config::DurationUnits) -> Result<Vec<String>, String> {
    let mut choices = Vec::new();
    for word in text.split_whitespace() {
        if choices.len() == 64 {
            return Err("Org EFFORT_ALL exceeds 64 choices".into());
        }
        Duration::parse(word, units)?;
        choices.push(word.into());
    }
    Ok(choices)
}
fn next_effort(choices: &[String], current: &str) -> Result<String, String> {
    if choices.is_empty() {
        return Err("No allowed EFFORT_ALL choices for cycling".into());
    }
    let index = choices
        .iter()
        .position(|s| s == current)
        .map_or(0, |i| (i + 1) % choices.len());
    Ok(choices[index].clone())
}
