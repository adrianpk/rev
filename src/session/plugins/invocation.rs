//! Bounded command discovery and captured, noninteractive native admission.
use super::Session;
use crate::{
    command::{Command, Invocation},
    plugins::lifecycle::editor::{Reply, Request},
};
use rev_plugin_sdk::{
    identity::*,
    methods::*,
    rpc::{ErrorKind, RpcError},
};
use serde_json::json;
fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
impl Session {
    pub(super) fn plugin_commands(&self, params: CommandsListParams) -> Result<Reply, RpcError> {
        let prefix = format!(
            "commands-{}-",
            self.plugins.host.handle("cursor".into()).session
        );
        let after = params
            .page
            .cursor
            .as_ref()
            .map(|c| {
                c.as_str()
                    .strip_prefix(&prefix)
                    .and_then(|n| n.parse::<usize>().ok())
                    .ok_or_else(|| error(ErrorKind::InvalidArguments))
            })
            .transpose()?
            .unwrap_or(0);
        let mut commands = Vec::new();
        for command in Command::ALL {
            commands.push(CommandMetadata {
                id: CommandTarget::Native(
                    CommandName::new(command.name()).expect("native command grammar"),
                ),
                description: Label::new(command.description().expect("static native description"))?,
                parameters: List::default(),
                prefix: PrefixPolicy::None,
                recording: RecordingPolicy::Reject,
                available: true,
                noninteractive: matches!(command.invocation(), Invocation::Motion(_)),
            });
        }
        for package in self.buffers.snapshot().plugins().packages() {
            for command in package.manifest.commands.iter() {
                let declaration = command.declaration();
                commands.push(CommandMetadata {
                    id: CommandTarget::Plugin(package.manifest.plugin.id.command(&command.name)),
                    description: declaration.description,
                    parameters: declaration.parameters,
                    prefix: declaration.prefix,
                    recording: declaration.recording,
                    available: self.buffers.snapshot().settings().plugins.enabled
                        && self.plugins.host.command_available(
                            &package.manifest.plugin.id,
                            &package.manifest.plugin.id.command(&command.name),
                        ),
                    noninteractive: true,
                });
            }
        }
        fn name(c: &CommandMetadata) -> &str {
            match &c.id {
                CommandTarget::Native(n) => n.as_str(),
                CommandTarget::Plugin(n) => n.as_str(),
            }
        }
        commands.sort_by(|a, b| name(a).cmp(name(b)));
        let mut matches = commands
            .into_iter()
            .filter(|c| name(c).contains(params.query.as_str()))
            .skip(after);
        let commands: Vec<_> = matches
            .by_ref()
            .take(usize::from(params.page.limit))
            .collect();
        let next = if matches.next().is_some() {
            Some(
                OpaqueId::new(format!("{prefix}{}", after + commands.len()))
                    .expect("finite command cursor"),
            )
        } else {
            None
        };
        Ok(Reply::Commands(CommandsListResult {
            commands: List::new(commands)?,
            next,
        }))
    }
    fn plugin_invocation_context(
        &self,
        request: &Request,
        context: &InvocationContext,
    ) -> Result<(), RpcError> {
        if self.pending.is_some()
            || self.input.focus() != crate::input::Focus::Editing
            || self.context_storage.locked()
            || self.context_prepared.is_some()
            || self.quit_pending()
            || (self.macros.active() && !self.plugin_macro_request(request))
            || self.capture_blocks_input()
            || self.capture_selected()
            || self.source_edit_selected()
            || self.windows.retained_view_active()
            || self.tangling.is_some()
            || self.execution.is_some()
            || self.feeds.is_some()
            || self.terminals.focused
            || self.tree.focused()
        {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        if let Some(stamp) = &context.stamp {
            let id = self.plugin_buffer(&stamp.buffer)?;
            let buffer = self.buffers.get(id).expect("resolved source");
            if self.plugin_descriptor(buffer)?.stamp != *stamp {
                return Err(error(ErrorKind::StaleRevision));
            }
            if context.point.is_some_and(|p| p > buffer.len_chars() as u64)
                || context
                    .selection
                    .is_some_and(|s| s.end() > buffer.len_chars() as u64)
            {
                return Err(error(ErrorKind::InvalidRange));
            }
        } else if context.point.is_some() || context.selection.is_some() || context.pane.is_some() {
            return Err(error(ErrorKind::InvalidArguments));
        }
        if let Some(pane) = &context.pane {
            self.plugin_invocation_pane(context, pane)?;
        }
        Ok(())
    }
    fn plugin_invocation_pane(
        &self,
        context: &InvocationContext,
        pane: &Handle,
    ) -> Result<crate::window::ViewId, RpcError> {
        if !self.plugins.host.owns(pane) {
            return Err(error(ErrorKind::UnsupportedInvocation));
        }
        self.windows
            .iter()
            .find(|(id, view)| {
                pane.id.as_str() == format!("pane-{id}")
                    && context.stamp.as_ref().is_some_and(|s| {
                        self.plugin_buffer(&s.buffer).ok() == Some(view.point.stamp.buffer)
                    })
            })
            .map(|(id, _)| id)
            .ok_or_else(|| error(ErrorKind::UnsupportedInvocation))
    }
    pub(super) fn plugin_invoke(&mut self, request: Request, params: CommandsInvokeParams) {
        let admission = self
            .plugin_invocation_context(&request, &params.context)
            .and_then(|()| match &params.command {
                CommandTarget::Plugin(id) => {
                    let (package, command) = self
                        .buffers
                        .snapshot()
                        .plugins()
                        .command(id.as_str())
                        .ok_or_else(|| error(ErrorKind::CommandUnavailable))?;
                    let declaration = command.declaration();
                    if self.macros.active() && declaration.recording != RecordingPolicy::Semantic {
                        return Err(error(ErrorKind::UnsupportedInvocation));
                    }
                    declaration.validate_arguments(
                        &params.arguments,
                        params.context.prefix,
                        &rev_plugin_sdk::limits::Limits::default(),
                    )?;
                    Ok(Some(package.clone()))
                }
                CommandTarget::Native(name) => {
                    if params.context.prefix.is_some() {
                        return Err(error(ErrorKind::UnsupportedInvocation));
                    }
                    if params.arguments != json!({}) {
                        return Err(error(ErrorKind::InvalidArguments));
                    }
                    let command = Command::named(name.as_str())
                        .ok_or_else(|| error(ErrorKind::CommandUnavailable))?;
                    match command.invocation() {
                        Invocation::Motion(motion) => {
                            let pane = self.plugin_invocation_pane(
                                &params.context,
                                params
                                    .context
                                    .pane
                                    .as_ref()
                                    .ok_or_else(|| error(ErrorKind::InvalidArguments))?,
                            )?;
                            let mut view = self.windows.get(pane).expect("admitted pane").clone();
                            view.point.position = crate::coordinates::Scalar(
                                usize::try_from(
                                    params
                                        .context
                                        .point
                                        .ok_or_else(|| error(ErrorKind::InvalidArguments))?,
                                )
                                .map_err(|_| error(ErrorKind::InvalidRange))?,
                            );
                            let buffer = self
                                .buffers
                                .get(view.point.stamp.buffer)
                                .expect("admitted source");
                            let size = self
                                .windows
                                .area(pane, self.editing_size())
                                .ok_or_else(|| error(ErrorKind::UnsupportedInvocation))?
                                .view_size();
                            let coordinates = crate::coordinates::Coordinates::new(buffer)
                                .map_err(|_| error(ErrorKind::InvalidRange))?;
                            let width = view
                                .text_width(buffer, size)
                                .map_err(|_| error(ErrorKind::UnsupportedInvocation))?;
                            view.point.position =
                                Self::observed_motion(&view, buffer, &coordinates, width, motion)
                                    .map_err(|_| error(ErrorKind::InvalidRange))?;
                            view.reset_navigation();
                            if !request.live() {
                                return Err(error(ErrorKind::Cancelled));
                            }
                            let target = view.point.stamp.buffer;
                            self.buffers
                                .get_mut(target)
                                .expect("admitted source")
                                .break_typing();
                            *self.windows.get_mut(pane).expect("admitted pane") = view;
                            self.plugin_macro_published();
                            if pane == self.windows.selected_id() {
                                self.invalidate_completion();
                                self.cancel_pointer_capture();
                                self.yank = None;
                                self.kills.break_chain();
                            }
                            Ok(None)
                        }
                        Invocation::Consent => Err(error(ErrorKind::ConfirmationRequired)),
                        Invocation::Unsupported => Err(error(ErrorKind::UnsupportedInvocation)),
                    }
                }
            });
        match admission {
            Err(error) => request.complete(Err(error)),
            Ok(None) => request.complete(Ok(Reply::Invoked(CommandsInvokeResult {
                value: serde_json::Value::Null,
            }))),
            Ok(Some(package)) => {
                let CommandTarget::Plugin(command) = params.command else {
                    unreachable!("admitted plugin")
                };
                // The pending caller permit follows the callee and its exact
                // result; a dropped callee reports disconnection to the caller.
                let _ = self.plugins.host.invoke_with(
                    &package,
                    command,
                    params.context,
                    params.arguments,
                    Some(request),
                );
            }
        }
    }
}
