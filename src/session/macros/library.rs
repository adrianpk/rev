//! Native explicit library prompts; persistence never runs from recording/replay.
use super::*;
use crate::buffer::edit::Point;
use crate::command::MacroName;
use crate::input::keymap::Sequence;
use crate::macros::library::{Record, Saved, Store};
use crate::macros::{Named, Target};
use crate::session::Pending;
use crate::window::ViewId;

pub(in crate::session) struct Prompt {
    command: MacroCommand,
    binding: bool,
    generation: u64,
    view: ViewId,
    point: Point,
}
pub(in crate::session) struct Consent {
    prompt: Prompt,
    name: MacroName,
    definition: Option<Arc<crate::macros::Definition>>,
    deletion: Option<(String, u64)>,
    omit_binding: bool,
}
impl Consent {
    pub(in crate::session) fn question(&self) -> &'static str {
        if self.prompt.command == MacroCommand::DeleteSaved {
            "Delete saved macro? [y/n]"
        } else if self.omit_binding {
            "Save macro without its previously saved binding? [y/n]"
        } else {
            "Save named macro? [y/n]"
        }
    }
}
impl Session {
    /// Attach the fixed library for an explicit root snapshot. Loading is read
    /// only; isolated new/empty embeddings never discover the user's files.
    pub fn with_macro_library(mut self, roots: &crate::config::Roots) -> Self {
        match Store::at_state(roots) {
            Ok(store) => {
                self.macros.library = Some(store);
                let action = self.reload_macro_library();
                let _ = action;
            }
            Err(error) => {
                self.notice(&error.to_string());
            }
        }
        self
    }
    pub(in crate::session) fn startup_macro_library(&mut self) -> Option<String> {
        match crate::config::Roots::from_env()
            .map_err(|e| e.to_string())
            .and_then(|roots| Store::at_state(&roots).map_err(|e| e.to_string()))
        {
            Ok(store) => {
                self.macros.library = Some(store);
                self.admit_macro_library().err()
            }
            Err(error) => Some(format!("Macro library: {error}")),
        }
    }
    pub(super) fn macro_library_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        if self.macros.active() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Library commands require an idle macro session",
            );
        }
        if command == MacroCommand::Load {
            return self.reload_macro_library();
        }
        if !self.macro_source() {
            return self.message("Macro library commands require ordinary source editing");
        }
        if command != MacroCommand::Export && self.macros.library.is_none() {
            return self.message("Macro library has no attached state root");
        }
        let prompt = Prompt {
            command,
            binding: argument.supplied(),
            generation: self.macros.catalog_revision,
            view: self.windows.selected_id(),
            point: self.view().point,
        };
        if let Err(error) = self.input.argument("Named macro: ", "") {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::MacroLibrary(Box::new(prompt)));
        self.message("")
    }
    fn macro_library_current(&self, prompt: &Prompt) -> bool {
        prompt.generation == self.macros.catalog_revision
            && prompt.view == self.windows.selected_id()
            && prompt.point == self.view().point
            && self.macro_source()
    }
    fn named_macro_binding(&self, name: MacroName) -> Option<Sequence> {
        self.macros.bindings.iter().find_map(|(key, target)| {
            matches!(target,Target::Named(n) if *n==name).then(|| key.clone())
        })
    }
    pub(in crate::session) fn macro_library_argument(
        &mut self,
        prompt: Prompt,
        value: &str,
    ) -> Action {
        if !self.macro_library_current(&prompt) {
            return self.message("Macro library source or catalog changed");
        }
        let parsed = if prompt.command == MacroCommand::DeleteSaved {
            MacroName::stored_name(value)
        } else {
            MacroName::new(value)
        };
        let name = match parsed {
            Ok(n) => n,
            Err(e) => return self.error(&e),
        };
        if prompt.command == MacroCommand::DeleteSaved {
            let catalog = match self
                .macros
                .library
                .as_ref()
                .expect("attached library")
                .load()
            {
                Ok(c) => c,
                Err(e) => return self.error(&e.to_string()),
            };
            let Some(record) = catalog.macros.iter().find(|r| r.name == value) else {
                return self.message("No saved macro has that name");
            };
            let expected = self
                .macros
                .names
                .get(&name)
                .and_then(|n| n.saved.as_ref())
                .map(|s| (s.id.clone(), s.revision))
                .unwrap_or_else(|| (record.id.clone(), record.revision));
            let consent = Consent {
                prompt,
                name,
                definition: None,
                deletion: Some(expected),
                omit_binding: false,
            };
            return self.macro_library_confirm(consent);
        }
        let Some(named) = self.macros.names.get(&name) else {
            return self.message("No session macro has that name");
        };
        if prompt.command == MacroCommand::Export {
            let binding = prompt
                .binding
                .then(|| self.named_macro_binding(name))
                .flatten();
            let text = match Record::capture(name, named, binding.as_ref()).and_then(|r| r.export())
            {
                Ok(t) => t,
                Err(e) => return self.error(&e.to_string()),
            };
            return self.paste_text(&text);
        }
        let consent = Consent {
            omit_binding: !prompt.binding
                && named.saved.as_ref().is_some_and(|s| s.binding.is_some()),
            prompt,
            name,
            definition: Some(Arc::clone(&named.definition)),
            deletion: None,
        };
        self.macro_library_confirm(consent)
    }
    fn macro_library_confirm(&mut self, consent: Consent) -> Action {
        if let Err(e) = self.input.decision(consent.question()) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::MacroLibraryConsent(Box::new(consent)));
        self.focus_message("The saved library changes only after confirmation")
    }
    pub(in crate::session) fn macro_library_answer(
        &mut self,
        consent: &Consent,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Macro library change canceled");
        }
        if !self.macro_library_current(&consent.prompt) {
            return self.message("Macro library source or catalog changed");
        }
        if self.macros.library.is_none() {
            return self.message("Macro library is unavailable");
        }
        if let Some((id, revision)) = &consent.deletion {
            return self.macro_library_delete(consent, id, *revision);
        }
        self.macro_library_save(consent)
    }

    fn macro_library_delete(&mut self, consent: &Consent, id: &str, revision: u64) -> Action {
        let store = self
            .macros
            .library
            .as_ref()
            .expect("admitted macro library");
        let publication = match store.delete(consent.name.name(), id, revision) {
            Ok(p) => p,
            Err(e) => return self.error(&e.to_string()),
        };
        if let Some(named) = self.macros.names.get_mut(&consent.name) {
            named.saved = None;
        }
        self.message(if publication.warning {
            "Saved macro deleted; durability could not be confirmed"
        } else {
            "Saved macro deleted; session snapshots remain available"
        })
    }

    fn macro_library_save(&mut self, consent: &Consent) -> Action {
        let store = self
            .macros
            .library
            .as_ref()
            .expect("admitted macro library");
        let Some(named) = self.macros.names.get(&consent.name) else {
            return self.message("Named macro disappeared");
        };
        if !consent
            .definition
            .as_ref()
            .is_some_and(|d| Arc::ptr_eq(d, &named.definition))
        {
            return self.message("Named macro changed before save");
        }
        let active_binding = self.named_macro_binding(consent.name);
        let included = consent
            .prompt
            .binding
            .then_some(active_binding.as_ref())
            .flatten();
        let record = match Record::capture(consent.name, named, included) {
            Ok(r) => r,
            Err(e) => return self.error(&e.to_string()),
        };
        let publication = match store.save(record, named.saved.as_ref()) {
            Ok(p) => p,
            Err(e) => return self.error(&e.to_string()),
        };
        let record = publication.record.expect("save publishes one record");
        let named = self
            .macros
            .names
            .get_mut(&consent.name)
            .expect("checked named owner");
        named.saved = Some(Saved {
            id: record.id,
            revision: record.revision,
            baseline: Arc::clone(&named.definition),
            description: named.description.clone(),
            binding: record
                .binding
                .as_ref()
                .and_then(|s| Sequence::parse(s).ok()),
            active_binding,
        });
        self.message(if publication.warning {"Macro saved; durability could not be confirmed; this saved identity/revision is retained"} else {"Named macro saved"})
    }
    fn reload_macro_library(&mut self) -> Action {
        match self.admit_macro_library() {
            Ok(()) => self.message("Saved macros loaded without execution"),
            Err(e) => self.error(&e),
        }
    }
    fn admit_macro_library(&mut self) -> Result<(), String> {
        if self.macros.active() {
            return Err("Macro library reload requires an idle macro session".into());
        }
        let catalog = self
            .macros
            .library
            .as_ref()
            .ok_or("Macro library has no attached state root")?
            .load()
            .map_err(|e| e.to_string())?;
        let mut diagnostics = Vec::new();
        let present: std::collections::BTreeSet<_> =
            catalog.macros.iter().map(|r| r.id.clone()).collect();
        // The whole aggregate has passed structural validation before any name
        // or binding is admitted. Independent collisions quarantine one record.
        for record in catalog.macros {
            self.admit_macro_library_record(record, &mut diagnostics)?;
        }
        for (name, named) in &self.macros.names {
            if named
                .saved
                .as_ref()
                .is_some_and(|s| !present.contains(&s.id))
            {
                diagnostics.push(format!(
                    "{}: saved record removed; session snapshot preserved",
                    name.name()
                ));
            }
        }
        // Bounded by the 256-record store and 64-byte native names.
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(format!("Macro library: {}", diagnostics.join("; ")))
        }
    }

    fn admit_macro_library_record(
        &mut self,
        record: Record,
        diagnostics: &mut Vec<String>,
    ) -> Result<(), String> {
        let name = match MacroName::new(&record.name) {
            Ok(n) => n,
            Err(_) => {
                diagnostics.push(format!("{}: command name collision", record.name));
                return Ok(());
            }
        };
        if self.macro_library_conflict(name, &record) {
            diagnostics.push(format!(
                "{}: unsaved or conflicting session definition preserved",
                name.name()
            ));
            return Ok(());
        }
        let source = record.definition(0).map_err(|e| e.to_string())?;
        let definition = match self.macros.snapshot(&source) {
            Ok(d) => d,
            Err(_) => {
                diagnostics.push(format!("{}: retained memory limit", name.name()));
                return Ok(());
            }
        };
        let (stored_binding, binding) = self.macro_library_bindings(&record, name);
        if stored_binding.is_some() && binding.is_none() {
            diagnostics.push(format!(
                "{}: saved binding inactive because it conflicts",
                name.name()
            ));
        }
        let saved = Saved {
            id: record.id,
            revision: record.revision,
            baseline: Arc::clone(&definition),
            description: record.description.clone(),
            binding: stored_binding,
            active_binding: binding.clone(),
        };
        let named = Named {
            source: (definition.id, definition.revision),
            definition,
            description: record.description,
            saved: Some(saved),
        };
        if let Err(e) = self.macros.name_snapshot(name, named) {
            diagnostics.push(format!("{}: {e}", name.name()));
            return Ok(());
        }
        self.macros
            .bindings
            .retain(|_, target| !matches!(target,Target::Named(n) if *n==name));
        if let Some(key) = binding {
            self.macros.bindings.insert(key, Target::Named(name));
        }
        self.sync_macro_catalog();
        Ok(())
    }

    fn macro_library_conflict(&self, name: MacroName, record: &Record) -> bool {
        let active = self.named_macro_binding(name);
        let stale = self.macros.names.get(&name).is_some_and(|n| {
            n.saved
                .as_ref()
                .is_none_or(|s| s.id != record.id || !s.clean(n, active.as_ref()))
        });
        let identity_collision = self
            .macros
            .names
            .iter()
            .any(|(n, v)| *n != name && v.saved.as_ref().is_some_and(|s| s.id == record.id));
        let command_collision = self
            .input
            .command_named(name.name())
            .is_some_and(|c| !matches!(c, Command::MacroNamed(_)));
        stale || identity_collision || command_collision
    }

    fn macro_library_bindings(
        &self,
        record: &Record,
        name: MacroName,
    ) -> (Option<Sequence>, Option<Sequence>) {
        let stored_binding = record
            .binding
            .as_ref()
            .map(|s| Sequence::parse(s).expect("structurally validated binding"));
        let binding = stored_binding
            .as_ref()
            .filter(|key| {
                self.input
                    .macro_library_binding_check(key, name)
                    .is_ok_and(|occupied| !occupied)
                    && (self.macros.bindings.len() < crate::macros::NAME_LIMIT
                        || self
                            .macros
                            .bindings
                            .values()
                            .any(|target| matches!(target, Target::Named(n) if *n == name)))
            })
            .cloned();
        (stored_binding, binding)
    }
}
