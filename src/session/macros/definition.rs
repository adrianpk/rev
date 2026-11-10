//! One native editable TOML buffer pins one independent session target.
use super::*;
use crate::buffer::BufferId;
use crate::command::{MacroCommand, MacroName};
use crate::input::keymap::Sequence;
use crate::macros::{
    Definition, Target,
    editing::Selection,
    library::{Catalog, Record},
};
use crate::session::Pending;

pub(in crate::session) struct Draft {
    pub buffer: BufferId,
    target: Selection,
    template: Record,
    mode: Option<crate::input::EditingState>,
    repeat: bool,
}
impl Session {
    pub(in crate::session) fn macro_definition_selected(&self) -> bool {
        self.macro_draft
            .as_ref()
            .is_some_and(|d| d.buffer == self.view().point.stamp.buffer)
    }
    pub(super) fn macro_definition_command(&mut self, command: MacroCommand) -> Action {
        match command {
            MacroCommand::FinishEdit => return self.macro_finish_definition(),
            MacroCommand::InsertKey => {
                if !self.macro_definition_selected() {
                    return self.message("Select a macro definition draft first");
                }
                if let Err(e) = self.input.macro_literal_key_capture() {
                    return self.error(&e.to_string());
                }
                self.pending = Some(Pending::MacroDefinitionKey);
                return self.message("Read a normalized native key sequence; C-g cancels");
            }
            MacroCommand::RegionLines => return self.macro_definition_region(),
            _ => {}
        }
        if self.macros.active() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Definition editing requires an idle macro session",
            );
        }
        if self.macro_draft.is_some() {
            return self.message("Apply or close the retained macro definition draft first");
        }
        if !self.macro_source() {
            return self.message("Definition editing requires ordinary source editing");
        }
        if command == MacroCommand::EditNamed {
            if let Err(e) = self.input.argument("Macro name or key sequence: ", "") {
                return self.error(&e.to_string());
            }
            self.pending = Some(Pending::MacroEditSelect);
            return self.message("");
        }
        if command == MacroCommand::Lossage {
            return self.macro_edit_lossage();
        }
        let Some(source) = self.macros.ring.front().cloned() else {
            return self.message("No keyboard macro is available");
        };
        self.macro_open_definition(Selection::Ring(source), command == MacroCommand::EditRepeat)
    }
    pub(in crate::session) fn macro_edit_selected(&mut self, value: &str) -> Action {
        if !self.macro_source() {
            return self.message("Macro editing source changed");
        }
        let target = MacroName::new(value)
            .ok()
            .and_then(|n| {
                self.macros
                    .names
                    .get(&n)
                    .map(|v| Selection::Named(n, Arc::clone(&v.definition)))
            })
            .or_else(|| {
                let key = Sequence::parse(value).ok()?;
                match self.macros.bindings.get(&key)? {
                    Target::Named(name) => self
                        .macros
                        .names
                        .get(name)
                        .map(|v| Selection::Named(*name, Arc::clone(&v.definition))),
                    Target::Snapshot(source) => Some(Selection::Bound(key, Arc::clone(source))),
                }
            });
        match target {
            Some(target) => self.macro_open_definition(target, false),
            None => self.message("No named or bound macro has that identity"),
        }
    }
    pub(super) fn macro_open_definition(&mut self, target: Selection, repeat: bool) -> Action {
        let result = (|| -> Result<(), String> {
            if self.macro_draft.is_some() {
                return Err("Another macro draft is retained".into());
            }
            self.stop_typing().map_err(|e| e.to_string())?;
            self.macros
                .admit_memory(Some(target.source()))
                .map_err(|_| "Macro retained memory exceeds 48 MiB")?;
            let description = match &target {
                Selection::Named(n, _) => {
                    self.macros.names.get(n).and_then(|v| v.description.clone())
                }
                _ => None,
            };
            let template = Record::draft(&target, description).map_err(|e| e.to_string())?;
            let text = template.draft_text().map_err(|e| e.to_string())?;
            let mode = self.editing_state();
            let buffer = self.buffers.macro_draft(&text).map_err(|e| e.to_string())?;
            let source = self.buffers.get(buffer).map_err(|e| e.to_string())?;
            if let Err(e) = self
                .windows
                .retain_view(source, crate::view::View::new(source).point)
            {
                self.discard_capture_buffer(buffer)?;
                return Err(e);
            }
            self.macros.draft_pin = Some(Arc::clone(target.source()));
            self.macro_draft = Some(Draft {
                buffer,
                target,
                template,
                mode,
                repeat,
            });
            self.sync_input_context();
            Ok(())
        })();
        match result {Ok(())=>self.message("Macro definition: C-c C-c applies; close buffer discards; C-c C-q inserts keys; C-c C-r selects body lines"),Err(e)=>self.error(&e)}
    }
    fn macro_definition_record(&self) -> Result<Record, String> {
        if !self.macro_definition_selected() {
            return Err("Select the retained macro definition draft".into());
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        if buffer.len_bytes() > crate::macros::DEFINITION_BYTES + 4096 {
            return Err("Macro draft exceeds the bounded definition document".into());
        }
        let text = buffer
            .slice(
                &buffer
                    .span(0..buffer.len_chars())
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?
            .to_string();
        let mut catalog = Catalog::decode(Some(text.as_bytes())).map_err(|e| e.to_string())?;
        if catalog.macros.len() != 1 {
            return Err("Macro draft requires exactly one definition".into());
        }
        let record = catalog.macros.remove(0);
        let draft = self.macro_draft.as_ref().ok_or("No macro draft")?;
        if record.id != draft.template.id
            || record.revision != draft.template.revision
            || record.name != draft.template.name
            || record.binding != draft.template.binding
            || (!matches!(draft.target, Selection::Named(..))
                && record.description != draft.template.description)
        {
            return Err(
                "Macro draft cannot retarget its pinned identity, revision, name or binding".into(),
            );
        }
        Ok(record)
    }
    fn macro_finish_definition(&mut self) -> Action {
        let result = (|| -> Result<(), String> {
            self.stop_typing().map_err(|e| e.to_string())?;
            let record = self.macro_definition_record()?;
            let draft = self.macro_draft.as_ref().ok_or("No macro draft")?;
            let definition = record
                .definition(draft.target.source().id)
                .map_err(|e| e.to_string())?;
            let limit = self.buffers.snapshot().settings().macros.ring_limit;
            self.macros
                .replace_definition(&draft.target, definition, record.description, limit)?;
            Ok(())
        })();
        match result {
            Ok(()) => self.macro_leave_definition(
                "Macro definition applied to its session target; saving remains explicit",
            ),
            Err(e) => self.error(&e),
        }
    }
    fn macro_leave_definition(&mut self, message: &str) -> Action {
        let Some(draft) = self.macro_draft.take() else {
            return self.message("No macro definition draft");
        };
        self.windows.restore_view();
        self.macros.draft_pin = None;
        self.input.cancel();
        self.pending = None;
        if let Err(e) = self.discard_capture_buffer(draft.buffer) {
            return self.error(&e);
        }
        self.sync_macro_catalog();
        self.sync_input_context();
        self.input.restore_macro_mode(draft.mode);
        if draft.repeat {
            self.macros.repeat = Some(crate::macros::Repeat::Ring);
        }
        self.message(message)
    }
    pub(in crate::session) fn macro_definition_guard(
        &mut self,
        command: Command,
    ) -> Option<Action> {
        if !self.macro_definition_selected() {
            return None;
        }
        if command == Command::Buffer(crate::command::BufferCommand::Kill) {
            return Some(
                self.macro_leave_definition("Macro draft discarded; prior definition retained"),
            );
        }
        if (!super::supported(command)
            && !matches!(command, Command::Macro(_) | Command::KeyboardQuit))
            || matches!(
                command,
                Command::SaveBuffer
                    | Command::WriteFile
                    | Command::FindFile
                    | Command::OpenDirectory
                    | Command::GenerateNewBuffer
                    | Command::Quit
                    | Command::ContextClose
                    | Command::DashboardOpen
                    | Command::ProjectCreate
                    | Command::ProjectSwitch
                    | Command::WorkspaceOpen
                    | Command::WorkspaceCreate
                    | Command::Buffer(_)
                    | Command::Window(_)
                    | Command::Split(_)
            )
        {
            return Some(self.message(
                "Apply or close the retained macro draft before changing its source owner",
            ));
        }
        None
    }
    pub(in crate::session) fn macro_definition_key(&mut self, keys: &Sequence) -> Action {
        if !self.macro_definition_selected() {
            return self.message("Macro key insertion target changed");
        }
        match Record::key_notation(keys) {
            Ok(text) => self.paste_text(&text),
            Err(e) => self.error(&e.to_string()),
        }
    }
    fn macro_definition_region(&mut self) -> Action {
        let result = (|| -> Result<String, String> {
            let record = self.macro_definition_record()?;
            let buffer = self.buffer().map_err(|e| e.to_string())?;
            let span = buffer
                .region(self.view().point)
                .map_err(|e| e.to_string())?
                .ok_or("Select complete definition lines first")?;
            if span.range.is_empty() {
                return Err("Select complete definition lines first".into());
            }
            for position in [span.range.start, span.range.end] {
                if position > 0
                    && buffer
                        .slice(
                            &buffer
                                .span(position - 1..position)
                                .map_err(|e| e.to_string())?,
                        )
                        .map_err(|e| e.to_string())?
                        != "\n"
                    && position != buffer.len_chars()
                {
                    return Err("Definition selection requires complete lines".into());
                }
            }
            let fragment = buffer.slice(&span).map_err(|e| e.to_string())?.to_string();
            record
                .region_body(&fragment)
                .and_then(|r| r.draft_text())
                .map_err(|e| e.to_string())
        })();
        match result {
            Ok(text) => {
                let len = self.buffer().expect("draft owner").len_chars();
                match self.replace(0..len, &text) {
                    Ok(_) => self
                        .message("Selected definition lines are the draft body; C-c C-c applies"),
                    Err(e) => self.error(&e.to_string()),
                }
            }
            Err(e) => self.error(&e),
        }
    }
    fn macro_edit_lossage(&mut self) -> Action {
        let Some((entry, _)) = self.macros.recent.entries.front() else {
            return self.message("No recent recordable input is available");
        };
        let steps: Vec<_> = self
            .macros
            .recent
            .entries
            .iter()
            .map(|(_, s)| s.clone())
            .collect();
        let definition = Definition {
            retained_bytes: steps
                .iter()
                .map(|s| s.bytes() + std::mem::size_of::<crate::macros::Step>())
                .sum(),
            id: 0,
            revision: 1,
            entry: *entry,
            counter: Default::default(),
            initial_format: self.macros.default_format.clone(),
            steps: steps.into(),
        };
        match self.macros.snapshot(&definition) {
            Ok(source) => self.macro_open_definition(Selection::New(source), false),
            Err(reason) => self.stop_macro(reason, "Recent input exceeds macro admission"),
        }
    }
}
