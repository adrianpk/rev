use super::*;
use crate::command::MacroName;
use crate::macros::{CALL_LIMIT, Target};

impl Session {
    pub(in crate::session) fn macro_named_call(
        &mut self,
        name: MacroName,
        argument: PrefixArgument,
    ) -> Action {
        let Some(definition) = self
            .macros
            .names
            .get(&name)
            .map(|named| Arc::clone(&named.definition))
        else {
            return self.stop_macro(StopReason::MissingCommand, "Named macro is unavailable");
        };
        self.start_definition(definition, argument.value(), Some(name))
    }

    pub(in crate::session) fn macro_bound_call(
        &mut self,
        id: u64,
        argument: PrefixArgument,
    ) -> Action {
        let definition = self
            .macros
            .bindings
            .values()
            .find_map(|target| match target {
                Target::Snapshot(definition) if definition.id == id => Some(Arc::clone(definition)),
                _ => None,
            });
        let Some(definition) = definition else {
            return self.stop_macro(StopReason::MissingCommand, "Bound macro is unavailable");
        };
        self.start_definition(definition, argument.value(), None)
    }

    pub(super) fn macro_call_admission(&self, name: Option<MacroName>) -> Result<(), StopReason> {
        let mut depth = 0;
        for root in self.macros.runner.iter().chain(
            self.macros
                .pauses
                .iter()
                .filter_map(|pause| pause.runner.as_ref()),
        ) {
            let mut current = Some(root);
            while let Some(run) = current {
                depth += 1;
                if name.is_some() && run.name == name {
                    return Err(StopReason::CallCycle);
                }
                current = run.parent.as_deref();
            }
        }
        if depth >= CALL_LIMIT {
            return Err(StopReason::CallDepth);
        }
        self.macros.admit_memory(None)
    }
}
