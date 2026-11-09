//! Generation-owned contributions and persistent selector matching. Input owns
//! normalized precedence, effective maps, dispatch and key hints.
use super::Session;
use crate::{
    buffer::BufferId,
    command::{Binding as NativeBinding, Command, PluginName},
    input::keymap::{
        Sequence,
        contributions::{Contribution, context, overlaps},
    },
    plugins::{
        declarations,
        lifecycle::editor::{Reply, Request},
    },
};
use rev_plugin_sdk::{
    identity::*,
    methods::*,
    rpc::{ErrorKind, RpcError},
};

#[derive(Clone)]
struct Owned {
    registration: OpaqueId,
    generation: Generation,
    buffer: Option<BufferId>,
    value: Contribution,
}
#[derive(Default)]
pub(super) struct Keymaps {
    serial: u64,
    owned: Vec<Owned>,
    cache: Option<Observation>,
    pub(super) values: Vec<Contribution>,
}
#[derive(PartialEq, Eq)]
struct Observation {
    buffer: BufferId,
    language: String,
    states: Vec<(PackageId, Option<Generation>, bool)>,
    serial: u64,
}
fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
impl Session {
    pub(in crate::session) fn refresh_plugin_keymaps(&mut self) {
        let snapshot = self.buffers.snapshot();
        if !snapshot.settings().plugins.enabled {
            return;
        }
        let states: Vec<_> = snapshot
            .plugins()
            .packages()
            .map(|p| {
                let id = &p.manifest.plugin.id;
                let (generation, enabled) = self.plugins.host.binding_state(id);
                (id.clone(), generation, enabled)
            })
            .collect();
        let owned = &mut self.plugins.keys.owned;
        let before = owned.len();
        owned.retain(|b| {
            states.iter().any(|(id, g, enabled)| {
                *enabled && *id == b.value.package && g.as_ref() == Some(&b.generation)
            }) && b.buffer.is_none_or(|id| self.buffers.get(id).is_ok())
        });
        if owned.len() != before {
            self.plugins.keys.cache = None;
        }
        let id = self.windows.selected().point.stamp.buffer;
        let Ok(buffer) = self.buffers.get(id) else {
            return;
        };
        let language = if buffer.is_transient() || buffer.terminal().is_some() {
            "unsupported"
        } else {
            buffer.language(snapshot)
        };
        let key = Observation {
            buffer: id,
            language: language.to_owned(),
            states,
            serial: self.plugins.keys.serial,
        };
        if self.plugins.keys.cache.as_ref() == Some(&key) {
            return;
        }
        let mut values = Vec::new();
        if language != "unsupported" {
            for package in snapshot.plugins().packages() {
                if !key
                    .states
                    .iter()
                    .any(|(id, _, enabled)| *id == package.manifest.plugin.id && *enabled)
                {
                    continue;
                }
                for binding in package.manifest.bindings.iter() {
                    if matches(binding, buffer, language) {
                        values.push(contribution(binding, false));
                    }
                }
            }
            for binding in &snapshot.settings().plugins.keybindings {
                if matches(binding, buffer, language) {
                    values.push(contribution(binding, true));
                }
            }
            values.extend(
                self.plugins
                    .keys
                    .owned
                    .iter()
                    .filter(|b| b.buffer.is_none_or(|target| target == id))
                    .map(|b| b.value.clone()),
            );
        }
        self.input.plugin_keymaps(values.clone());
        self.plugins.keys.values = values;
        self.plugins.keys.cache = Some(key);
    }
    pub(super) fn plugin_bind(
        &mut self,
        request: &Request,
        params: KeymapsBindParams,
    ) -> Result<Reply, RpcError> {
        if !params.command.belongs_to(&request.package) {
            return Err(error(ErrorKind::NamespaceConflict));
        }
        if self
            .buffers
            .snapshot()
            .plugins()
            .command(params.command.as_str())
            .is_none()
        {
            return Err(error(ErrorKind::CommandUnavailable));
        }
        let sequence = Sequence::parse(params.keys.as_str())
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        let context = context(params.context);
        self.input
            .admit_plugin_binding(context, &sequence)
            .map_err(|_| error(ErrorKind::InvalidArguments))?;
        let buffer = match params.scope {
            Scope::Global => None,
            Scope::Buffer { buffer } => Some(self.plugin_buffer(&buffer)?),
        };
        let registry = self.buffers.snapshot().plugins();
        let package = registry
            .package(request.package.as_str())
            .expect("connected declared package");
        if package.manifest.bindings.len()
            + self
                .plugins
                .keys
                .owned
                .iter()
                .filter(|b| b.value.package == request.package)
                .count()
            >= 128
            || registry
                .packages()
                .map(|p| p.manifest.bindings.len())
                .sum::<usize>()
                + self.plugins.keys.owned.len()
                >= 512
        {
            return Err(error(ErrorKind::LimitExceeded));
        }
        let serial = self
            .plugins
            .keys
            .serial
            .checked_add(1)
            .ok_or_else(|| error(ErrorKind::LimitExceeded))?;
        let registration = OpaqueId::new(format!(
            "binding:{}:{}:{serial}",
            self.plugins.host.handle("registration".into()).session,
            request.generation
        ))
        .expect("bounded binding identity");
        let value = Contribution {
            context,
            sequence,
            command: PluginName::new(params.command.as_str()).expect("qualified command"),
            package: request.package.clone(),
            local: buffer.is_some(),
            user: false,
        };
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        self.plugins.keys.serial = serial;
        self.plugins.keys.owned.push(Owned {
            registration: registration.clone(),
            generation: request.generation.clone(),
            buffer,
            value: value.clone(),
        });
        self.refresh_plugin_keymaps();
        // Inspect the explicitly bound buffer even when another buffer owns
        // focus; ordinary global contributions do not substitute a local target.
        let result = self.plugin_binding_result(registration, &value, buffer);
        Ok(Reply::Bound(result))
    }
    pub(super) fn plugin_unbind(
        &mut self,
        request: &Request,
        params: KeymapsUnbindParams,
    ) -> Result<Reply, RpcError> {
        let mut seen = std::collections::BTreeSet::new();
        for id in params.registrations.iter() {
            if !seen.insert(id) {
                return Err(error(ErrorKind::InvalidArguments));
            }
            let owned = self
                .plugins
                .keys
                .owned
                .iter()
                .find(|b| &b.registration == id)
                .ok_or_else(|| error(ErrorKind::InvalidArguments))?;
            if owned.value.package != request.package || owned.generation != request.generation {
                return Err(error(ErrorKind::NamespaceConflict));
            }
        }
        if !request.live() {
            return Err(error(ErrorKind::Cancelled));
        }
        self.plugins
            .keys
            .owned
            .retain(|b| !seen.contains(&b.registration));
        self.plugins.keys.cache = None;
        self.refresh_plugin_keymaps();
        Ok(Reply::Unbound(Empty {}))
    }
    fn plugin_binding_result(
        &self,
        registration: OpaqueId,
        value: &Contribution,
        buffer: Option<BufferId>,
    ) -> KeymapsBindResult {
        let selected = self.windows.selected().point.stamp.buffer;
        let source = self
            .buffers
            .get(buffer.unwrap_or(selected))
            .expect("admitted binding source");
        let base = self
            .input
            .plugin_binding_native(value.context, source.is_org());
        let (values, map, statuses) = if buffer.is_none_or(|id| id == selected) {
            let values: Vec<_> = self
                .plugins
                .keys
                .values
                .iter()
                .filter(|v| v.context == value.context)
                .cloned()
                .collect();
            let statuses = values
                .iter()
                .map(|v| self.input.plugin_binding_status(v))
                .collect();
            (
                values,
                self.input.plugin_binding_map(value.context).clone(),
                statuses,
            )
        } else {
            // Resolve against the same native owner without selecting its pane.
            let id = buffer.expect("explicit hidden target");
            let source = self.buffers.get(id).expect("admitted buffer");
            let snapshot = self.buffers.snapshot();
            let language = source.language(snapshot);
            let mut values = Vec::new();
            for package in snapshot.plugins().packages() {
                if !self
                    .plugins
                    .host
                    .binding_state(&package.manifest.plugin.id)
                    .1
                {
                    continue;
                }
                values.extend(
                    package
                        .manifest
                        .bindings
                        .iter()
                        .filter(|b| matches(b, source, language))
                        .map(|b| contribution(b, false)),
                );
            }
            values.extend(
                snapshot
                    .settings()
                    .plugins
                    .keybindings
                    .iter()
                    .filter(|b| matches(b, source, language))
                    .map(|b| contribution(b, true)),
            );
            values.extend(
                self.plugins
                    .keys
                    .owned
                    .iter()
                    .filter(|b| b.buffer.is_none_or(|target| target == id))
                    .map(|b| b.value.clone()),
            );
            values.retain(|v| v.context == value.context);
            let (map, statuses) = base.contributions(&values);
            (values, map, statuses)
        };
        let status = values
            .iter()
            .position(|v| v == value)
            .map_or(ResolutionStatus::Shadowed, |i| statuses[i]);
        let mut underlying = Vec::new();
        let mut conflicts = Vec::new();
        for (other, state) in values.iter().zip(&statuses) {
            if !overlaps(&other.sequence, &value.sequence) || other == value {
                continue;
            }
            let source = binding_source(other);
            if *state == ResolutionStatus::Conflicting {
                conflicts.push(source);
            } else {
                underlying.push(source);
            }
        }

        for (sequence, binding) in base
            .bindings()
            .filter(|(s, _)| overlaps(s, &value.sequence))
        {
            if let Some(source) = native_source(binding, base.is_custom(sequence)) {
                underlying.push(source);
            }
        }
        fn order(v: &BindingSource) -> String {
            format!("{:?}:{:?}:{:?}", v.provenance, v.package, v.command)
        }
        underlying.sort_by_key(order);
        conflicts.sort_by_key(order);
        let truncated = underlying.len() > 512 || conflicts.len() > 512;
        underlying.truncate(512);
        conflicts.truncate(512);
        KeymapsBindResult {
            registration,
            status,
            effective: map.lookup(&value.sequence).and_then(|b| {
                if let NativeBinding::Command(Command::PluginDefined(name)) = b {
                    values
                        .iter()
                        .zip(&statuses)
                        .find(|(v, status)| {
                            v.command == name
                                && v.sequence == value.sequence
                                && **status == ResolutionStatus::Effective
                        })
                        .map(|(v, _)| binding_source(v))
                } else {
                    native_source(b, base.is_custom(&value.sequence))
                }
            }),
            underlying: List::new(underlying).expect("bounded inspection"),
            conflicts: List::new(conflicts).expect("bounded inspection"),
            diagnostic: (truncated || status != ResolutionStatus::Effective).then(|| {
                Label::new(if truncated {
                    "Inspection limited to 512 sources; resolve explicit user scope."
                } else if status == ResolutionStatus::Conflicting {
                    "Competing defaults conflict; no plugin winner in this scope."
                } else {
                    "A higher-priority binding shadows this contribution."
                })
                .expect("literal diagnostic")
            }),
        }
    }
}
fn matches(
    binding: &declarations::Binding,
    buffer: &crate::buffer::Buffer,
    language: &str,
) -> bool {
    binding.scope == declarations::Scope::Global
        || ((binding.filetypes.is_empty()
            || binding.filetypes.iter().any(|f| f.as_str() == language))
            && binding
                .buffer_owner
                .as_ref()
                .is_none_or(|owner| buffer.plugin_owner() == Some(owner)))
}
fn contribution(binding: &declarations::Binding, user: bool) -> Contribution {
    let package = binding
        .command
        .as_str()
        .rsplit_once('.')
        .expect("qualified command")
        .0;
    Contribution {
        context: context(binding.context),
        sequence: Sequence::parse(binding.keys.as_str()).expect("admitted sequence"),
        command: PluginName::new(binding.command.as_str()).expect("qualified command"),
        package: PackageId::new(package).expect("qualified package"),
        local: binding.scope == declarations::Scope::Buffer,
        user,
    }
}
fn binding_source(value: &Contribution) -> BindingSource {
    BindingSource {
        command: CommandTarget::Plugin(
            CommandId::new(value.command.name()).expect("qualified command"),
        ),
        provenance: if value.user {
            Provenance::User
        } else {
            Provenance::Plugin
        },
        package: Some(value.package.clone()),
    }
}
fn native_source(value: NativeBinding, user: bool) -> Option<BindingSource> {
    let NativeBinding::Command(command) = value else {
        return None;
    };
    Some(BindingSource {
        command: CommandTarget::Native(CommandName::new(command.name()).ok()?),
        provenance: if user {
            Provenance::User
        } else {
            Provenance::Factory
        },
        package: None,
    })
}
