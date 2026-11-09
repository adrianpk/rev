//! Pinned session targets and one checked publication path for native editing.
use super::*;
use crate::command::MacroName;
use crate::input::keymap::Sequence;

#[derive(Clone)]
pub(crate) enum Selection {
    Ring(Arc<Definition>),
    Named(MacroName, Arc<Definition>),
    Bound(Sequence, Arc<Definition>),
    New(Arc<Definition>),
}
impl Selection {
    pub fn source(&self) -> &Arc<Definition> {
        match self {
            Self::Ring(d) | Self::Named(_, d) | Self::Bound(_, d) | Self::New(d) => d,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Named(name, _) => name.name(),
            _ => "macro-draft",
        }
    }
}
impl State {
    pub fn replace_definition(
        &mut self,
        target: &Selection,
        mut definition: Definition,
        description: Option<String>,
        limit: usize,
    ) -> Result<(), String> {
        let source = target.source();
        definition.id = source.id;
        definition.revision = source
            .revision
            .checked_add(1)
            .ok_or("Macro definition revision exhausted")?;
        let candidate = Arc::new(definition);
        self.admit_memory(Some(&candidate))
            .map_err(|_| "Macro retained memory exceeds 48 MiB")?;
        let revision = self
            .catalog_revision
            .checked_add(1)
            .ok_or("Macro catalog revision exhausted")?;
        match target {
            Selection::Ring(source) => {
                let current = self
                    .ring
                    .iter_mut()
                    .find(|d| d.id == source.id)
                    .ok_or("Macro target was removed")?;
                if !Arc::ptr_eq(current, source) {
                    return Err("Macro target changed; draft retained".into());
                }
                *current = candidate;
            }
            Selection::Named(name, source) => {
                let named = self
                    .names
                    .get_mut(name)
                    .ok_or("Named macro target was removed")?;
                if !Arc::ptr_eq(&named.definition, source) {
                    return Err("Named macro target changed; draft retained".into());
                }
                named.source = (candidate.id, candidate.revision);
                named.definition = candidate;
                named.description = description;
            }
            Selection::Bound(key, source) => {
                let Some(Target::Snapshot(current)) = self.bindings.get_mut(key) else {
                    return Err("Bound macro target changed".into());
                };
                if !Arc::ptr_eq(current, source) {
                    return Err("Bound macro target changed; draft retained".into());
                }
                *current = candidate;
            }
            Selection::New(_) => {
                self.ring.push_front(candidate);
                self.ring.truncate(limit);
            }
        }
        self.catalog_revision = revision;
        Ok(())
    }
}
impl Step {
    pub fn summary(&self) -> String {
        match self {
            Self::Key(key) => crate::input::keymap::Sequence::from_chords(&[*key])
                .map_or_else(|| "key".into(), |s| s.to_string()),
            Self::Text(text) => format!("text {:?}", text.chars().take(48).collect::<String>()),
            Self::Command(c) => format!("{} [{} arguments]", c.name, c.arguments.len()),
            Self::Plugin(c) => format!("{} [plugin call]", c.command),
            Self::Completion(c) => format!("completion [{} edits]", c.edits.len()),
            Self::Counter(_) => "counter".into(),
            Self::Query(_) => "query".into(),
            Self::Redisplay => "redisplay".into(),
        }
    }
}
