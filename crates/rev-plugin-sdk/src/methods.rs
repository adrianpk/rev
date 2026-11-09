//! Concrete method schemas and directions for the complete experimental surface.
//!
//! All parameter objects reject unknown fields. Optional observations are
//! explicit: `expected_version: null` means create only if absent; a null state
//! *value* remains ordinary JSON content. Save requires both source and disk
//! observations. No schema grants force, consent, prompt access or publication.

use crate::{
    API,
    identity::*,
    limits::*,
    rpc::{ErrorKind, RpcError},
};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeOwned, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeSet, fmt, marker::PhantomData, ops::Deref};

/// A collection rejects its first excess element, before deserializing that element.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct List<T, const N: usize>(Vec<T>);
impl<T, const N: usize> List<T, N> {
    /// Retain an owned collection within its declared item ceiling.
    ///
    /// # Errors
    /// Returns `LimitExceeded` when `values` contains more than `N` items.
    pub fn new(values: Vec<T>) -> Result<Self, RpcError> {
        if values.len() > N {
            Err(error(ErrorKind::LimitExceeded))
        } else {
            Ok(Self(values))
        }
    }
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
    pub(crate) fn retain(&mut self, predicate: impl FnMut(&T) -> bool) {
        self.0.retain(predicate);
    }
}
impl<T, const N: usize> Default for List<T, N> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl<T, const N: usize> Deref for List<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'de, T: Deserialize<'de>, const N: usize> Deserialize<'de> for List<T, N> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Bounded<T, const N: usize>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Bounded<T, N> {
            type Value = List<T, N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} items")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while values.len() < N {
                    match seq.next_element()? {
                        Some(value) => values.push(value),
                        None => return Ok(List(values)),
                    }
                }
                // IgnoredAny consumes no application value or collection allocation.
                if seq.next_element::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom("collection limit exceeded"));
                }
                Ok(List(values))
            }
        }
        deserializer.deserialize_seq(Bounded::<T, N>(PhantomData))
    }
}

