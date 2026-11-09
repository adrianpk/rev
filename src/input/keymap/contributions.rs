//! Plugin contributions resolve inside native maps, without another dispatcher.
use super::{Context, Map, Sequence};
use crate::command::{Binding, Command, PluginName};
use rev_plugin_sdk::{
    identity::PackageId,
    methods::{Context as WireContext, ResolutionStatus},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Contribution {
    pub context: Context,
    pub sequence: Sequence,
    pub command: PluginName,
    pub package: PackageId,
    pub local: bool,
    pub user: bool,
}
impl Contribution {
    fn rank(&self) -> u8 {
        u8::from(self.local) * 2 + u8::from(self.user)
    }
}
pub(crate) fn context(value: WireContext) -> Context {
    match value {
        WireContext::Editor => Context::Editor,
        WireContext::VimNormal => Context::VimNormal,
        WireContext::VimInsert => Context::VimInsert,
        WireContext::VimReplace => Context::VimReplace,
        WireContext::VimVisualCharacter => Context::VimVisualCharacter,
        WireContext::VimVisualLine => Context::VimVisualLine,
        WireContext::VimVisualBlock => Context::VimVisualBlock,
    }
}
pub(crate) const CONTEXTS: [Context; 7] = [
    Context::Editor,
    Context::VimNormal,
    Context::VimInsert,
    Context::VimReplace,
    Context::VimVisualCharacter,
    Context::VimVisualLine,
    Context::VimVisualBlock,
];
pub(crate) fn overlaps(a: &Sequence, b: &Sequence) -> bool {
    a.chords().starts_with(b.chords()) || b.chords().starts_with(a.chords())
}
impl Map {
    /// Preserve native recovery/state transitions and refuse unreachable chords.
    pub(crate) fn admit_plugin(&self, sequence: &Sequence) -> Result<(), String> {
        use crossterm::event::{KeyCode, KeyModifiers};
        if sequence
            .chords()
            .first()
            .is_some_and(|k| k.modifiers == KeyModifiers::ALT && k.code == KeyCode::Char('x'))
            || sequence
                .chords()
                .iter()
                .any(|k| k.control('g') || k.control(']') || k.code == KeyCode::Esc)
            || sequence.chords().first().is_some_and(|k| {
                k.modifiers == KeyModifiers::ALT
                    && matches!(k.code,KeyCode::Char(c) if c.is_ascii_digit() || c == '-')
            })
        {
            return Err(
                "plugin binding shadows native cancellation, Escape or prefix arguments".into(),
            );
        }
        for (key, target) in self.bindings() {
            if !overlaps(key, sequence) {
                continue;
            }
            if matches!(
                target,
                Binding::Command(
                    Command::KeyboardQuit | Command::ExecuteExtended | Command::Modal(_)
                )
            ) || matches!(target, Binding::Command(c) if c.prefix_control())
            {
                return Err(format!(
                    "plugin binding shadows native recovery or state transition: {key}"
                ));
            }
            if key != sequence {
                return Err(format!("unreachable native command/prefix overlap: {key}"));
            }
        }
        Ok(())
    }
    pub(crate) fn with_macros(
        &self,
        bindings: &std::collections::BTreeMap<Sequence, Command>,
    ) -> Self {
        let mut map = self.clone();
        for (sequence, command) in bindings {
            map.entries.retain(|key, _| !overlaps(key, sequence));
            map.entries.insert(
                sequence.clone(),
                super::map::Entry {
                    spelling: sequence.to_string(),
                    target: Binding::Command(*command),
                    custom: true,
                },
            );
        }
        map
    }
    /// Local scope precedes global scope; user choices precede defaults inside
    /// a scope. Equal-priority overlapping defaults never select an arrival winner.
    pub(crate) fn contributions(&self, values: &[Contribution]) -> (Self, Vec<ResolutionStatus>) {
        let mut map = self.clone();
        let mut statuses = vec![ResolutionStatus::Shadowed; values.len()];
        for rank in (0..=3).rev() {
            let eligible: Vec<_> = values
                .iter()
                .enumerate()
                .filter(|(_, v)| v.rank() == rank)
                .filter(|(_, v)| {
                    !(!v.local
                        && self
                            .entries
                            .iter()
                            .any(|(k, e)| e.custom && overlaps(k, &v.sequence)))
                })
                .filter(|(_, v)| {
                    !values.iter().zip(&statuses).any(|(other, status)| {
                        other.rank() > rank
                            && *status == ResolutionStatus::Effective
                            && overlaps(&other.sequence, &v.sequence)
                    })
                })
                .map(|(i, _)| i)
                .collect();
            for &i in &eligible {
                let value = &values[i];
                let conflict = eligible.iter().any(|&j| {
                    i != j
                        && overlaps(&values[j].sequence, &value.sequence)
                        && (values[j].command != value.command
                            || values[j].sequence != value.sequence)
                });
                statuses[i] = if conflict {
                    ResolutionStatus::Conflicting
                } else {
                    ResolutionStatus::Effective
                };
            }
        }
        for (value, status) in values.iter().zip(&statuses) {
            if *status != ResolutionStatus::Effective {
                continue;
            }
            map.entries.retain(|key, _| !overlaps(key, &value.sequence));
            map.entries.insert(
                value.sequence.clone(),
                super::map::Entry {
                    spelling: value.sequence.to_string(),
                    target: Binding::Command(Command::PluginDefined(value.command)),
                    custom: true,
                },
            );
        }
        (map, statuses)
    }
}

impl super::Effective {
    pub(crate) fn admit_plugin(&self, context: Context, sequence: &Sequence) -> Result<(), String> {
        if !matches!(
            context,
            Context::Editor | Context::VimInsert | Context::VimReplace
        ) && sequence
            .chords()
            .first()
            .and_then(|key| key.plain())
            .is_some_and(
                |key| matches!(key,crossterm::event::KeyCode::Char(c) if c.is_ascii_digit()),
            )
        {
            return Err("plugin binding shadows native modal count grammar".into());
        }
        self.map(context).admit_plugin(sequence)?;
        if context == Context::Editor {
            self.map(Context::Org).admit_plugin(sequence)?;
        } else {
            self.vim_map(context, Some(Context::Org))
                .admit_plugin(sequence)?;
        }
        Ok(())
    }
}
