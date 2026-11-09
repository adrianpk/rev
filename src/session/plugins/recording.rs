//! Native semantic capture and one cooperative in-flight automatic call.
use super::{Action, Session};
use crate::{
    command::{PluginName, PrefixArgument},
    macros::{PluginCall, Step, StopReason},
    plugins::lifecycle::{Receipt, editor::Request},
};
use rev_plugin_sdk::{identity::*, methods::*};

pub(super) struct Automatic {
    receipt: Receipt,
}

impl Session {
    pub(in crate::session) fn plugin_defined(&mut self, name: PluginName) -> Action {
        self.plugin_counted(name, PrefixArgument::default())
    }
    pub(in crate::session) fn plugin_counted(
        &mut self,
        name: PluginName,
        argument: PrefixArgument,
    ) -> Action {
        self.plugin_call(PluginCall {
            command: CommandId::new(name.name()).expect("native qualified identity"),
            arguments: serde_json::json!({}),
            prefix: argument.supplied().then(|| i64::from(argument.value())),
        })
    }
    pub(in crate::session) fn plugin_call(&mut self, call: PluginCall) -> Action {
        let admission = (|| {
            if !self.buffers.snapshot().settings().plugins.enabled {
                return Err("Experimental plugins are disabled".to_owned());
            }
            call.validate().map_err(str::to_owned)?;
            let (package, command) = self
                .buffers
                .snapshot()
                .plugins()
                .command(call.command.as_str())
                .ok_or_else(|| {
                    format!(
                        "Plugin command unavailable (not declared): {}",
                        call.command
                    )
                })?;
            let declaration = command.declaration();
            declaration
                .validate_arguments(
                    &call.arguments,
                    call.prefix,
                    &rev_plugin_sdk::limits::Limits::default(),
                )
                .map_err(|error| error.to_string())?;
            if self.macros.active()
                && (declaration.recording != RecordingPolicy::Semantic || self.macro_manual())
            {
                return Err(format!(
                    "Plugin command does not admit semantic recording: {}",
                    call.command
                ));
            }
            if self.plugin_macro_waiting() {
                return Err("A semantic plugin call is already pending".to_owned());
            }
            Ok((
                package.clone(),
                declaration.recording == RecordingPolicy::Semantic,
            ))
        })();
        let (package, semantic) = match admission {
            Ok(value) => value,
            Err(error) => return self.error(&error),
        };
        // Prefix keys and palette presentation are replaced by exactly one
        // semantic call. Generated RPC edits never pass through input capture.
        if !self.macros.expanded
            && let Some(recorder) = &mut self.macros.recorder
        {
            recorder.discard_pending();
            recorder.selecting = false;
            if let Err(reason) = recorder
                .stage(Step::Plugin(call.clone()))
                .and_then(|()| recorder.accept())
            {
                return self.stop_macro(reason, "Plugin call exceeds macro capture limits");
            }
            if let Err(reason) = self.macros.admit_memory(None) {
                return self.stop_macro(reason, "Macro retained memory exceeds 48 MiB");
            }
        }
        let mut context = self.plugins.host.context();
        context.prefix = call.prefix;
        if let Ok(buffer) = self.buffer()
            && !buffer.is_transient()
            && buffer.terminal().is_none()
        {
            context.stamp = Some(Stamp {
                buffer: self.plugins.host.handle(format!("buffer-{}", buffer.id())),
                revision: Revision::new(buffer.stamp().revision.to_string())
                    .expect("native revision"),
            });
            context.pane = Some(
                self.plugins
                    .host
                    .handle(format!("pane-{}", self.windows.selected_id())),
            );
            context.point = Some(self.view().point.position.0 as u64);
            context.selection = match buffer.region(self.view().point) {
                Ok(selection) => selection.map(|s| {
                    ScalarRange::new(s.range.start as u64, s.range.end as u64)
                        .expect("native region")
                }),
                Err(error) => return self.error(&error.to_string()),
            };
        }
        match self.plugins.host.invoke_with(
            &package,
            crate::plugins::lifecycle::Invocation {
                command: call.command.clone(),
                context,
                arguments: call.arguments.clone(),
            },
            None,
        ) {
            Ok(receipt) => {
                if self.macros.active() {
                    self.plugins.automatic = Some(Automatic { receipt });
                }
                if !self.macros.expanded && semantic {
                    self.macros.recent.draft = None;
                    let entry = self.macro_entry();
                    self.macros.recent.push(entry, Step::Plugin(call.clone()));
                }
                self.message(&format!(
                    "Plugin command pending: {}; C-g cancels",
                    call.command
                ))
            }
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn plugin_macro_waiting(&self) -> bool {
        self.plugins.automatic.is_some()
    }
    pub(in crate::session) fn cancel_plugin_macro(&mut self) {
        if let Some(call) = self.plugins.automatic.take() {
            call.receipt.cancel();
        }
    }
    pub(in crate::session) fn plugin_macro_request(&self, request: &Request) -> bool {
        self.plugins.automatic.as_ref().is_some_and(|call| {
            request.live()
                && self.macros.runner.as_ref().is_none_or(|run| {
                    run.expected == (self.windows.selected_id(), self.view().point)
                })
                && request
                    .cause
                    .as_ref()
                    .is_some_and(|cause| cause.operation == call.receipt.operation)
        })
    }
    pub(super) fn plugin_macro_published(&mut self) {
        let expected = (self.windows.selected_id(), self.view().point);
        if let Some(run) = &mut self.macros.runner {
            run.expected = expected;
        }
    }
    pub(super) fn advance_plugin_macro(&mut self) -> Option<Action> {
        let call = self.plugins.automatic.as_ref()?;
        if !self.macros.active() {
            self.cancel_plugin_macro();
            return None;
        }
        let result = call.receipt.poll()?;
        // Completion ends this automatic operation's authority, including any
        // descendant native request queued before the peer's final response.
        call.receipt.retire();
        if let Err(error) = result {
            let detail = error.kind().map_or_else(
                || format!("RPC error {}", error.code),
                |kind| format!("{kind:?}"),
            );
            return Some(self.stop_macro(
                StopReason::NativeFailure,
                &format!("Semantic plugin call failed: {detail}"),
            ));
        }
        self.plugins.automatic = None;
        Some(Action::Redraw)
    }
}