/// A caller-owned UTF-8 string with a byte ceiling (not a scalar ceiling).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Text<const N: usize>(String);
impl<const N: usize> Text<N> {
    /// Retain an owned UTF-8 string within its declared byte ceiling.
    ///
    /// # Errors
    /// Returns `LimitExceeded` when the string contains more than `N` bytes.
    pub fn new(value: impl Into<String>) -> Result<Self, RpcError> {
        let value = value.into();
        if value.len() > N {
            Err(error(ErrorKind::LimitExceeded))
        } else {
            Ok(Self(value))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl<const N: usize> TryFrom<String> for Text<N> {
    type Error = RpcError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl<const N: usize> From<Text<N>> for String {
    fn from(value: Text<N>) -> Self {
        value.0
    }
}
pub type SourceText = Text<TEXT_BYTES>;
pub type Label = Text<4096>;

fn error(kind: ErrorKind) -> RpcError {
    RpcError::application(kind)
}
fn require(condition: bool, kind: ErrorKind) -> Result<(), RpcError> {
    if condition { Ok(()) } else { Err(error(kind)) }
}

/// Additional semantic admission beyond Serde's shape/identity/collection checks.
pub trait Validate {
    /// Check the value's semantic invariants and negotiated limits.
    ///
    /// # Errors
    /// Returns the applicable argument, namespace, revision or limit error when
    /// the value violates its concrete schema's admission rules.
    fn validate(&self, _limits: &Limits) -> Result<(), RpcError> {
        Ok(())
    }
}
macro_rules! object {
    ($(#[$meta:meta])* $name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $(pub $field: $ty,)* }
    };
}
macro_rules! plain {
    ($($name:ty),+ $(,)?) => { $(impl Validate for $name {})+ };
}
macro_rules! choices {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant,)+ }
    };
}

choices!(Capability {
    Buffers,
    Edits,
    Display,
    FileLifecycle,
    Commands,
    Invocation,
    Keymaps,
    Events,
    Messages,
    State
});
choices!(Units { UnicodeScalar });
choices!(PrefixPolicy { None, Integer });
choices!(RecordingPolicy { Reject, Semantic });
choices!(ArgumentType {
    Null,
    Boolean,
    Integer,
    String,
    Array,
    Object
});
choices!(ResourceKind {
    Text,
    Terminal,
    Generated
});
choices!(Placement {
    Current,
    SplitHorizontal,
    SplitVertical
});
choices!(Context {
    Editor,
    VimNormal,
    VimInsert,
    VimReplace,
    VimVisualCharacter,
    VimVisualLine,
    VimVisualBlock
});
choices!(ResolutionStatus {
    Effective,
    Shadowed,
    Conflicting
});
choices!(Provenance {
    User,
    Plugin,
    Factory
});
choices!(Severity {
    Info,
    Warning,
    Error
});
choices!(EventKind {
    Opened,
    Changed,
    Closed,
    Resnapshot
});

object!(Empty {});
object!(Package {
    id: PackageId,
    version: Label
});
object!(Handler {
    command: CommandId,
    handler: OpaqueId
});
object!(InitializeParams {
    session: OpaqueId, generation: Generation, package: Package, api: Label,
    capabilities: List<Capability, 10>, units: Units, limits: Limits,
});
object!(InitializeResult {
    session: OpaqueId, generation: Generation, package: Package, api: Label,
    capabilities: List<Capability, 10>, units: Units, limits: Limits,
    handlers: List<Handler, 64>,
});
impl Validate for InitializeParams {
    fn validate(&self, _: &Limits) -> Result<(), RpcError> {
        require(self.api.as_str() == API, ErrorKind::UnsupportedApi)?;
        require(self.limits.validate(), ErrorKind::LimitExceeded)?;
        require(
            self.capabilities.iter().collect::<BTreeSet<_>>().len() == self.capabilities.len(),
            ErrorKind::InvalidArguments,
        )?;
        require(
            !self.package.version.as_str().is_empty(),
            ErrorKind::InvalidArguments,
        )
    }
}
impl InitializeParams {
    /// Package-side acknowledgement; rejects a foreign identity or revision.
    ///
    /// # Errors
    /// Returns an API, namespace, argument or limit error when the offer or
    /// proposed handlers cannot form the exact negotiated agreement.
    pub fn acknowledge(
        &self,
        package: &Package,
        handlers: List<Handler, 64>,
    ) -> Result<InitializeResult, RpcError> {
        self.validate(&self.limits)?;
        require(&self.package == package, ErrorKind::NamespaceConflict)?;
        let result = InitializeResult {
            session: self.session.clone(),
            generation: self.generation.clone(),
            package: package.clone(),
            api: self.api.clone(),
            capabilities: self.capabilities.clone(),
            units: self.units,
            limits: self.limits.clone(),
            handlers,
        };
        result.validate(&self.limits)?;
        Ok(result)
    }
    /// Host-side agreement check. Handler declarations are checked separately by the host.
    ///
    /// # Errors
    /// Returns an API, argument, namespace or limit error when either side is
    /// invalid or the response differs from the offered connection/profile.
    pub fn accepts(&self, result: &InitializeResult) -> Result<(), RpcError> {
        self.validate(&self.limits)?;
        result.validate(&self.limits)?;
        require(result.api == self.api, ErrorKind::UnsupportedApi)?;
        require(
            result.session == self.session
                && result.generation == self.generation
                && result.package == self.package
                && result.capabilities == self.capabilities
                && result.units == self.units
                && result.limits == self.limits,
            ErrorKind::InvalidArguments,
        )
    }
}
impl Validate for InitializeResult {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        InitializeParams {
            session: self.session.clone(),
            generation: self.generation.clone(),
            package: self.package.clone(),
            api: self.api.clone(),
            capabilities: self.capabilities.clone(),
            units: self.units,
            limits: self.limits.clone(),
        }
        .validate(limits)?;
        require(
            self.handlers.len() <= limits.commands_per_package as usize,
            ErrorKind::LimitExceeded,
        )?;
        let mut commands = BTreeSet::new();
        let mut handlers = BTreeSet::new();
        for handler in self.handlers.iter() {
            require(
                handler.command.belongs_to(&self.package.id)
                    && commands.insert(&handler.command)
                    && handlers.insert(&handler.handler),
                ErrorKind::NamespaceConflict,
            )?;
        }
        Ok(())
    }
}

object!(
    /// Pagination uses opaque session-local cursors. Limits are 1..=256.
    Page { cursor: Option<OpaqueId>, limit: u16 }
);
impl Validate for Page {
    fn validate(&self, _: &Limits) -> Result<(), RpcError> {
        require(
            self.limit > 0 && self.limit <= 256,
            ErrorKind::LimitExceeded,
        )
    }
}
/// A disk observation is an opaque native token or explicit absence, never a force flag.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiskObservation {
    Absent,
    Observed { token: OpaqueId },
}
object!(BufferDescriptor {
    stamp: Stamp, name: Label, path: Option<Label>, kind: ResourceKind,
    read_only: bool, modified: bool, expected_disk: DiskObservation,
    owner: Option<PackageId>, scalar_len: u64,
});
object!(BuffersListParams { page: Page, kind: Option<ResourceKind> });
object!(BuffersListResult { buffers: List<BufferDescriptor, 256>, next: Option<OpaqueId> });
object!(BuffersOpenParams { path: Label });
object!(BuffersCreateParams {
    name: Label,
    text: SourceText,
    read_only: bool
});
object!(BufferParams { buffer: Handle });
object!(BuffersReadParams { buffer: Handle, range: ScalarRange, expected_revision: Option<Revision> });
object!(BuffersReadResult {
    stamp: Stamp,
    range: ScalarRange,
    text: SourceText
});
object!(BuffersShowParams { buffer: Handle, pane: Option<Handle>, placement: Placement });
object!(BuffersShowResult { pane: Handle });
object!(BuffersSaveParams { buffer: Handle, path: Option<Label>, expected_revision: Revision, expected_disk: DiskObservation });
object!(BuffersSaveResult {
    stamp: Stamp,
    expected_disk: DiskObservation
});
object!(BuffersCloseParams {
    buffer: Handle,
    expected_revision: Revision
});
object!(BuffersCloseResult { closed: bool });

