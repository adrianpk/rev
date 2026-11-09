use crate::command::Command;
use crate::input::{Focus, Invocation};
use crate::macros::{Argument, Role, Step, StopReason};
use crate::session::{Action, Pending, Session};
use crossterm::event::{Event, KeyEvent};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl Session {
    /// One finite host turn; native layout completes before the next step.
    /// Consent gives input back to its native owner without an automatic wake.
    pub(in crate::session) fn expire_macro(&mut self) -> Action {
        self.macro_finish_capture();
        if self.plugin_macro_waiting() {
            if self
                .macros
                .runner
                .as_ref()
                .is_some_and(|run| run.limits.elapsed() >= Duration::from_secs(60))
            {
                return self.stop_macro(
                    StopReason::TimeBudget,
                    "Keyboard macro active time limit reached",
                );
            }
            return Action::Wait;
        }
        if self.macros.runner.is_none()
            || self.layout_active()
            || self
                .macros
                .runner
                .as_ref()
                .is_some_and(|run| run.consent.is_some() || run.query)
        {
            return Action::Wait;
        }
        let started = Instant::now();
        for _ in 0..256 {
            let point = (self.windows.selected_id(), self.view().point);
            if let Some(run) = &mut self.macros.runner
                && run.waiting
            {
                run.expected = point;
                run.waiting = false;
            }
            if let Some(action) = self.macro_step_boundary() {
                return action;
            }
            let entry = self.macro_entry();
            let run = self.macros.runner.as_mut().expect("live runner");
            if point != run.expected {
                return self.stop_macro(
                    StopReason::OwnerChanged,
                    "Keyboard macro source owner changed",
                );
            }
            if run.limits.elapsed() >= Duration::from_secs(60) {
                return self.stop_macro(
                    StopReason::TimeBudget,
                    "Keyboard macro active time limit reached",
                );
            }
            let completed = run.step == run.definition.steps.len()
                && run.trigger.is_empty()
                && run.arguments.is_empty()
                && run.iterations == Some(run.iteration + 1)
                && self.pending.is_none()
                && self.input.focus() == Focus::Editing;
            if run.limits.steps.load(Ordering::Relaxed) >= crate::macros::RUN_STEPS && !completed {
                return self.stop_macro(
                    StopReason::StepBudget,
                    "Keyboard macro logical step limit reached",
                );
            }
            let trigger = run.trigger.pop_front();
            let argument = if trigger.is_none() {
                run.arguments.pop_front()
            } else {
                None
            };
            let step = if argument.is_none() && trigger.is_none() {
                if self.macro_consent_pending() {
                    let run = self.macros.runner.as_mut().expect("live runner");
                    run.consent = Some(Instant::now());
                    run.limits.pause();
                    return Action::Redraw;
                }
                if self.pending.is_some() || self.input.focus() != Focus::Editing {
                    return self.stop_macro(
                        StopReason::PromptMismatch,
                        "Macro prompt has no matching captured argument",
                    );
                }
                let run = self.macros.runner.as_mut().expect("live runner");
                if run.step == run.definition.steps.len() {
                    if run.region.is_some() {
                        return self.macro_region_next();
                    }
                    run.iteration += 1;
                    if Some(run.iteration) == run.iterations {
                        return self.macro_run_finished();
                    }
                    if entry != run.definition.entry {
                        return self.stop_macro(
                            StopReason::EntryState,
                            "Keyboard macro did not restore its entry state for another iteration",
                        );
                    }
                    if !run.limits.iteration() {
                        return self.stop_macro(
                            StopReason::IterationBudget,
                            "Keyboard macro iteration limit reached",
                        );
                    }
                    run.step = 0;
                    run.counter.iteration_start = run.counter.current;
                    run.counter.format = run.definition.initial_format.clone();
                    run.group = run.outer_group.clone().unwrap_or_else(|| {
                        crate::buffer::edit::AutomaticGroup::new(run.budget.clone())
                    });
                }
                if run.parent.is_none()
                    && let Some(editor) = &mut self.macros.step_editor
                    && !editor.capturing
                {
                    editor.allowance = editor.allowance.saturating_sub(1);
                }
                let step = run.definition.steps[run.step].clone();
                run.step += 1;
                Some(step)
            } else {
                None
            };
            let refresh = matches!(step, Some(Step::Redisplay));
            let run = self.macros.runner.as_mut().expect("live runner");
            run.limits.steps.fetch_add(1, Ordering::Relaxed);
            self.buffers.automatic_group(Some(run.group.clone()));
            self.macros.expanded = true;
            if let Some(trigger) = trigger {
                self.macro_replay_trigger(trigger);
            } else if let Some(argument) = argument {
                self.macro_replay_argument(argument);
            } else if let Some(step) = step {
                self.macro_replay_step(step);
            }
            self.macros.expanded = false;
            self.buffers.automatic_group(None);
            let waiting = self.layout_active();
            let expected = (self.windows.selected_id(), self.view().point);
            if let Some(run) = self.macros.runner.as_mut() {
                run.expected = expected;
                run.waiting = waiting;
                if run.consent.is_some() || run.query {
                    return Action::Redraw;
                }
            } else {
                return Action::Redraw;
            };
            if self.plugin_macro_waiting() {
                break;
            }
            if waiting || refresh || started.elapsed() >= crate::session::events::INPUT_BATCH_TIME {
                break;
            }
        }
        Action::Redraw
    }

    fn macro_replay_step(&mut self, step: Step) {
        match step {
            Step::Plugin(call) => {
                self.plugin_call(call);
            }
            Step::Key(key) => {
                self.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
            }
            Step::Text(text) => {
                self.handle(Event::Paste(text));
            }
            Step::Counter(operation) => {
                self.macro_counter_operation(operation);
            }
            Step::Redisplay => {}
            Step::Query(mode) => {
                self.macro_query_mode(mode);
            }
            Step::Completion(edit) => {
                if let Err(error) = self.replay_macro_completion(edit) {
                    self.error(&error);
                }
            }
            Step::Command(command) => {
                self.macros.runner.as_mut().expect("live runner").arguments =
                    command.arguments.into();
                if let Some(keys) = command.keys {
                    self.macros.runner.as_mut().expect("live runner").trigger = keys.into();
                } else if let Some(native) = self.input.command_named(&command.name) {
                    self.sync_input_context();
                    if command.argument.supplied() {
                        self.execute_invocation(Invocation {
                            command: native.canonical(),
                            argument: command.argument,
                            origin: self.input.invocation_origin(),
                        });
                    } else {
                        self.execute(native);
                    }
                } else {
                    self.stop_macro(
                        StopReason::MissingCommand,
                        "Recorded command name is unavailable",
                    );
                }
            }
        }
    }

    fn macro_replay_trigger(&mut self, step: Step) {
        match step {
            Step::Key(key) => {
                self.handle(Event::Key(KeyEvent::new(key.code, key.modifiers)));
            }
            Step::Command(prefix) if prefix.keys.is_none() && prefix.arguments.is_empty() => {
                if let Some(command) = Command::named(&prefix.name).filter(|c| c.prefix_control()) {
                    self.execute(command);
                } else {
                    self.stop_macro(StopReason::Unsupported, "Invalid macro prefix input");
                }
            }
            _ => {
                self.stop_macro(StopReason::Unsupported, "Invalid macro key invocation");
            }
        }
    }

    fn macro_replay_argument(&mut self, argument: Argument) {
        if self.macro_consent_pending() {
            // A consent decision can precede another recorded text argument.
            self.macros
                .runner
                .as_mut()
                .expect("live runner")
                .arguments
                .push_front(argument);
            let run = self.macros.runner.as_mut().expect("live runner");
            run.consent = Some(Instant::now());
            run.limits.pause();
            return;
        }
        let role = self.macro_prompt_role();
        let search = matches!(argument.role, Role::Search | Role::SearchRepeat(_));
        if role != Some(argument.role) && !(role == Some(Role::Search) && search) {
            self.stop_macro(
                StopReason::PromptMismatch,
                "Recorded argument does not match the native prompt role",
            );
            return;
        }
        if search {
            if let Err(error) = self.input.search_literal(&argument.value) {
                self.error(&error.to_string());
                return;
            }
            self.search_step();
            if self.macros.runner.is_none() {
                return;
            }
            if let Role::SearchRepeat(direction) = argument.role {
                self.search_action(crate::input::SearchAction::Repeat(direction));
            } else {
                self.input.cancel();
                self.search_action(crate::input::SearchAction::Accept);
            }
        } else if let Some(Pending::Inventory(default)) = self.pending {
            self.input.cancel();
            self.buffer_argument(
                crate::command::BufferCommand::Switch,
                default,
                &argument.value,
            );
        } else if argument.role == Role::Path {
            self.input.cancel();
            let path = std::path::Path::new(&argument.value);
            if path.is_dir() {
                self.stop_macro(
                    StopReason::Unsupported,
                    "Macro cannot replace a directory context",
                );
            } else {
                let _ = self.load_file(path);
            }
        } else {
            self.input.cancel();
            self.argument(argument.value);
        }
    }

    fn macro_prompt_role(&self) -> Option<Role> {
        match &self.pending {
            Some(Pending::Path(_)) => Some(Role::Path),
            Some(Pending::Buffer { .. } | Pending::Inventory(_)) => Some(Role::Buffer),
            Some(Pending::GotoLine { .. }) => Some(Role::Line),
            Some(Pending::Write { .. }) => Some(Role::Write),
            Some(Pending::Search(_)) => Some(Role::Search),
            Some(Pending::Replace(state)) => state.macro_role(),
            _ => None,
        }
    }

    fn macro_consent_pending(&self) -> bool {
        self.input.focus() == Focus::Decision
            && matches!(
                &self.pending,
                Some(Pending::Save { .. } | Pending::Kill { .. }) | Some(Pending::Replace(_))
            )
    }

    pub(in crate::session) fn macro_user_group(&mut self) -> bool {
        let run = self
            .macros
            .runner
            .as_ref()
            .filter(|run| run.consent.is_some() || run.query)
            .or_else(|| {
                self.macros
                    .step_editor
                    .as_ref()
                    .filter(|e| e.capturing)
                    .and_then(|_| self.macros.pauses.last().and_then(|p| p.runner.as_ref()))
            })
            .or_else(|| {
                self.macro_manual()
                    .then(|| {
                        self.macros
                            .pauses
                            .iter()
                            .rev()
                            .find_map(|pause| pause.runner.as_ref())
                    })
                    .flatten()
            });
        if let Some(run) = run {
            self.buffers.automatic_group(Some(run.group.clone()));
            true
        } else {
            false
        }
    }

    pub(in crate::session) fn macro_consent_progress(&mut self) {
        let expected = (self.windows.selected_id(), self.view().point);
        if let Some(run) = self
            .macros
            .runner
            .as_mut()
            .filter(|run| run.consent.is_some())
        {
            run.expected = expected;
            if self.pending.is_none()
                || (self.input.focus() != Focus::Decision && !run.arguments.is_empty())
            {
                run.consent.take().expect("consent pause");
                run.limits.resume();
            }
        }
    }

    pub(in crate::session) fn macro_layout_group(&mut self, active: bool) {
        if self.macro_manual() {
            if active {
                self.macro_manual_guard();
            } else if self.macros.pauses.last().is_some_and(|pause| pause.waiting) {
                self.macro_manual_progress();
            }
        }
        let run = self.macros.runner.as_ref().or_else(|| {
            (self.macro_manual()
                || self
                    .macros
                    .step_editor
                    .as_ref()
                    .is_some_and(|e| e.capturing))
            .then(|| {
                self.macros
                    .pauses
                    .iter()
                    .rev()
                    .find_map(|pause| pause.runner.as_ref())
            })
            .flatten()
        });
        if let Some(run) = run {
            self.buffers
                .automatic_group(active.then(|| run.group.clone()));
        }
        self.macros.expanded = active && self.macros.runner.is_some();
        self.macros.dispatching = active && self.macros.recorder.is_some() && self.layout_active();
    }
}
