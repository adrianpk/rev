use super::Request;
use crate::buffer::edit::{Error, Replacement};
use crate::command::OrgCommand;
use crate::org::{Outline, derived_property, property_name};
use crate::session::{Action, Pending, Session};
use std::sync::Arc;

pub(in crate::session) struct Prompt {
    request: Request,
    model: Arc<Outline>,
    entry: usize,
    key: String,
    stage: Stage,
}
enum Stage {
    Name,
    Owner(Vec<usize>),
    Value(Option<usize>),
}
impl Session {
    pub(super) fn start_property(
        &mut self,
        model: Arc<Outline>,
        request: Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let entry = model.property_target(buffer, self.view().point.position)?;
        if request.command != OrgCommand::InspectProperty && buffer.is_readonly() {
            return Err(Error::ReadOnly.to_string());
        }
        let prompt = Prompt {
            request,
            model,
            entry,
            key: String::new(),
            stage: Stage::Name,
        };
        self.property_prompt(
            prompt,
            "Org property: ",
            "Local property name; append + to add a value",
        )
        .map(|_| ())
    }
    fn property_prompt(
        &mut self,
        prompt: Prompt,
        label: &'static str,
        feedback: &str,
    ) -> Result<Action, String> {
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Property(Box::new(prompt)));
        Ok(self.message(feedback))
    }
    pub(in crate::session) fn property_argument(
        &mut self,
        mut prompt: Prompt,
        draft: &str,
    ) -> Action {
        if !self.org_request_current(&prompt.request) {
            return self.message("Org property source or view changed; request cancelled");
        }
        let stage = std::mem::replace(&mut prompt.stage, Stage::Name);
        let result = match stage {
            Stage::Name => self.property_named(prompt, draft),
            Stage::Value(record) => self.property_changed(prompt, record, draft),
            Stage::Owner(records) => self.property_owner(prompt, &records, draft),
        };
        match result {
            Ok(action) => action,
            Err(error) => self.error(&error),
        }
    }
    fn property_named(&mut self, mut prompt: Prompt, draft: &str) -> Result<Action, String> {
        prompt.key = property_name(draft.trim())?;
        if derived_property(&prompt.key) && prompt.request.command == OrgCommand::InspectProperty {
            return Ok(self.message(&format!(
                "{} is derived from heading/file state; it has no stored property owner",
                prompt.key
            )));
        }
        if derived_property(&prompt.key) {
            return Err(
                "Derived Org property; inspect the heading or file instead of storing it".into(),
            );
        }
        match prompt.request.command {
            OrgCommand::InspectProperty => self.inspect_property(&prompt),
            OrgCommand::DeleteProperty => self.property_deleted(prompt),
            OrgCommand::EditPropertyOwner => self.choose_property_owner(prompt),
            _ => {
                prompt.stage = Stage::Value(None);
                self.property_prompt(
                    prompt,
                    "Org property value: ",
                    "Set a locally owned value; C-g cancels",
                )
            }
        }
    }
    fn effective_property(&self, prompt: &Prompt) -> Result<crate::org::PropertyValue, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        prompt.model.property_value(
            buffer,
            prompt.entry,
            &prompt.key,
            &self.buffers.snapshot().settings().org.properties.inherit,
        )
    }
    fn inspect_property(&mut self, prompt: &Prompt) -> Result<Action, String> {
        let value = self.effective_property(prompt)?;
        let origins = self.property_origins(prompt, &value.records)?;
        let value: String = if value.undefined {
            "<undefined>".into()
        } else {
            value.text.chars().take(512).collect()
        };
        Ok(self.message(&format!("{} = {value:?}; {origins}", prompt.key)))
    }
    fn property_origins(&self, prompt: &Prompt, records: &[usize]) -> Result<String, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let mut origins = Vec::new();
        for record in records.iter().take(16) {
            origins.push(
                prompt
                    .model
                    .property_origin(buffer, *record, prompt.entry)?,
            );
        }
        if records.len() > 16 {
            origins.push("additional owners; inspect source".into());
        }
        Ok(if origins.is_empty() {
            "absent".into()
        } else {
            origins.join("; ")
        })
    }
    fn choose_property_owner(&mut self, mut prompt: Prompt) -> Result<Action, String> {
        let value = self.effective_property(&prompt)?;
        if value.records.is_empty() {
            return Err("No effective Org property owner".into());
        }
        let origins = self.property_origins(&prompt, &value.records)?;
        if value.records.len() == 1 {
            prompt.stage = Stage::Value(Some(value.records[0]));
            return self.property_prompt(prompt, "Owner property value: ", &origins);
        }
        prompt.stage = Stage::Owner(value.records);
        self.property_prompt(
            prompt,
            "Property owner number: ",
            &format!("Choose 1-based contribution: {origins}"),
        )
    }
    fn property_owner(
        &mut self,
        mut prompt: Prompt,
        records: &[usize],
        draft: &str,
    ) -> Result<Action, String> {
        let index = draft
            .parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|n| records.get(n))
            .copied()
            .ok_or("Invalid Org property owner number")?;
        let origin = self.property_origins(&prompt, &[index])?;
        prompt.stage = Stage::Value(Some(index));
        self.property_prompt(prompt, "Owner property value: ", &origin)
    }
    fn property_deleted(&mut self, prompt: Prompt) -> Result<Action, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let edits = prompt
            .model
            .property_edit(buffer, prompt.entry, &prompt.key, None)?;
        self.publish_property(&edits).map_err(|e| e.to_string())?;
        Ok(self.message("Local Org property removed; inherited owners retained"))
    }
    fn property_changed(
        &mut self,
        prompt: Prompt,
        record: Option<usize>,
        value: &str,
    ) -> Result<Action, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let edits = match record {
            Some(record) => prompt.model.property_owner_edit(buffer, record, value)?,
            None => prompt
                .model
                .property_edit(buffer, prompt.entry, &prompt.key, Some(value))?,
        };
        self.publish_property(&edits).map_err(|e| e.to_string())?;
        Ok(self.message("Org property changed"))
    }
    fn publish_property(&mut self, edits: &[Replacement]) -> Result<(), Error> {
        if edits.is_empty() {
            return Ok(());
        }
        let buffer = self
            .buffers
            .get_mut(self.windows.selected().point.stamp.buffer)?;
        let points = self.windows.points(buffer)?;
        let mut prepared = buffer.prepare(edits, &points.values, None)?;
        prepared.retain_initiator(points.initiator)?;
        prepared.set_origin(self.windows.selected_id());
        self.windows.publish(buffer, prepared, points)?;
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