object!(Replacement {
    range: ScalarRange,
    text: SourceText
});
object!(BufferEdits { stamp: Stamp, replacements: List<Replacement, 256> });
object!(Origin { package: Option<PackageId>, operation: OpaqueId, depth: u8 });
object!(EditsApplyParams { operation: OpaqueId, targets: List<BufferEdits, 16>, origin: Origin });
object!(EditsApplyResult { operation: OpaqueId, stamps: List<Stamp, 16> });
impl Validate for Origin {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        require(
            u32::from(self.depth) <= limits.causal_depth,
            ErrorKind::LimitExceeded,
        )
    }
}
impl Validate for EditsApplyParams {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        self.origin.validate(limits)?;
        require(
            !self.targets.is_empty() && self.targets.len() <= limits.edit_buffers as usize,
            ErrorKind::LimitExceeded,
        )?;
        let mut buffers = BTreeSet::new();
        let mut count = 0;
        let mut bytes = 0;
        for target in self.targets.iter() {
            require(
                buffers.insert((&target.stamp.buffer.session, &target.stamp.buffer.id)),
                ErrorKind::EditConflict,
            )?;
            bytes += target.replacement_bytes()?;
            count += target.replacements.len();
        }
        require(
            count <= limits.replacements as usize && bytes <= limits.text_bytes as usize,
            ErrorKind::LimitExceeded,
        )
    }
}

impl BufferEdits {
    fn replacement_bytes(&self) -> Result<usize, RpcError> {
        require(!self.replacements.is_empty(), ErrorKind::InvalidArguments)?;
        let mut previous: Option<ScalarRange> = None;
        let mut bytes = 0;
        for replacement in self.replacements.iter() {
            if let Some(range) = previous {
                require(
                    replacement.range.start() >= range.end()
                        && replacement.range.start() != range.start(),
                    ErrorKind::EditConflict,
                )?;
            }
            previous = Some(replacement.range);
            bytes += replacement.text.as_str().len();
        }
        Ok(bytes)
    }
}

