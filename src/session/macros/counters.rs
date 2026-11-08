use super::*;
use crate::command::ArgumentForm;
use crate::macros::{
    Step,
    counter::{self, Operation},
};
use crate::session::Pending;

impl Session {
    pub(super) fn macro_counter_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        if !self.macro_source() {
            return self.message("Macro counters require ordinary source editing");
        }
        let plain = matches!(argument.form(), ArgumentForm::PlainPrefix { .. });
        let operation = match command {
            MacroCommand::InsertCounter => Operation::Insert(argument),
            MacroCommand::SetCounter if plain => Operation::Reset,
            MacroCommand::SetCounter if argument.supplied() => {
                Operation::Set(i64::from(argument.value()))
            }
            MacroCommand::AddCounter if plain => Operation::Restore,
            MacroCommand::AddCounter if argument.supplied() => {
                Operation::Add(i64::from(argument.value()))
            }
            _ => {
                if self.macros.expanded {
                    return self.stop_macro(
                        StopReason::PromptMismatch,
                        "Macro counter prompt has no captured value",
                    );
                }
                let label = match command {
                    MacroCommand::SetFormat => "Macro counter format: ",
                    MacroCommand::SetCounter => "Set macro counter: ",
                    _ => "Add to macro counter: ",
                };
                if let Err(error) = self.input.argument(label, "") {
                    return self.error(&error.to_string());
                }
                self.pending = Some(Pending::MacroCounter {
                    command,
                    view: self.windows.selected_id(),
                    point: self.view().point,
                });
                return self.message("");
            }
        };
        self.macro_counter_operation(operation)
    }

    pub(in crate::session) fn macro_counter_argument(
        &mut self,
        command: MacroCommand,
        view: crate::window::ViewId,
        point: crate::buffer::edit::Point,
        value: &str,
    ) -> Action {
        if view != self.windows.selected_id() || point != self.view().point {
            return self.stop_macro(
                StopReason::OwnerChanged,
                "Macro counter prompt source changed",
            );
        }
        let operation = if command == MacroCommand::SetFormat {
            let format = if value.is_empty() { "%d" } else { value };
            if let Err(error) = counter::render(format, i64::MIN) {
                return self.message(error);
            }
            Operation::Format(format.into())
        } else {
            let Ok(number) = value.parse::<i64>() else {
                return self.message("Macro counter requires a signed 64-bit integer");
            };
            if command == MacroCommand::SetCounter {
                Operation::Set(number)
            } else {
                Operation::Add(number)
            }
        };
        self.macro_counter_operation(operation)
    }

    pub(super) fn macro_counter_operation(&mut self, operation: Operation) -> Action {
        if !self.macro_source() {
            return self.stop_macro(
                StopReason::EntryState,
                "Macro counter source is unavailable",
            );
        }
        let mut state = self.macros.counter().clone();
        let mut insertion = None;
        match &operation {
            Operation::Insert(argument) => {
                let plain = matches!(argument.form(), ArgumentForm::PlainPrefix { .. });
                let value = if plain { state.previous } else { state.current };
                if !plain {
                    let Some(next) = state.current.checked_add(i64::from(argument.value())) else {
                        return self
                            .stop_macro(StopReason::CounterOverflow, "Macro counter overflow");
                    };
                    state.previous = state.current;
                    state.current = next;
                }
                match counter::render(&state.format, value) {
                    Ok(text) => insertion = Some((value, text)),
                    Err(error) => return self.message(error),
                }
            }
            Operation::Set(value) => state.current = *value,
            Operation::Reset => state.current = state.iteration_start,
            Operation::Add(value) => {
                let Some(next) = state.current.checked_add(*value) else {
                    return self.stop_macro(StopReason::CounterOverflow, "Macro counter overflow");
                };
                state.current = next;
            }
            Operation::Restore => state.current = self.macros.last_inserted,
            Operation::Format(format) => {
                if let Err(error) = counter::render(format, i64::MIN) {
                    return self.message(error);
                }
                state.format = format.clone();
            }
        }
        if !self.macros.expanded
            && let Some(recorder) = &mut self.macros.recorder
            && let Err(reason) = recorder
                .stage(Step::Counter(operation.clone()))
                .and_then(|()| recorder.accept())
        {
            return self.stop_macro(reason, "Keyboard macro recording limit exceeded");
        }
        if let Some((value, text)) = insertion {
            if !text.is_empty() {
                self.timestamp_repeat = None;
                self.yank = None;
                self.kills.break_chain();
                if let Err(error) = self.stop_typing() {
                    return self.error(&error.to_string());
                }
                let result = self.paste_entry(&text);
                if let Err(error) = result {
                    return self.error(&error.to_string());
                }
                self.windows.selected_mut().reset_navigation();
                self.finish(result);
            }
            self.macros.last_inserted = value;
        }
        if let Operation::Format(format) = &operation
            && !self.macros.active()
        {
            self.macros.default_format = format.clone();
            return self.message("Macro counter format set for new recordings");
        }
        if !self.macros.expanded {
            self.macros
                .recent
                .push(self.macro_entry(), Step::Counter(operation));
        }
        self.macros.set_counter(state);
        self.message("Macro counter updated")
    }
}
