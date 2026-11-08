use super::*;
use crate::command::MacroName;
use std::collections::BTreeSet;

pub(crate) const NAME_LIMIT: usize = 256;
pub(crate) const CALL_LIMIT: usize = 16;
pub(crate) const RETAINED_BYTES: usize = 48 * 1024 * 1024;

pub(crate) struct Named {
    pub definition: Arc<Definition>,
    pub source: (u64, u64),
    pub description: Option<String>,
    pub saved: Option<library::Saved>,
}

#[derive(Clone)]
pub(crate) enum Target {
    Named(MacroName),
    Snapshot(Arc<Definition>),
}

impl State {
    pub fn snapshot(&mut self, source: &Definition) -> Result<Arc<Definition>, StopReason> {
        let id = self
            .next_id
            .checked_add(1)
            .ok_or(StopReason::CaptureLimit)?;
        let definition = Arc::new(Definition {
            id,
            revision: 1,
            ..source.clone()
        });
        self.admit_memory(Some(&definition))?;
        self.next_id = id;
        Ok(definition)
    }

    pub fn admit_memory(&self, extra: Option<&Arc<Definition>>) -> Result<(), StopReason> {
        let mut retained = Retained::default();
        retained.bytes += self.names.len() * 4096 + self.bindings.len() * 1024;
        for definition in self
            .ring
            .iter()
            .chain(self.names.values().map(|named| &named.definition))
        {
            retained.definition(definition);
        }
        for named in self.names.values() {
            if let Some(saved) = &named.saved {
                retained.definition(&saved.baseline);
            }
        }
        for target in self.bindings.values() {
            if let Target::Snapshot(definition) = target {
                retained.definition(definition);
            }
        }
        if let Some(definition) = extra {
            retained.definition(definition);
        }
        if let Some(recorder) = &self.recorder {
            retained.recorder(recorder);
        }
        if let Some(run) = &self.runner {
            retained.runner(run);
        }
        for pause in &self.pauses {
            if let Some(run) = &pause.runner {
                retained.runner(run);
            }
            if let Some(recorder) = &pause.recorder {
                retained.recorder(recorder);
            }
        }
        // Reserve one bounded native naming/binding draft and selector copies.
        (retained.bytes + 8192 <= RETAINED_BYTES)
            .then_some(())
            .ok_or(StopReason::RetainedMemory)
    }

    pub fn name_snapshot(&mut self, name: MacroName, named: Named) -> Result<(), String> {
        if !self.names.contains_key(&name) && self.names.len() == NAME_LIMIT {
            return Err("Session macro names exceed 256 records".into());
        }
        if named
            .description
            .as_ref()
            .is_some_and(|text| text.len() > 512)
        {
            return Err("Macro descriptions require at most 512 UTF-8 bytes".into());
        }
        self.admit_memory(Some(&named.definition))
            .map_err(|_| "Macro retained memory exceeds 48 MiB")?;
        let revision = self
            .catalog_revision
            .checked_add(1)
            .ok_or("Macro catalog revision exhausted")?;
        self.names.insert(name, named);
        self.catalog_revision = revision;
        Ok(())
    }
}

#[derive(Default)]
struct Retained {
    bytes: usize,
    definitions: BTreeSet<usize>,
    bodies: BTreeSet<usize>,
    regions: BTreeSet<usize>,
}
impl Retained {
    fn definition(&mut self, definition: &Arc<Definition>) {
        if self.definitions.insert(Arc::as_ptr(definition) as usize) {
            self.bytes += 4096;
        }
        if self
            .bodies
            .insert(Arc::as_ptr(&definition.steps) as *const Step as usize)
        {
            self.bytes += definition.retained_bytes;
        }
    }
    fn recorder(&mut self, recorder: &Recorder) {
        self.bytes += recorder.bytes
            + recorder.pending_bytes
            + (recorder.steps.len() + recorder.pending.len()) * std::mem::size_of::<Step>();
        if let Some(command) = &recorder.command {
            self.bytes += command.bytes();
        }
    }
    fn runner(&mut self, run: &Runner) {
        self.definition(&run.definition);
        self.bytes += 4096
            + run
                .trigger
                .iter()
                .map(|step| std::mem::size_of::<Step>() + step.bytes())
                .sum::<usize>();
        self.bytes += run
            .arguments
            .iter()
            .map(|arg| arg.value.len() + 64)
            .sum::<usize>();
        if let Some(region) = &run.region
            && self.regions.insert(Arc::as_ptr(&region.anchors) as usize)
        {
            self.bytes += region
                .anchors
                .lock()
                .expect("macro line owner poisoned")
                .positions
                .len()
                * std::mem::size_of::<Option<crate::coordinates::Scalar>>()
                + 256;
        }
        if let Some(parent) = &run.parent {
            self.runner(parent);
        }
    }
}