object!(
    /// Argument contracts admit inert JSON values, never executable snippets.
    Parameter { name: RecordId, kind: ArgumentType, required: bool }
);
object!(CommandDeclaration {
    name: CommandName, description: Label, parameters: List<Parameter, 64>,
    prefix: PrefixPolicy, recording: RecordingPolicy,
});
object!(CommandContribution {
    declaration: CommandDeclaration,
    handler: OpaqueId
});
/// Existing native command grammar and qualified package identities remain distinct.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandTarget {
    Plugin(CommandId),
    Native(CommandName),
}
object!(CommandMetadata {
    id: CommandTarget, description: Label, parameters: List<Parameter, 64>,
    prefix: PrefixPolicy, recording: RecordingPolicy, available: bool,
    noninteractive: bool,
});
object!(CommandsRegisterParams { commands: List<CommandContribution, 64> });
object!(Registration {
    id: OpaqueId,
    command: CommandId
});
object!(CommandsRegisterResult { registrations: List<Registration, 64> });
object!(CommandsUnregisterParams { registrations: List<OpaqueId, 64> });
object!(CommandsListParams {
    page: Page,
    query: Label
});
object!(CommandsListResult { commands: List<CommandMetadata, 256>, next: Option<OpaqueId> });
object!(InvocationContext {
    invocation: OpaqueId, generation: Generation, stamp: Option<Stamp>, pane: Option<Handle>,
    point: Option<u64>, selection: Option<ScalarRange>, prefix: Option<i64>, origin: Origin,
});
object!(CommandsInvokeParams {
    command: CommandTarget,
    arguments: Value,
    context: InvocationContext
});
object!(CommandsInvokeResult { value: Value });
object!(CommandInvokeParams {
    command: CommandId,
    handler: OpaqueId,
    arguments: Value,
    context: InvocationContext
});
object!(CommandCancelParams {
    invocation: OpaqueId,
    generation: Generation
});

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Scope {
    Global,
    Buffer { buffer: Handle },
}
object!(KeymapsBindParams {
    command: CommandId,
    keys: Label,
    context: Context,
    scope: Scope
});
object!(BindingSource { command: CommandTarget, provenance: Provenance, package: Option<PackageId> });
object!(KeymapsBindResult {
    registration: OpaqueId, status: ResolutionStatus, effective: Option<BindingSource>,
    underlying: List<BindingSource, 512>, conflicts: List<BindingSource, 512>, diagnostic: Option<Label>,
});
object!(KeymapsUnbindParams { registrations: List<OpaqueId, 128> });
object!(EventsSubscribeParams { kinds: List<EventKind, 4>, buffers: List<Handle, 256> });
object!(EventsSubscribeResult {
    subscription: OpaqueId,
    sequence: Revision
});
object!(EventsUnsubscribeParams {
    subscription: OpaqueId
});
object!(
    /// `resnapshot` explicitly invalidates any incomplete delta stream.
    BufferEvent {
    subscription: OpaqueId, sequence: Revision, generation: Generation,
    kind: EventKind, stamp: Option<Stamp>, origin: Origin,
});
object!(MessagesShowParams {
    text: Label,
    severity: Severity
});

object!(StateListParams {
    concern: RecordId,
    page: Page
});
object!(StateEntry {
    record: RecordId,
    version: Revision
});
object!(StateListResult { records: List<StateEntry, 256>, next: Option<OpaqueId> });
object!(StateGetParams {
    concern: RecordId,
    record: RecordId
});
/// An absent record differs from a present JSON null value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum StateGetResult {
    Absent,
    Present { version: Revision, value: Value },
}
object!(StatePutParams {
    concern: RecordId,
    record: RecordId,
    expected_version: ExpectedVersion,
    value: Value
});
object!(StatePutResult { version: Revision });
object!(StateDeleteParams {
    concern: RecordId,
    record: RecordId,
    expected_version: Revision
});

/// Required precondition: explicit null means absence; omission is invalid.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ExpectedVersion(pub Option<Revision>);
impl<'de> Deserialize<'de> for ExpectedVersion {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Expected;
        impl Visitor<'_> for Expected {
            type Value = ExpectedVersion;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("null or a decimal-string version")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(ExpectedVersion(None))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Revision::new(value)
                    .map(|value| ExpectedVersion(Some(value)))
                    .map_err(E::custom)
            }
        }
        deserializer.deserialize_any(Expected)
    }
}

