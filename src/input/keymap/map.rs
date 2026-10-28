use std::collections::BTreeMap;

use crate::command::{Binding, Command};

use super::{Context, Sequence};

/// The error site selects the original TOML key or value span when available.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    Key,
    Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub context: Context,
    pub sequence: String,
    pub message: String,
    pub site: Site,
}

impl Error {
    pub(super) fn new(
        context: Context,
        sequence: &str,
        site: Site,
        message: impl Into<String>,
    ) -> Self {
        Self {
            context,
            sequence: sequence.into(),
            site,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    spelling: String,
    target: Binding,
    custom: bool,
}

/// One fully validated inherited map. Raw spellings survive only for useful
/// diagnostics; normalized sequences own lookup, replacement and prefix identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    entries: BTreeMap<Sequence, Entry>,
}

impl Map {
    pub(super) fn compile(
        context: Context,
        overrides: &BTreeMap<String, String>,
    ) -> Result<Self, Error> {
        Self::builtins(context).apply(context, overrides)
    }

    pub(super) fn inherit(
        parent: &Self,
        context: Context,
        overrides: &BTreeMap<String, String>,
    ) -> Result<Self, Error> {
        let mut map = parent.clone();
        map.entries.extend(Self::builtins(context).entries);
        map.apply(context, overrides)
    }

    fn apply(
        mut self,
        context: Context,
        overrides: &BTreeMap<String, String>,
    ) -> Result<Self, Error> {
        let mut incoming = BTreeMap::new();
        for (spelling, name) in overrides {
            let sequence = Sequence::parse(spelling)
                .map_err(|error| Error::new(context, spelling, Site::Key, error.0))?;
            let target = context.target(spelling, name)?;
            super::cancellation::admit(context, spelling, &sequence, target)?;
            if let Some(previous) = incoming.insert(
                sequence,
                Entry {
                    spelling: spelling.clone(),
                    target,
                    custom: true,
                },
            ) {
                return Err(Error::new(
                    context,
                    spelling,
                    Site::Key,
                    format!(
                        "normalized sequence duplicates '{}' (terminal aliases cannot be distinguished)",
                        previous.spelling
                    ),
                ));
            }
        }
        self.entries.extend(incoming);
        self.conflicts(context)?;
        Ok(self)
    }

    fn builtins(context: Context) -> Self {
        let entries = context
            .defaults()
            .iter()
            .map(|(sequence, name)| {
                let key = Sequence::parse(sequence).expect("audited built-in sequence");
                let entry = Entry {
                    spelling: (*sequence).into(),
                    target: Binding::named(name).expect("audited built-in action"),
                    custom: false,
                };
                (key, entry)
            })
            .collect();
        Self { entries }
    }

    pub fn lookup(&self, sequence: &Sequence) -> Option<Binding> {
        self.entries.get(sequence).map(|entry| entry.target)
    }

    /// Inherited entries preserve native stage-specific key distinctions;
    /// explicit aliases select the named action at its canonical owner.
    pub(crate) fn is_custom(&self, sequence: &Sequence) -> bool {
        self.entries.get(sequence).is_some_and(|entry| entry.custom)
    }

    pub fn is_prefix(&self, sequence: &Sequence) -> bool {
        self.entries
            .range(sequence.clone()..)
            .next()
            .is_some_and(|(next, _)| {
                next != sequence && next.chords().starts_with(sequence.chords())
            })
    }

    /// Decision stages may wait only for cancellation sequences. Other mapped
    /// actions cannot consume an answer as a suffix and accidentally consent.
    pub(crate) fn is_cancel_prefix(&self, sequence: &Sequence) -> bool {
        self.entries
            .range(sequence.clone()..)
            .take_while(|(next, _)| next.chords().starts_with(sequence.chords()))
            .any(|(next, entry)| {
                next != sequence && entry.target == Binding::Command(Command::KeyboardQuit)
            })
    }

    pub fn bindings(&self) -> impl Iterator<Item = (&Sequence, Binding)> {
        self.entries
            .iter()
            .map(|(sequence, entry)| (sequence, entry.target))
    }

    fn conflicts(&self, context: Context) -> Result<(), Error> {
        // Lexicographic neighbors suffice: a leaf sorts immediately before its
        // first descendant. This keeps validation linear after map construction.
        for ((prefix, first), (sequence, second)) in
            self.entries.iter().zip(self.entries.iter().skip(1))
        {
            if sequence.chords().starts_with(prefix.chords()) {
                return Err(conflict(context, first, second));
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "keys.{}.{}: {}",
            self.context.name(),
            self.sequence,
            self.message
        )
    }
}

impl std::error::Error for Error {}

fn conflict(context: Context, first: &Entry, second: &Entry) -> Error {
    let implicated = if first.custom { first } else { second };
    Error::new(
        context,
        &implicated.spelling,
        Site::Key,
        format!(
            "command/prefix conflict between '{}' and '{}'",
            first.spelling, second.spelling
        ),
    )
}
