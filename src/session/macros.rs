mod calls;
mod capture;
mod controls;
mod counters;
pub(in crate::session) mod definition;
pub(in crate::session) mod library;
mod literal;
pub(in crate::session) mod menu;
pub(in crate::session) mod names;
mod queries;
mod recent;
mod regions;
mod repeat;
mod runner;
mod step_edit;

use std::sync::Arc;

use crossterm::event::Event;

use super::{Action, Session};
use crate::command::{Command, MacroCommand, PrefixArgument};
use crate::input::{Focus, Input};
use crate::macros::{Entry, Recorder, Runner, Status, Stop, StopReason};

impl Session {
    pub fn macro_status(&self) -> Status {
        self.macros.status()
    }

    pub fn macro_stop(&self) -> Option<Stop> {
        self.macros.stop
    }

    /// Observe plugin, native layout, consent, query or manual editing ownership.
    /// User waits have no automatic macro wake; the host waits for native input.
    pub fn macro_wait(&self) -> Option<crate::macros::Wait> {
        if self.plugin_macro_waiting() {
            return Some(crate::macros::Wait::Plugin);
        }
        if self
            .macros
            .step_editor
            .as_ref()
            .is_some_and(|e| e.asking || e.input.is_some())
        {
            return Some(crate::macros::Wait::Step);
        }

        if self.macros.runner.is_none() && !self.macros.pauses.is_empty() {
            return Some(crate::macros::Wait::Editing);
        }
        self.macros.runner.as_ref().and_then(|run| {
            if run.query {
                Some(crate::macros::Wait::Query)
            } else if run.consent.is_some() {
                Some(crate::macros::Wait::Consent)
            } else if run.waiting {
                Some(crate::macros::Wait::Layout)
            } else {
                None
            }
        })
    }

    pub(in crate::session) fn macro_entry(&self) -> Entry {
        Entry {
            profile: self.input.interaction_profile(),
            mode: self.input.editing_state(),
        }
    }

    pub(in crate::session) fn macro_source(&self) -> bool {
        self.source_layout_focus()
            && !self.macro_definition_selected()
            && !self.macro_menu_active()
            && self.input.focus() == Focus::Editing
            && !self.terminals.focused
            && self.startup_screen.is_none()
            && !self.git_visible()
            && !self.bookmark_list_active()
            && self.pending.is_none()
            && !self.completion.has_popup()
    }

    pub(super) fn stop_macro(&mut self, reason: StopReason, message: &str) -> Action {
        if self.macros.recent_capturing {
            self.macros.recorder = None;
            return Action::Wait;
        }
        self.cancel_plugin_macro();
        if self.macros.active() {
            self.restore_search();
            self.pending = None;
        }
        self.macros.fail(reason);
        self.input.macro_editing = false;
        self.buffers.automatic_group(None);
        self.input.cancel();
        let detail = self.macros.stop.map_or(String::new(), |stop| {
            if stop.iteration == 0 {
                String::new()
            } else {
                format!(" (iteration {}, step {})", stop.iteration, stop.step)
            }
        });
        self.message(&format!("{message}{detail}"))
    }

    /// Physical input cannot interleave with an automatic step. Resize and
    /// cancellation retain their native owners; the run itself never blocks IO.
    pub(super) fn macro_input_guard(&mut self, event: &Event) -> Option<Action> {
        self.input.macro_editing = self.macro_manual();
        if !self.macros.expanded && self.plugin_macro_waiting() {
            if Input::interrupt(event) {
                self.stop_macro(StopReason::Canceled, "Semantic plugin call canceled");
                return None;
            }
            if matches!(event, Event::Paste(_) | Event::Mouse(_))
                || matches!(event, Event::Key(key) if key.kind != crossterm::event::KeyEventKind::Release)
            {
                return Some(self.message("Semantic plugin call is pending; C-g cancels"));
            }
        }
        self.macro_native_input_guard(event)
    }