fn json_bytes(value: &Value, max: usize) -> Result<(), RpcError> {
    // The transport's frame ceiling already bounds parsed input; this enforces
    // the smaller per-record/argument ceiling for locally constructed values.
    struct Counter {
        bytes: usize,
        max: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.max.saturating_sub(self.bytes) {
                return Err(std::io::Error::other("JSON size limit"));
            }
            self.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { bytes: 0, max }, value)
        .map_err(|_| error(ErrorKind::LimitExceeded))
}
impl Validate for StatePutParams {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        json_bytes(&self.value, limits.state_record_bytes as usize)
    }
}
impl Validate for StateGetResult {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        if let Self::Present { value, .. } = self {
            json_bytes(value, limits.state_record_bytes as usize)?;
        }
        Ok(())
    }
}
impl Validate for InvocationContext {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        self.origin.validate(limits)?;
        require(
            self.stamp.is_some() || (self.point.is_none() && self.selection.is_none()),
            ErrorKind::InvalidArguments,
        )?;
        if let (Some(stamp), Some(pane)) = (&self.stamp, &self.pane) {
            require(
                stamp.buffer.session == pane.session,
                ErrorKind::InvalidArguments,
            )?;
        }
        Ok(())
    }
}
impl CommandDeclaration {
    /// Validate declared argument names/types and the explicit prefix policy.
    ///
    /// # Errors
    /// Returns `UnsupportedInvocation` for a disallowed prefix, `LimitExceeded`
    /// for excessive JSON, or an argument error for missing/unknown/wrong types.
    pub fn validate_arguments(
        &self,
        arguments: &Value,
        prefix: Option<i64>,
        limits: &Limits,
    ) -> Result<(), RpcError> {
        self.validate(limits)?;
        require(
            prefix.is_none() || self.prefix == PrefixPolicy::Integer,
            ErrorKind::UnsupportedInvocation,
        )?;
        json_bytes(arguments, limits.text_bytes as usize)?;
        let args = arguments
            .as_object()
            .ok_or_else(|| error(ErrorKind::InvalidArguments))?;
        require(
            args.keys().all(|key| {
                self.parameters
                    .iter()
                    .any(|param| param.name.as_str() == key)
            }),
            ErrorKind::InvalidArguments,
        )?;
        for param in self.parameters.iter() {
            param.validate_argument(args.get(param.name.as_str()))?;
        }
        Ok(())
    }
}

impl Parameter {
    fn validate_argument(&self, value: Option<&Value>) -> Result<(), RpcError> {
        require(
            !self.required || value.is_some(),
            ErrorKind::InvalidArguments,
        )?;
        if let Some(value) = value {
            require(argument_type(self.kind, value), ErrorKind::InvalidArguments)?;
        }
        Ok(())
    }
}

