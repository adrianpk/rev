use super::{Error, *};
use crate::command::{ArgumentForm, MacroName, PrefixArgument};
use crate::search::Direction;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value};

pub(super) const LIBRARY_BYTES: usize = 8 * 1024 * 1024;

// Collection admission happens in the serde visitor, before allocating another
// element. The enclosing native read caps all source/decoded strings at 8 MiB.
fn bounded<'de, D, T, const N: usize>(de: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Limit<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Limit<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "at most {N} elements")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while values.len() < N {
                match seq.next_element()? {
                    Some(v) => values.push(v),
                    None => return Ok(values),
                }
            }
            if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(serde::de::Error::custom("macro collection limit"));
            }
            Ok(values)
        }
    }
    de.deserialize_seq(Limit::<T, N>(std::marker::PhantomData))
}

fn key_list<'de, D: Deserializer<'de>>(de: D) -> Result<Option<Vec<String>>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Keys(#[serde(deserialize_with = "bounded::<_, _, 32>")] Vec<String>);
    Option::<Keys>::deserialize(de).map(|value| value.map(|keys| keys.0))
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Catalog {
    pub schema_version: u32,
    pub revision: u64,
    #[serde(deserialize_with = "bounded::<_, _, 256>")]
    pub macros: Vec<Record>,
}
impl Default for Catalog {
    fn default() -> Self {
        Self {
            schema_version: 1,
            revision: 0,
            macros: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub id: String,
    pub revision: u64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    profile: Profile,
    entry_state: Mode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<String>,
    counter: Count,
    // Replay begins with the captured initial format, independently of the
    // working counter's last format. Omission means the counter format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    initial_format: Option<String>,
    #[serde(deserialize_with = "bounded::<_, _, 4096>")]
    steps: Vec<Input>,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Profile {
    NonModal,
    Modal,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Editing,
    Normal,
    Insert,
    Replace,
    VisualCharacter,
    VisualLine,
    VisualBlock,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Count {
    current: i64,
    previous: i64,
    iteration_start: i64,
    format: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "form", rename_all = "kebab-case", deny_unknown_fields)]
enum RawArgument {
    Absent,
    PlainPrefix { depth: u8 },
    Integer { value: i32 },
    NegativeOnly,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Arg {
    role: PromptRole,
    value: String,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum PromptRole {
    Path,
    Buffer,
    Line,
    Write,
    Search,
    SearchForward,
    SearchBackward,
    ReplaceQuery,
    Replacement,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Edit {
    start: i64,
    end: i64,
    expected: String,
    text: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
enum CounterInput {
    Insert { argument: RawArgument },
    Set { value: i64 },
    Reset,
    Add { value: i64 },
    Restore,
    Format { format: String },
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Query {
    Confirm,
    Edit,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Input {
    PluginCall {
        command: String,
        arguments_json: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<i64>,
    },
    Key {
        key: String,
    },
    Text {
        text: String,
    },
    Command {
        name: String,
        argument: RawArgument,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "key_list"
        )]
        keys: Option<Vec<String>>,
        #[serde(deserialize_with = "bounded::<_, _, 32>")]
        arguments: Vec<Arg>,
    },
    Search(SearchInput),
    Completion {
        #[serde(deserialize_with = "bounded::<_, _, 4096>")]
        edits: Vec<Edit>,
        cursor: i64,
    },
    Counter {
        action: CounterInput,
    },
    Redisplay,
    Query {
        mode: Query,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum SearchDirection {
    Forward,
    Backward,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum SearchEntry {
    Incremental,
    Modal,
}
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Termination {
    Accept,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SearchRepeat {
    direction: SearchDirection,
    pattern: String,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SearchInput {
    direction: SearchDirection,
    entry: SearchEntry,
    argument: RawArgument,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "key_list"
    )]
    keys: Option<Vec<String>>,
    pattern: String,
    #[serde(deserialize_with = "bounded::<_, _, 31>")]
    continuations: Vec<SearchRepeat>,
    termination: Termination,
}
impl SearchInput {
    fn matches(command: &CommandStep) -> bool {
        matches!(
            command.name.as_str(),
            "isearch-forward"
                | "isearch-backward"
                | "modal-search-forward"
                | "modal-search-backward"
        ) && matches!(
            command.arguments.last(),
            Some(Argument {
                role: Role::Search,
                ..
            })
        ) && command.arguments[..command.arguments.len() - 1]
            .iter()
            .all(|a| matches!(a.role, Role::SearchRepeat(_)))
    }
    fn capture(command: &CommandStep) -> Result<Self, Error> {
        let (entry, direction) = match command.name.as_str() {
            "isearch-forward" => (SearchEntry::Incremental, SearchDirection::Forward),
            "isearch-backward" => (SearchEntry::Incremental, SearchDirection::Backward),
            "modal-search-forward" => (SearchEntry::Modal, SearchDirection::Forward),
            "modal-search-backward" => (SearchEntry::Modal, SearchDirection::Backward),
            _ => return Err(Error::Invalid("search entry")),
        };
        let pattern = command
            .arguments
            .last()
            .ok_or(Error::Invalid("accepted search pattern"))?
            .value
            .clone();
        let continuations = command.arguments[..command.arguments.len() - 1]
            .iter()
            .map(|arg| match arg.role {
                Role::SearchRepeat(direction) => Ok(SearchRepeat {
                    direction: SearchDirection::from_native(direction),
                    pattern: arg.value.clone(),
                }),
                _ => Err(Error::Invalid("search continuation")),
            })
            .collect::<Result<_, _>>()?;
        let keys = command
            .keys
            .as_ref()
            .map(|keys| {
                keys.iter()
                    .map(|s| match s {
                        Step::Key(k) => key_text(*k),
                        _ => Err(Error::Invalid("search trigger")),
                    })
                    .collect()
            })
            .transpose()?;
        Ok(Self {
            direction,
            entry,
            argument: RawArgument::from_native(command.argument),
            keys,
            pattern,
            continuations,
            termination: Termination::Accept,
        })
    }
    fn native(&self) -> Result<Step, Error> {
        if self.pattern.contains('\0')
            || self.continuations.iter().any(|c| c.pattern.contains('\0'))
            || self
                .keys
                .as_ref()
                .is_some_and(|k| k.is_empty() || k.len() > 32)
            || self.continuations.len() > 31
        {
            return Err(Error::Invalid("search pattern or continuation"));
        }
        let name = self.command_name();
        let mut arguments: Vec<_> = self
            .continuations
            .iter()
            .map(|c| Argument {
                role: Role::SearchRepeat(c.direction.native()),
                value: c.pattern.clone(),
            })
            .collect();
        arguments.push(Argument {
            role: Role::Search,
            value: self.pattern.clone(),
        });
        Ok(Step::Command(CommandStep {
            name: name.into(),
            argument: self.argument.native()?,
            keys: self
                .keys
                .as_ref()
                .map(|keys| keys.iter().map(|k| parse_key(k).map(Step::Key)).collect())
                .transpose()?,
            arguments,
        }))
    }
    fn command_name(&self) -> &'static str {
        match (self.entry, self.direction) {
            (SearchEntry::Incremental, SearchDirection::Forward) => "isearch-forward",
            (SearchEntry::Incremental, SearchDirection::Backward) => "isearch-backward",
            (SearchEntry::Modal, SearchDirection::Forward) => "modal-search-forward",
            (SearchEntry::Modal, SearchDirection::Backward) => "modal-search-backward",
        }
    }
}
impl SearchDirection {
    fn from_native(direction: Direction) -> Self {
        match direction {
            Direction::Forward => Self::Forward,
            Direction::Backward => Self::Backward,
        }
    }
    fn native(self) -> Direction {
        match self {
            Self::Forward => Direction::Forward,
            Self::Backward => Direction::Backward,
        }
    }
}

impl Catalog {
    pub fn decode(bytes: Option<&[u8]>) -> Result<Self, Error> {
        let Some(bytes) = bytes else {
            return Ok(Self::default());
        };
        if bytes.len() > LIBRARY_BYTES {
            return Err(Error::Capacity("8 MiB encoded library"));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid("UTF-8"))?;
        let catalog: Self =
            toml::from_str(text).map_err(|_| Error::Invalid("macro TOML fields or bounds"))?;
        catalog.validate()?;
        Ok(catalog)
    }
    fn validate(&self) -> Result<(), Error> {
        self.validate_header()?;
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut retained = 0usize;
        for record in &self.macros {
            if self.schema_version == 1 && record.schema() > 1 {
                return Err(Error::Invalid("plugin-call requires schema 2"));
            }
            if !ids.insert(&record.id) || !names.insert(&record.name) {
                return Err(Error::Invalid("duplicate identity or name"));
            }
            let definition = record.definition(0)?;
            retained = retained.saturating_add(definition.retained_bytes + 4096);
            if retained > super::super::names::RETAINED_BYTES {
                return Err(Error::Capacity("48 MiB decoded definitions"));
            }
        }
        Ok(())
    }
    fn validate_header(&self) -> Result<(), Error> {
        if !matches!(self.schema_version, 1 | 2)
            || self.revision == 0
            || self.revision > i64::MAX as u64
        {
            return Err(Error::Invalid("schema version or aggregate revision"));
        }
        if self.macros.len() > NAME_LIMIT {
            return Err(Error::Capacity("256 saved records"));
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        self.validate()?;
        let mut doc = DocumentMut::new();
        doc["schema_version"] = toml_edit::value(i64::from(self.schema_version));
        doc["revision"] = toml_edit::value(
            i64::try_from(self.revision).expect("validated macro catalog revision"),
        );
        let mut records = ArrayOfTables::new();
        for record in &self.macros {
            records.push(record.table()?);
        }
        doc["macros"] = if records.is_empty() {
            Item::Value(Value::Array(Array::new()))
        } else {
            Item::ArrayOfTables(records)
        };
        let bytes = doc.to_string().into_bytes();
        if bytes.len() > LIBRARY_BYTES {
            return Err(Error::Capacity("8 MiB encoded library"));
        }
        Ok(bytes)
    }
}
impl Record {
    pub(super) fn schema(&self) -> u32 {
        if self
            .steps
            .iter()
            .any(|step| matches!(step, Input::PluginCall { .. }))
        {
            2
        } else {
            1
        }
    }
    pub fn capture(
        name: MacroName,
        named: &Named,
        binding: Option<&Sequence>,
    ) -> Result<Self, Error> {
        let id = match &named.saved {
            Some(s) => s.id.clone(),
            None => identity()?,
        };
        let revision = named.saved.as_ref().map_or(1, |s| s.revision);
        let entry = named.definition.entry;
        let (profile, entry_state) = match (entry.profile, entry.mode) {
            (InteractionProfile::NonModal, None) => (Profile::NonModal, Mode::Editing),
            (InteractionProfile::Modal, Some(state)) => (Profile::Modal, Mode::from_native(state)),
            _ => return Err(Error::Invalid("entry profile/state")),
        };
        let counter = &named.definition.counter;
        let record = Self {
            id,
            revision,
            name: name.name().into(),
            description: named.description.clone(),
            profile,
            entry_state,
            binding: binding.map(ToString::to_string),
            counter: Count {
                current: counter.current,
                previous: counter.previous,
                iteration_start: counter.iteration_start,
                format: counter.format.clone(),
            },
            initial_format: (named.definition.initial_format != counter.format)
                .then(|| named.definition.initial_format.clone()),
            steps: named
                .definition
                .steps
                .iter()
                .map(Input::from_native)
                .collect::<Result<_, _>>()?,
        };
        record.definition(0)?;
        Ok(record)
    }
    pub fn export(&self) -> Result<String, Error> {
        let doc = Catalog {
            schema_version: self.schema(),
            revision: 1,
            macros: vec![self.clone()],
        }
        .encode()?;
        String::from_utf8(doc).map_err(|_| Error::Invalid("encoded UTF-8"))
    }
    fn table(&self) -> Result<Table, Error> {
        let object = serde_json::to_value(self).map_err(|_| Error::Invalid("record encoding"))?;
        let mut table = Table::new();
        for (key, value) in object.as_object().ok_or(Error::Invalid("record table"))? {
            table[key] = Item::Value(toml_value(value)?);
        }
        if table.to_string().len() > DEFINITION_BYTES {
            return Err(Error::Capacity("256 KiB encoded definition"));
        }
        Ok(table)
    }
    pub fn definition(&self, id: u64) -> Result<Definition, Error> {
        self.validate_identity()?;
        let entry = self.native_entry()?;
        if self.steps.len() > STEP_LIMIT {
            return Err(Error::Capacity("4096 steps"));
        }
        counter::render(&self.counter.format, i64::MIN).map_err(Error::Invalid)?;
        let initial = self.initial_format.as_ref().unwrap_or(&self.counter.format);
        counter::render(initial, i64::MIN).map_err(Error::Invalid)?;
        self.table()?;
        let steps: Vec<Step> = self
            .steps
            .iter()
            .map(Input::native)
            .collect::<Result<_, _>>()?;
        let retained_bytes = steps
            .iter()
            .map(|s| s.bytes().saturating_add(std::mem::size_of::<Step>()))
            .sum();
        Ok(Definition {
            retained_bytes,
            id,
            revision: 1,
            counter: Counter {
                current: self.counter.current,
                previous: self.counter.previous,
                iteration_start: self.counter.iteration_start,
                format: self.counter.format.clone(),
            },
            initial_format: initial.clone(),
            entry,
            steps: steps.into(),
        })
    }
    fn validate_identity(&self) -> Result<(), Error> {
        if self.id.len() != 32
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.revision == 0
            || self.revision > i64::MAX as u64
        {
            return Err(Error::Invalid("record identity/revision"));
        }
        MacroName::stored_name(&self.name).map_err(|_| Error::Invalid("command name"))?;
        if self.description.as_ref().is_some_and(|s| s.len() > 512) {
            return Err(Error::Capacity("512-byte description"));
        }
        if let Some(binding) = &self.binding {
            Sequence::parse(binding).map_err(|_| Error::Invalid("native binding"))?;
        }
        Ok(())
    }

    fn native_entry(&self) -> Result<Entry, Error> {
        Ok(match (self.profile, self.entry_state) {
            (Profile::NonModal, Mode::Editing) => Entry {
                profile: InteractionProfile::NonModal,
                mode: None,
            },
            (Profile::Modal, mode) if mode != Mode::Editing => Entry {
                profile: InteractionProfile::Modal,
                mode: Some(mode.native()?),
            },
            _ => return Err(Error::Invalid("entry profile/state")),
        })
    }
}
fn identity() -> Result<String, Error> {
    use std::io::Read;
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|_| Error::Unavailable("cannot allocate saved identity"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn toml_value(value: &serde_json::Value) -> Result<Value, Error> {
    Ok(match value {
        serde_json::Value::String(s) => Value::from(s.as_str()),
        serde_json::Value::Number(n) => {
            Value::from(n.as_i64().ok_or(Error::Invalid("TOML integer"))?)
        }
        serde_json::Value::Array(values) => {
            let mut array = Array::new();
            for v in values {
                array.push(toml_value(v)?);
            }
            Value::Array(array)
        }
        serde_json::Value::Object(values) => {
            let mut table = InlineTable::new();
            for (k, v) in values {
                table.insert(k, toml_value(v)?);
            }
            Value::InlineTable(table)
        }
        _ => return Err(Error::Invalid("TOML field type")),
    })
}
impl RawArgument {
    fn from_native(value: PrefixArgument) -> Self {
        match value.form() {
            ArgumentForm::Absent => Self::Absent,
            ArgumentForm::PlainPrefix { depth } => Self::PlainPrefix { depth },
            ArgumentForm::Integer(value) => Self::Integer { value },
            ArgumentForm::NegativeOnly => Self::NegativeOnly,
        }
    }
    fn native(&self) -> Result<PrefixArgument, Error> {
        let mut value = PrefixArgument::default();
        match *self {
            Self::Absent => {}
            Self::PlainPrefix { depth } => {
                if depth == 0 {
                    return Err(Error::Invalid("prefix depth"));
                }
                for _ in 0..depth {
                    value = value.extend().ok_or(Error::Invalid("prefix depth"))?;
                }
            }
            Self::Integer { value: n } => extend_integer_argument(&mut value, n)?,
            Self::NegativeOnly => value = value.negate(),
        }
        Ok(value)
    }
}
impl Mode {
    fn from_native(state: EditingState) -> Self {
        match state {
            EditingState::Normal => Self::Normal,
            EditingState::Insert => Self::Insert,
            EditingState::Replace => Self::Replace,
            EditingState::VisualCharacter => Self::VisualCharacter,
            EditingState::VisualLine => Self::VisualLine,
            EditingState::VisualBlock => Self::VisualBlock,
        }
    }
    fn native(self) -> Result<EditingState, Error> {
        Ok(match self {
            Self::Normal => EditingState::Normal,
            Self::Insert => EditingState::Insert,
            Self::Replace => EditingState::Replace,
            Self::VisualCharacter => EditingState::VisualCharacter,
            Self::VisualLine => EditingState::VisualLine,
            Self::VisualBlock => EditingState::VisualBlock,
            Self::Editing => return Err(Error::Invalid("modal state")),
        })
    }
}
impl PromptRole {
    fn from_native(role: Role) -> Self {
        match role {
            Role::Path => Self::Path,
            Role::Buffer => Self::Buffer,
            Role::Line => Self::Line,
            Role::Write => Self::Write,
            Role::Search => Self::Search,
            Role::SearchRepeat(Direction::Forward) => Self::SearchForward,
            Role::SearchRepeat(Direction::Backward) => Self::SearchBackward,
            Role::ReplaceQuery => Self::ReplaceQuery,
            Role::Replacement => Self::Replacement,
        }
    }
    fn native(self) -> Role {
        match self {
            Self::Path => Role::Path,
            Self::Buffer => Role::Buffer,
            Self::Line => Role::Line,
            Self::Write => Role::Write,
            Self::Search => Role::Search,
            Self::SearchForward => Role::SearchRepeat(Direction::Forward),
            Self::SearchBackward => Role::SearchRepeat(Direction::Backward),
            Self::ReplaceQuery => Role::ReplaceQuery,
            Self::Replacement => Role::Replacement,
        }
    }
}
impl Input {
    fn from_native(step: &Step) -> Result<Self, Error> {
        match step {
            Step::Plugin(call) => Self::plugin_from_native(call),
            Step::Key(key) => key_text(*key).map(|key| Self::Key { key }),
            Step::Text(text) => Ok(Self::Text { text: text.clone() }),
            Step::Command(command) => Self::command_from_native(command),
            Step::Completion(c) => Ok(Self::Completion {
                edits: c
                    .edits
                    .iter()
                    .map(|e| Edit {
                        start: e.start,
                        end: e.end,
                        expected: e.expected.clone(),
                        text: e.text.clone(),
                    })
                    .collect(),
                cursor: c.cursor,
            }),
            Step::Counter(op) => Ok(Self::Counter {
                action: CounterInput::from_native(op),
            }),
            Step::Redisplay => Ok(Self::Redisplay),
            Step::Query(mode) => Ok(Self::Query {
                mode: Query::from_native(*mode),
            }),
        }
    }

    fn plugin_from_native(call: &PluginCall) -> Result<Self, Error> {
        Ok(Self::PluginCall {
            command: call.command.to_string(),
            arguments_json: serde_json::to_string(&call.arguments)
                .map_err(|_| Error::Invalid("plugin arguments"))?,
            prefix: call.prefix,
        })
    }

    fn command_from_native(c: &CommandStep) -> Result<Self, Error> {
        if SearchInput::matches(c) {
            return Ok(Self::Search(SearchInput::capture(c)?));
        }
        Ok(Self::Command {
            name: c.name.clone(),
            argument: RawArgument::from_native(c.argument),
            keys: c
                .keys
                .as_ref()
                .map(|keys| {
                    keys.iter()
                        .map(|s| match s {
                            Step::Key(k) => key_text(*k),
                            _ => Err(Error::Invalid("command trigger key")),
                        })
                        .collect()
                })
                .transpose()?,
            arguments: c
                .arguments
                .iter()
                .map(|a| Arg {
                    role: PromptRole::from_native(a.role),
                    value: a.value.clone(),
                })
                .collect(),
        })
    }

    fn native(&self) -> Result<Step, Error> {
        match self {
            Self::PluginCall {
                command,
                arguments_json,
                prefix,
            } => native_plugin(command, arguments_json, *prefix),
            Self::Key { key } => parse_key(key).map(Step::Key),
            Self::Text { text } => native_text(text),
            Self::Command {
                name,
                argument,
                keys,
                arguments,
            } => native_command(name, argument, keys.as_deref(), arguments),
            Self::Search(search) => search.native(),
            Self::Completion { edits, cursor } => native_completion(edits, *cursor),
            Self::Counter { action } => action.native().map(Step::Counter),
            Self::Redisplay => Ok(Step::Redisplay),
            Self::Query { mode } => Ok(Step::Query(mode.native())),
        }
    }
}

impl CounterInput {
    fn from_native(operation: &counter::Operation) -> Self {
        match operation {
            counter::Operation::Insert(p) => Self::Insert {
                argument: RawArgument::from_native(*p),
            },
            counter::Operation::Set(value) => Self::Set { value: *value },
            counter::Operation::Reset => Self::Reset,
            counter::Operation::Add(value) => Self::Add { value: *value },
            counter::Operation::Restore => Self::Restore,
            counter::Operation::Format(format) => Self::Format {
                format: format.clone(),
            },
        }
    }

    fn native(&self) -> Result<counter::Operation, Error> {
        Ok(match self {
            Self::Insert { argument } => counter::Operation::Insert(argument.native()?),
            Self::Set { value } => counter::Operation::Set(*value),
            Self::Reset => counter::Operation::Reset,
            Self::Add { value } => counter::Operation::Add(*value),
            Self::Restore => counter::Operation::Restore,
            Self::Format { format } => {
                counter::render(format, i64::MIN).map_err(Error::Invalid)?;
                counter::Operation::Format(format.clone())
            }
        })
    }
}

impl Query {
    fn from_native(mode: QueryMode) -> Self {
        match mode {
            QueryMode::Confirm => Self::Confirm,
            QueryMode::Edit => Self::Edit,
        }
    }

    fn native(self) -> QueryMode {
        match self {
            Self::Confirm => QueryMode::Confirm,
            Self::Edit => QueryMode::Edit,
        }
    }
}

fn native_plugin(command: &str, arguments_json: &str, prefix: Option<i64>) -> Result<Step, Error> {
    let call = PluginCall {
        command: rev_plugin_sdk::identity::CommandId::new(command)
            .map_err(|_| Error::Invalid("qualified plugin identity"))?,
        arguments: serde_json::from_str(arguments_json)
            .map_err(|_| Error::Invalid("plugin argument JSON"))?,
        prefix,
    };
    call.validate().map_err(Error::Invalid)?;
    Ok(Step::Plugin(call))
}

fn native_text(text: &str) -> Result<Step, Error> {
    if text.contains('\0') {
        return Err(Error::Invalid("NUL text"));
    }
    Ok(Step::Text(text.into()))
}

fn native_command(
    name: &str,
    argument: &RawArgument,
    keys: Option<&[String]>,
    arguments: &[Arg],
) -> Result<Step, Error> {
    if MacroName::stored_name(name).is_err() || keys.is_some_and(|k| k.is_empty() || k.len() > 32) {
        return Err(Error::Invalid("command name or trigger"));
    }
    Ok(Step::Command(CommandStep {
        name: name.into(),
        argument: argument.native()?,
        keys: keys
            .map(|ks| ks.iter().map(|k| parse_key(k).map(Step::Key)).collect())
            .transpose()?,
        arguments: arguments
            .iter()
            .map(|a| Argument {
                role: a.role.native(),
                value: a.value.clone(),
            })
            .collect(),
    }))
}

fn native_completion(edits: &[Edit], cursor: i64) -> Result<Step, Error> {
    if edits.iter().any(|e| {
        e.end < e.start
            || !i64::try_from(e.expected.chars().count())
                .is_ok_and(|count| e.end.checked_sub(e.start) == Some(count))
            || e.text.contains('\0')
    }) {
        return Err(Error::Invalid("completion source precondition"));
    }
    Ok(Step::Completion(LiteralCompletion {
        edits: edits
            .iter()
            .map(|e| CompletionEdit {
                start: e.start,
                end: e.end,
                expected: e.expected.clone(),
                text: e.text.clone(),
            })
            .collect(),
        cursor,
    }))
}

fn extend_integer_argument(value: &mut PrefixArgument, n: i32) -> Result<(), Error> {
    if n.unsigned_abs() > PrefixArgument::LIMIT as u32 {
        return Err(Error::Invalid("prefix integer"));
    }
    if n < 0 {
        *value = value.negate();
    }
    for digit in n.unsigned_abs().to_string().bytes() {
        *value = value
            .digit(u32::from(digit - b'0'))
            .ok_or(Error::Invalid("prefix integer"))?;
    }
    Ok(())
}

fn key_text(key: Key) -> Result<String, Error> {
    Sequence::from_chords(&[key])
        .map(|s| s.to_string())
        .ok_or(Error::Invalid("native key"))
}
fn parse_key(text: &str) -> Result<Key, Error> {
    // ESC is itself a recorded key, while sequence syntax reserves ESC as Meta.
    if text == "ESC" {
        return Ok(Key {
            code: crossterm::event::KeyCode::Esc,
            modifiers: crossterm::event::KeyModifiers::NONE,
        });
    }
    let sequence = Sequence::parse(text).map_err(|_| Error::Invalid("normalized native key"))?;
    if sequence.chords().len() != 1 || sequence.to_string() != text {
        return Err(Error::Invalid("one normalized native key"));
    }
    Ok(sequence.chords()[0])
}

#[cfg(test)]
#[path = "codec/tests.rs"]
pub(super) mod tests;

impl Record {
    /// A session edit pins its process-local target separately from any saved
    /// identity/revision. Applying a draft never advances persistent metadata.
    pub fn draft(
        target: &super::super::editing::Selection,
        description: Option<String>,
    ) -> Result<Self, Error> {
        let source = target.source();
        let named = Named {
            definition: std::sync::Arc::clone(source),
            source: (source.id, source.revision),
            description,
            saved: None,
        };
        let mut record = Self::capture(
            MacroName::new(target.name()).map_err(|_| Error::Invalid("draft name"))?,
            &named,
            None,
        )?;
        record.id = format!("{:032x}", source.id);
        record.revision = source.revision;
        Ok(record)
    }
    pub fn draft_text(&self) -> Result<String, Error> {
        self.definition(0)?;
        let mut table = self.table()?;
        let steps = table.remove("steps").ok_or(Error::Invalid("draft steps"))?;
        let values = steps.as_array().ok_or(Error::Invalid("draft step array"))?;
        if values.is_empty() {
            table["steps"] = toml_edit::value(Array::new());
        } else {
            let mut tables = ArrayOfTables::new();
            for value in values {
                let inline = value
                    .as_inline_table()
                    .ok_or(Error::Invalid("draft step table"))?;
                let mut step = Table::new();
                for (key, value) in inline.iter() {
                    step[key] = Item::Value(value.clone());
                }
                tables.push(step);
            }
            table["steps"] = Item::ArrayOfTables(tables);
        }
        let mut doc = DocumentMut::new();
        doc["schema_version"] = toml_edit::value(i64::from(self.schema()));
        doc["revision"] = toml_edit::value(1);
        let mut records = ArrayOfTables::new();
        records.push(table);
        doc["macros"] = Item::ArrayOfTables(records);
        Ok(doc.to_string())
    }
    pub fn key_notation(keys: &Sequence) -> Result<String, Error> {
        let mut text = String::new();
        for key in keys.chords() {
            text.push_str("[[macros.steps]]\nkind = \"key\"\nkey = ");
            text.push_str(&Value::from(key_text(*key)?).to_string());
            text.push('\n');
        }
        Ok(text)
    }
    pub fn region_body(&self, fragment: &str) -> Result<Self, Error> {
        let mut base = self.clone();
        base.steps.clear();
        // Definition edits can insert a new plugin step without writing or
        // migrating the attached library. Draft admission is inert schema 2.
        let mut text = base.draft_text()?.replace("steps = []\n", "").replacen(
            "schema_version = 1",
            "schema_version = 2",
            1,
        );
        text.push_str(fragment);
        let mut catalog = Catalog::decode(Some(text.as_bytes()))?;
        if catalog.macros.len() != 1 {
            return Err(Error::Invalid("one definition body"));
        }
        let mut candidate = catalog.macros.remove(0);
        let steps = std::mem::take(&mut candidate.steps);
        if candidate != base {
            return Err(Error::Invalid("selected lines contain definition metadata"));
        }
        candidate.steps = steps;
        candidate.definition(0)?;
        Ok(candidate)
    }
}