    fn macro_native_input_guard(&mut self, event: &Event) -> Option<Action> {
        if !self.macros.expanded && self.macro_manual() {
            if Input::interrupt(event) {
                self.stop_macro(StopReason::Canceled, "Keyboard macro canceled");
                return None;
            }
            return self.macro_manual_guard();
        }
        if self.macros.recorder.is_some() && matches!(event, Event::Mouse(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro recording stopped before pointer input",
            ));
        }
        self.macro_running_input_guard(event)
    }

    fn macro_running_input_guard(&mut self, event: &Event) -> Option<Action> {
        if self.macros.expanded || self.macros.runner.is_none() {
            return None;
        }
        if Input::interrupt(event) {
            self.stop_macro(StopReason::Canceled, "Keyboard macro canceled");
            return None;
        }
        if self
            .macros
            .runner
            .as_ref()
            .is_some_and(|run| run.consent.is_some() || run.query)
        {
            let current = (self.windows.selected_id(), self.view().point);
            if self
                .macros
                .runner
                .as_ref()
                .is_some_and(|run| run.expected != current)
            {
                return Some(
                    self.stop_macro(StopReason::OwnerChanged, "Macro consent source changed"),
                );
            }
            return None;
        }
        if matches!(event, Event::Paste(_) | Event::Mouse(_))
            || matches!(event, Event::Key(key) if key.kind != crossterm::event::KeyEventKind::Release)
        {
            return Some(self.message("Keyboard macro is running; C-g cancels"));
        }
        None
    }

    pub(super) fn macro_command_guard(&mut self, command: Command) -> Option<Action> {
        if command == Command::KeyboardQuit {
            self.macros.repeat = None;
        }
        if !self.macros.active() {
            return None;
        }
        if command != Command::KeyboardQuit && self.plugin_macro_waiting() && !self.macros.expanded
        {
            return Some(self.message("Semantic plugin call is pending; C-g cancels"));
        }
        if command == Command::KeyboardQuit {
            self.stop_macro(StopReason::Canceled, "Keyboard macro canceled");
            return None;
        }
        if self.macro_manual() {
            return self.macro_manual_command_guard(command);
        }
        self.macro_automatic_command_guard(command)
    }

    fn macro_manual_command_guard(&mut self, command: Command) -> Option<Action> {
        if let Some(action) = self.macro_manual_guard() {
            return Some(action);
        }
        if supported(command) || matches!(command, Command::Macro(_)) {
            return None;
        }
        Some(self.stop_macro(
            StopReason::Unsupported,
            "Manual macro editing stopped before unsupported command",
        ))
    }

    fn macro_automatic_command_guard(&mut self, command: Command) -> Option<Action> {
        if matches!(command, Command::MacroNamed(_) | Command::MacroBinding(_)) {
            return None;
        }
        if matches!(command, Command::Macro(_)) && !self.macros.expanded {
            return None;
        }
        if self.macros.expanded {
            return self.macro_expanded_command_guard(command);
        }
        self.macro_dispatch_command_guard(command)
    }

    fn macro_expanded_command_guard(&mut self, command: Command) -> Option<Action> {
        if self.macros.expanded
            && matches!(
                command,
                Command::Macro(
                    MacroCommand::InsertCounter
                        | MacroCommand::SetCounter
                        | MacroCommand::AddCounter
                        | MacroCommand::SetFormat
                        | MacroCommand::Redisplay
                        | MacroCommand::Query
                )
            )
        {
            return None;
        }
        if self.macros.expanded && matches!(command, Command::Completion(_)) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Macro completion requires a captured literal edit",
            ));
        }
        self.macro_dispatch_command_guard(command)
    }

    fn macro_dispatch_command_guard(&mut self, command: Command) -> Option<Action> {
        if (!self.macros.expanded && !self.macros.dispatching) || !supported(command) {
            return Some(self.stop_macro(
                StopReason::Unsupported,
                "Keyboard macro stopped before unsupported command",
            ));
        }
        None
    }

    pub(super) fn macro_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        self.finish_repeat_entry();
        if let Err(error) = self.stop_typing() {
            return self.error(&error.to_string());
        }
        if let Some(action) = self.macro_library_control(command, argument) {
            return action;
        }
        if command == MacroCommand::List {
            return self.macro_menu_open();
        }
        if menu::local_command(command) {
            return self.message("Open the macro menu before using its local commands");
        }
        if let Some(action) = self.macro_definition_control(command) {
            return action;
        }
        if let Some(action) = self.macro_session_control(command, argument) {
            return action;
        }
        self.macro_recording_command(command, argument)
    }

    fn macro_library_control(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Option<Action> {
        match command {
            MacroCommand::Save
            | MacroCommand::Load
            | MacroCommand::DeleteSaved
            | MacroCommand::Export => Some(self.macro_library_command(command, argument)),
            _ => None,
        }
    }

    fn macro_definition_control(&mut self, command: MacroCommand) -> Option<Action> {
        match command {
            MacroCommand::StepEdit => Some(self.macro_step_start()),
            MacroCommand::Edit
            | MacroCommand::EditRepeat
            | MacroCommand::EditNamed
            | MacroCommand::Lossage
            | MacroCommand::FinishEdit
            | MacroCommand::InsertKey
            | MacroCommand::RegionLines => Some(self.macro_definition_command(command)),
            _ => None,
        }
    }

    fn macro_session_control(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Option<Action> {
        let action = match command {
            MacroCommand::Name => self.macro_name_prompt(false),
            MacroCommand::Bind => self.macro_name_prompt(true),
            MacroCommand::Query => self.macro_query_command(argument),
            MacroCommand::Resume => self.macro_resume(),
            MacroCommand::Abort => {
                if self.macro_manual() {
                    self.stop_macro(StopReason::Canceled, "Macro editing aborted")
                } else {
                    self.message("No macro editing pause is active")
                }
            }
            MacroCommand::ApplyLines => self.macro_region_start(),
            _ => return None,
        };
        Some(action)
    }

    fn macro_recording_effect(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Option<Action> {
        if command == MacroCommand::StartOrCounter && self.macros.recorder.is_some() {
            return Some(self.macro_counter_command(MacroCommand::InsertCounter, argument));
        }
        if matches!(
            command,
            MacroCommand::InsertCounter
                | MacroCommand::SetCounter
                | MacroCommand::AddCounter
                | MacroCommand::SetFormat
        ) {
            return Some(self.macro_counter_command(command, argument));
        }
        if command == MacroCommand::Redisplay {
            return Some(self.macro_redisplay());
        }
        None
    }

    fn macro_redisplay(&mut self) -> Action {
        if let Some(recorder) = &mut self.macros.recorder
            && let Err(reason) = recorder
                .stage(crate::macros::Step::Redisplay)
                .and_then(|()| recorder.accept())
        {
            return self.stop_macro(reason, "Keyboard macro recording limit exceeded");
        }
        self.message("")
    }

    fn macro_recording_command(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Action {
        if let Some(action) = self.macro_recording_effect(command, argument) {
            return action;
        }
        if self.macros.expanded || self.macros.runner.is_some() {
            return self.stop_macro(
                StopReason::Unsupported,
                "Nested macro controls are unavailable",
            );
        }
        if matches!(command, MacroCommand::Start | MacroCommand::StartOrCounter) {
            return self.macro_start_recording(argument);
        }
        if let Some(action) = self.macro_ring_control(command, argument) {
            return action;
        }
        self.macro_end_recording(command, argument)
    }

    fn macro_ring_control(
        &mut self,
        command: MacroCommand,
        argument: PrefixArgument,
    ) -> Option<Action> {
        if matches!(
            command,
            MacroCommand::Next
                | MacroCommand::Previous
                | MacroCommand::Delete
                | MacroCommand::Swap
                | MacroCommand::CallSecond
        ) {
            return Some(self.macro_ring_command(command, argument));
        }
        None
    }

    fn macro_publish_recording(&mut self, command: MacroCommand) -> Result<bool, Action> {
        let recorded = self.macros.recorder.is_some();
        if let Some(recorder) = self.macros.recorder.take() {
            if recorder.steps.len() == recorder.original_len {
                self.macros.stop = Some(Stop {
                    reason: StopReason::Empty,
                    iteration: 0,
                    step: 0,
                });
                return Err(self.message("Empty recording discarded; previous macro retained"));
            }
            let limit = self.buffers.snapshot().settings().macros.ring_limit;
            if let Err(reason) = self.macros.publish(recorder, limit) {
                return Err(
                    self.stop_macro(reason, "Keyboard macro definition revision limit reached")
                );
            }
        } else if command == MacroCommand::End {
            return Err(self.message("No keyboard macro is being recorded"));
        }
        Ok(recorded)
    }

    fn macro_end_recording(&mut self, command: MacroCommand, argument: PrefixArgument) -> Action {
        let recorded = match self.macro_publish_recording(command) {
            Ok(recorded) => recorded,
            Err(action) => return action,
        };
        let count = recorded_count(command, argument, recorded);
        self.macros.repeat = match command {
            MacroCommand::EndAndCall => Some(crate::macros::Repeat::EndCall),
            MacroCommand::CallRepeat => Some(crate::macros::Repeat::Ring),
            _ => None,
        };
        if count == 0
            && recorded
            && (command == MacroCommand::EndOrCall || !argument.supplied() || argument.value() > 0)
        {
            return self.message("Keyboard macro recorded");
        }
        self.start_macro(count)
    }

    fn start_macro(&mut self, count: i32) -> Action {
        let Some(definition) = self.macros.ring.front().cloned() else {
            return self.message("No keyboard macro is available");
        };
        self.start_definition(definition, count, None)
    }

    fn start_definition(
        &mut self,
        definition: Arc<crate::macros::Definition>,
        count: i32,
        name: Option<crate::command::MacroName>,
    ) -> Action {
        if let Err(reason) = self.macro_call_admission(name) {
            return self.stop_macro(
                reason,
                "Macro call cycle, depth or retained-memory limit reached",
            );
        }
        if definition.steps.is_empty() {
            return self.stop_macro(StopReason::Empty, "Empty keyboard macro cannot run");
        }
        if !self.macro_source() || definition.entry != self.macro_entry() {
            if self.macros.active() {
                return self.stop_macro(
                    StopReason::EntryState,
                    "Called macro entry profile or source state differs",
                );
            }
            self.macros.stop = Some(Stop {
                reason: StopReason::EntryState,
                iteration: 1,
                step: 0,
            });
            return self.message("Keyboard macro entry profile or source state differs");
        }
        let ancestor = self.macros.runner.as_ref().or_else(|| {
            self.macros
                .pauses
                .iter()
                .rev()
                .find_map(|pause| pause.runner.as_ref())
        });
        let limits = ancestor
            .map(|run| Arc::clone(&run.limits))
            .unwrap_or_else(|| Arc::new(crate::macros::Limits::new()));
        if !limits.iteration() {
            return self.stop_macro(
                StopReason::IterationBudget,
                "Keyboard macro iteration limit reached",
            );
        }
        limits.resume();
        let budget = Arc::clone(&limits.text);
        let outer_group = ancestor.map(|run| Arc::clone(&run.group));
        let group = outer_group
            .clone()
            .unwrap_or_else(|| crate::buffer::edit::AutomaticGroup::new(Arc::clone(&budget)));
        self.input.macro_editing = false;
        self.macros.stop = None;
        let mut counter = definition.counter.clone();
        counter.iteration_start = counter.current;
        counter.format = definition.initial_format.clone();
        let parent = self.macros.runner.take().map(Box::new);
        self.macros.runner = Some(Runner {
            name,
            parent,
            counter,
            append: false,
            definition,
            iterations: usize::try_from(count).ok().filter(|count| *count > 0),
            iteration: 0,
            step: 0,
            limits,
            query: false,
            region: None,
            expected: (self.windows.selected_id(), self.view().point),
            waiting: false,
            group,
            outer_group,
            budget,
            arguments: Default::default(),
            trigger: Default::default(),
            consent: None,
        });
        self.message("Keyboard macro playing")
    }
}

#[cfg(test)]
mod tests;

fn supported(command: Command) -> bool {
    if matches!(
        command,
        Command::Completion(
            crate::command::CompletionCommand::Mode | crate::command::CompletionCommand::AutoMode
        )
    ) {
        return false;
    }
    matches!(
        command,
        Command::MacroNamed(_)
            | Command::PluginDefined(_)
            | Command::MacroBinding(_)
            | Command::Motion(_)
            | Command::Navigate(_)
            | Command::Edit(_)
            | Command::Modal(_)
            | Command::Scroll(_)
            | Command::History(_)
            | Command::Region(_)
            | Command::Rectangle(_)
            | Command::Kill(_)
            | Command::Linear(_)
            | Command::Window(_)
            | Command::Split(_)
            | Command::Newline
            | Command::InsertTab
            | Command::BackwardDeleteChar
            | Command::DeleteChar
            | Command::PrefixArgument
            | Command::UniversalArgument
            | Command::DigitArgument
            | Command::NegativeArgument
            | Command::ExecuteExtended
            | Command::SearchForward
            | Command::SearchBackward
            | Command::QueryReplace
            | Command::FindFile
            | Command::GotoLine
            | Command::SaveBuffer
            | Command::WriteFile
            | Command::Buffer(_)
            | Command::GenerateNewBuffer
            | Command::Completion(_)
    )
}

fn recorded_count(command: MacroCommand, argument: PrefixArgument, recorded: bool) -> i32 {
    if recorded {
        match command {
            MacroCommand::EndOrCall => 0,
            MacroCommand::End if argument.value() > 0 => {
                if argument.supplied() {
                    argument.value() - 1
                } else {
                    0
                }
            }
            _ => argument.value(),
        }
    } else {
        argument.value()
    }
}