fn argument_type(kind: ArgumentType, value: &Value) -> bool {
    match kind {
        ArgumentType::Null => value.is_null(),
        ArgumentType::Boolean => value.is_boolean(),
        ArgumentType::Integer => value.is_i64(),
        ArgumentType::String => value.is_string(),
        ArgumentType::Array => value.is_array(),
        ArgumentType::Object => value.is_object(),
    }
}
impl Validate for CommandDeclaration {
    fn validate(&self, _: &Limits) -> Result<(), RpcError> {
        require(
            !self.description.as_str().trim().is_empty(),
            ErrorKind::InvalidArguments,
        )?;
        require(
            self.parameters
                .iter()
                .map(|p| &p.name)
                .collect::<BTreeSet<_>>()
                .len()
                == self.parameters.len(),
            ErrorKind::InvalidArguments,
        )
    }
}
impl Validate for CommandsRegisterParams {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        require(
            !self.commands.is_empty()
                && self.commands.len() <= limits.commands_per_package as usize,
            ErrorKind::LimitExceeded,
        )?;
        let mut names = BTreeSet::new();
        let mut handlers = BTreeSet::new();
        for command in self.commands.iter() {
            command.declaration.validate(limits)?;
            require(
                names.insert(&command.declaration.name) && handlers.insert(&command.handler),
                ErrorKind::NamespaceConflict,
            )?;
        }
        Ok(())
    }
}
macro_rules! paged {
    ($($name:ty),+) => { $(impl Validate for $name {
        fn validate(&self, limits: &Limits) -> Result<(), RpcError> { self.page.validate(limits) }
    })+ };
}
paged!(BuffersListParams, CommandsListParams, StateListParams);
impl Validate for BuffersCreateParams {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        require(
            self.text.as_str().len() <= limits.text_bytes as usize,
            ErrorKind::LimitExceeded,
        )
    }
}
impl Validate for BuffersReadResult {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        require(
            self.text.as_str().len() <= limits.text_bytes as usize,
            ErrorKind::LimitExceeded,
        )?;
        require(
            self.text.as_str().chars().count() as u64 == self.range.end() - self.range.start(),
            ErrorKind::InvalidRange,
        )
    }
}
macro_rules! invocation {
    ($($name:ty),+) => { $(impl Validate for $name {
        fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
            self.context.validate(limits)?;
            require(self.arguments.is_object(), ErrorKind::InvalidArguments)?;
            json_bytes(&self.arguments, limits.text_bytes as usize)
        }
    })+ };
}
invocation!(CommandsInvokeParams, CommandInvokeParams);
impl Validate for CommandsInvokeResult {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        json_bytes(&self.value, limits.text_bytes as usize)
    }
}
impl BufferEvent {
    /// Callback identity accepted by commands.invoke for this delivered event.
    /// Copy generation, origin and any required captured stamp into its context.
    ///
    /// # Panics
    /// Panics if a valid decimal sequence cannot form a bounded opaque identity,
    /// indicating an internal violation of the identity length contract.
    pub fn invocation(&self) -> OpaqueId {
        OpaqueId::new(format!("event:{}", self.sequence)).expect("bounded event sequence")
    }
}
impl Validate for BufferEvent {
    fn validate(&self, limits: &Limits) -> Result<(), RpcError> {
        self.origin.validate(limits)?;
        require(
            self.kind == EventKind::Resnapshot || self.stamp.is_some(),
            ErrorKind::InvalidArguments,
        )
    }
}
impl Validate for EventsSubscribeParams {
    fn validate(&self, _: &Limits) -> Result<(), RpcError> {
        require(
            !self.kinds.is_empty()
                && self.kinds.iter().collect::<BTreeSet<_>>().len() == self.kinds.len(),
            ErrorKind::InvalidArguments,
        )
    }
}
plain!(
    Empty,
    BufferDescriptor,
    BuffersListResult,
    BuffersOpenParams,
    BufferParams,
    BuffersReadParams,
    BuffersShowParams,
    BuffersShowResult,
    BuffersSaveParams,
    BuffersSaveResult,
    BuffersCloseParams,
    BuffersCloseResult,
    EditsApplyResult,
    CommandsRegisterResult,
    CommandsUnregisterParams,
    CommandsListResult,
    CommandCancelParams,
    KeymapsBindParams,
    KeymapsBindResult,
    KeymapsUnbindParams,
    EventsSubscribeResult,
    EventsUnsubscribeParams,
    MessagesShowParams,
    StateListResult,
    StateGetParams,
    StatePutResult,
    StateDeleteParams
);

