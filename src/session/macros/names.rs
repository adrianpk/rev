use super::*;
use crate::buffer::edit::Point;
use crate::command::MacroName;
use crate::input::keymap::Sequence;
use crate::macros::{NAME_LIMIT, Named, Target};
use crate::window::ViewId;

pub(in crate::session) struct Draft {
    source: Arc<crate::macros::Definition>,
    view: ViewId,
    point: Point,
    generation: u64,
}
pub(in crate::session) enum Consent {
    Name(Draft, MacroName),
    Binding(Draft, Sequence),
}

impl Consent {
    pub(in crate::session) fn question(&self) -> &'static str {
        match self {
            Self::Name(_, _) => "Replace named macro? [y/n]",
            Self::Binding(_, _) => "Replace occupied source binding? [y/n]",
        }
    }
}

impl Session {
    pub(super) fn macro_name_prompt(&mut self, binding: bool) -> Action {
        if self.macros.recorder.is_some() || self.macros.runner.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Naming and binding require an idle macro or manual pause",
            );
        }
        if !self.macro_source() {
            return self.message("Macro naming requires ordinary source editing");
        }
        let Some(source) = self.macros.ring.front().cloned() else {
            return self.message("No keyboard macro is available");
        };
        let draft = Draft {
            source,
            view: self.windows.selected_id(),
            point: self.view().point,
            generation: self.macros.catalog_revision,
        };
        let result = if binding {
            self.input.macro_key_capture()
        } else {
            self.input.argument("Macro command name: ", "")
        };
        if let Err(error) = result {
            return self.error(&error.to_string());
        }
        self.pending = Some(if binding {
            super::super::Pending::MacroBind(Box::new(draft))
        } else {
            super::super::Pending::MacroName(Box::new(draft))
        });
        self.message("")
    }

    fn macro_draft_current(&self, draft: &Draft) -> bool {
        draft.view == self.windows.selected_id()
            && draft.point == self.view().point
            && draft.generation == self.macros.catalog_revision
    }

    pub(in crate::session) fn macro_name_argument(&mut self, draft: Draft, value: &str) -> Action {
        if !self.macro_draft_current(&draft) {
            return self.message("Macro naming source or catalog changed");
        }
        let name = match MacroName::new(value) {
            Ok(name) => name,
            Err(error) => return self.error(&error),
        };
        if self
            .input
            .command_named(name.name())
            .is_some_and(|command| !matches!(command, Command::MacroNamed(_)))
        {
            return self.error("Macro name collides with an admitted dynamic command");
        }
        if self.macros.names.contains_key(&name) {
            if let Err(error) = self.input.decision("Replace named macro? [y/n]") {
                return self.error(&error.to_string());
            }
            self.pending = Some(super::super::Pending::MacroConsent(Box::new(
                Consent::Name(draft, name),
            )));
            return self.focus_message("Bindings keep the same command name");
        }
        self.macro_publish_name(draft, name)
    }

    fn macro_publish_name(&mut self, draft: Draft, name: MacroName) -> Action {
        if !self.macro_draft_current(&draft) {
            return self.message("Macro naming source or catalog changed");
        }
        let definition = match self.macros.snapshot(&draft.source) {
            Ok(value) => value,
            Err(reason) => return self.stop_macro(reason, "Macro snapshot admission failed"),
        };
        let named = Named {
            saved: self.macros.names.get(&name).and_then(|n| n.saved.clone()),
            definition,
            source: (draft.source.id, draft.source.revision),
            description: self
                .macros
                .names
                .get(&name)
                .and_then(|named| named.description.clone()),
        };
        if let Err(error) = self.macros.name_snapshot(name, named) {
            return self.error(&error);
        }
        self.sync_macro_catalog();
        self.message(&format!("Session macro named {}", name.name()))
    }

    pub(in crate::session) fn macro_key_argument(&mut self, sequence: Sequence) -> Action {
        let Some(super::super::Pending::MacroBind(draft)) = self.pending.take() else {
            return self.message("No macro binding is pending");
        };
        if !self.macro_draft_current(&draft) {
            return self.message("Macro binding source or catalog changed");
        }
        match self.input.macro_binding_check(&sequence) {
            Err(error) => self.error(&error),
            Ok(true) => {
                if let Err(error) = self
                    .input
                    .decision("Replace occupied source binding? [y/n]")
                {
                    return self.error(&error.to_string());
                }
                self.pending = Some(super::super::Pending::MacroConsent(Box::new(
                    Consent::Binding(*draft, sequence),
                )));
                self.focus_message("Only the session source layer changes")
            }
            Ok(false) => self.macro_publish_binding(*draft, sequence),
        }
    }

    fn macro_publish_binding(&mut self, draft: Draft, sequence: Sequence) -> Action {
        if !self.macro_draft_current(&draft) {
            return self.message("Macro binding source or catalog changed");
        }
        if let Err(reason) = self.macros.admit_memory(None) {
            return self.stop_macro(reason, "Macro retained memory exceeds 48 MiB");
        }
        if let Err(error) = self.input.macro_binding_check(&sequence) {
            return self.error(&error);
        }
        if !self.macros.bindings.contains_key(&sequence) && self.macros.bindings.len() == NAME_LIMIT
        {
            return self.error("Session macro bindings exceed 256 records");
        }
        let target = self
            .macros
            .names
            .iter()
            .filter(|(_, named)| named.source == (draft.source.id, draft.source.revision))
            .max_by_key(|(_, named)| named.definition.id)
            .map(|(name, _)| Target::Named(*name));
        let target = match target {
            Some(target) => target,
            None => match self.macros.snapshot(&draft.source) {
                Ok(definition) => Target::Snapshot(definition),
                Err(reason) => {
                    return self.stop_macro(reason, "Macro binding snapshot admission failed");
                }
            },
        };
        let Some(revision) = self.macros.catalog_revision.checked_add(1) else {
            return self.error("Macro catalog revision exhausted");
        };
        self.macros.bindings.retain(|key, previous| {
            !crate::input::macro_bindings::overlaps(key, &sequence)
                && !matches!((previous, &target), (Target::Named(a), Target::Named(b)) if a == b)
        });
        self.macros.bindings.insert(sequence.clone(), target);
        self.macros.catalog_revision = revision;
        self.sync_macro_catalog();
        self.message(&format!("Session macro bound to {sequence}"))
    }

    pub(in crate::session) fn macro_name_consent(
        &mut self,
        consent: Consent,
        accepted: bool,
    ) -> Action {
        if !accepted {
            return self.message("Session macro change canceled");
        }
        match consent {
            Consent::Name(draft, name) => self.macro_publish_name(draft, name),
            Consent::Binding(draft, sequence) => self.macro_publish_binding(draft, sequence),
        }
    }

    pub(in crate::session) fn sync_macro_catalog(&mut self) {
        let names = self
            .macros
            .names
            .iter()
            .map(|(name, named)| (*name, named.description.clone()))
            .collect();
        let bindings = self
            .macros
            .bindings
            .iter()
            .map(|(key, target)| {
                (
                    key.clone(),
                    match target {
                        Target::Named(name) => Command::MacroNamed(*name),
                        Target::Snapshot(definition) => Command::MacroBinding(definition.id),
                    },
                )
            })
            .collect();
        self.input.macro_catalog(names, bindings);
    }
}