/// Request schemas bind a method name to its parameter/result types and direction.
pub trait Method {
    const NAME: &'static str;
    const CAPABILITY: Option<Capability>;
    const HOST_TO_PACKAGE: bool;
    type Params: Serialize + DeserializeOwned + Validate;
    type Result: Serialize + DeserializeOwned + Validate;
}
macro_rules! methods {
    ($($name:ident, $wire:literal, $cap:expr, $host:expr, $params:ty => $result:ty;)+) => {
        $(pub struct $name;
        impl Method for $name {
            const NAME: &'static str = $wire;
            const CAPABILITY: Option<Capability> = $cap;
            const HOST_TO_PACKAGE: bool = $host;
            type Params = $params;
            type Result = $result;
        })+
        /// Validate a known method before native dispatch. Execution still checks
        /// session, generation, advertised capability and native resource policy.
        ///
        /// # Errors
        /// Returns `UnsupportedMethod` for an unknown name, `InvalidArguments`
        /// for an invalid shape, or the concrete parameter validation error.
        pub fn validate_params(method: &str, params: Value, limits: &Limits) -> Result<(), RpcError> {
            match method {
                $($wire => {
                    let params: $params = serde_json::from_value(params).map_err(|_| error(ErrorKind::InvalidArguments))?;
                    params.validate(limits)
                },)+
                _ => Err(error(ErrorKind::UnsupportedMethod)),
            }
        }
        /// Apply result shape and finite payload admission before correlation completes.
        ///
        /// # Errors
        /// Returns `UnsupportedMethod` for an unknown name, `InvalidArguments`
        /// for an invalid shape, or the concrete result validation error.
        pub fn validate_result(method: &str, value: Value, limits: &Limits) -> Result<(), RpcError> {
            match method {
                $($wire => {
                    let result: $result = serde_json::from_value(value).map_err(|_| error(ErrorKind::InvalidArguments))?;
                    result.validate(limits)
                },)+
                _ => Err(error(ErrorKind::UnsupportedMethod)),
            }
        }
        /// Schema catalog; this is not a list of implemented host capabilities.
        pub const METHODS: &[(&str, Option<Capability>, bool)] = &[
            $(($wire, $cap, $host),)+
        ];
    };
}
use Capability::*;
methods! {
    Initialize, "initialize", None, true, InitializeParams => InitializeResult;
    BuffersList, "buffers.list", Some(Buffers), false, BuffersListParams => BuffersListResult;
    BuffersOpen, "buffers.open", Some(Buffers), false, BuffersOpenParams => BufferDescriptor;
    BuffersCreate, "buffers.create", Some(Buffers), false, BuffersCreateParams => BufferDescriptor;
    BuffersDescribe, "buffers.describe", Some(Buffers), false, BufferParams => BufferDescriptor;
    BuffersRead, "buffers.read", Some(Buffers), false, BuffersReadParams => BuffersReadResult;
    BuffersShow, "buffers.show", Some(Display), false, BuffersShowParams => BuffersShowResult;
    BuffersSave, "buffers.save", Some(FileLifecycle), false, BuffersSaveParams => BuffersSaveResult;
    BuffersClose, "buffers.close", Some(FileLifecycle), false, BuffersCloseParams => BuffersCloseResult;
    EditsApply, "edits.apply", Some(Edits), false, EditsApplyParams => EditsApplyResult;
    CommandsRegister, "commands.register", Some(Commands), false, CommandsRegisterParams => CommandsRegisterResult;
    CommandsUnregister, "commands.unregister", Some(Commands), false, CommandsUnregisterParams => Empty;
    CommandsList, "commands.list", Some(Commands), false, CommandsListParams => CommandsListResult;
    CommandsInvoke, "commands.invoke", Some(Invocation), false, CommandsInvokeParams => CommandsInvokeResult;
    CommandInvoke, "command.invoke", Some(Commands), true, CommandInvokeParams => CommandsInvokeResult;
    CommandCancel, "command.cancel", Some(Commands), true, CommandCancelParams => Empty;
    KeymapsBind, "keymaps.bind", Some(Keymaps), false, KeymapsBindParams => KeymapsBindResult;
    KeymapsUnbind, "keymaps.unbind", Some(Keymaps), false, KeymapsUnbindParams => Empty;
    EventsSubscribe, "events.subscribe", Some(Events), false, EventsSubscribeParams => EventsSubscribeResult;
    EventsUnsubscribe, "events.unsubscribe", Some(Events), false, EventsUnsubscribeParams => Empty;
    MessagesShow, "messages.show", Some(Messages), false, MessagesShowParams => Empty;
    StateList, "state.list", Some(State), false, StateListParams => StateListResult;
    StateGet, "state.get", Some(State), false, StateGetParams => StateGetResult;
    StatePut, "state.put", Some(State), false, StatePutParams => StatePutResult;
    StateDelete, "state.delete", Some(State), false, StateDeleteParams => Empty;
}

/// Sole buffer event notification method; it never creates a response.
pub const BUFFER_EVENT: &str = "buffers.event";
