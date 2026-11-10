use super::*;
use crate::git::families::{FamilyProof, Notes, NotesRef};

impl Runner {
    fn note_refs(
        &mut self,
        name: &std::ffi::OsStr,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Vec<NotesRef>, String> {
        let symbolic = self
            .command_os(
                &[
                    "symbolic-ref".into(),
                    "--quiet".into(),
                    "--no-recurse".into(),
                    name.into(),
                ],
                Phase::Checking,
                true,
                None,
            )?
            .0;
        if !symbolic.is_empty() {
            return Err(
                "Notes requires a direct full ref, including when the symbolic target is absent"
                    .into(),
            );
        }
        // The native branch catalog deliberately excludes notes. Capture their
        // own complete ref anchors so same-tree rewrites and aliases stay checked.
        let bytes = self.command(
            &[
                "for-each-ref",
                "--sort=refname",
                &format!("--count={}", self.policy.max_refs + 1),
                "--format=%(refname)%00%(objectname)%00%(symref)%00",
                "refs/notes/",
            ],
            Phase::Checking,
            false,
        )?;
        let mut references = Vec::new();
        for line in bytes.split_inclusive(|b| *b == b'\n') {
            if references.len() == self.policy.max_refs {
                return Err("Notes refs exceed complete count admission".into());
            }
            references.push(note_reference(line, format)?);
        }
        admission.grow(
            references.capacity() * std::mem::size_of::<NotesRef>()
                + references
                    .iter()
                    .map(|r| r.name.capacity() + r.symbolic.capacity())
                    .sum::<usize>(),
        )?;
        Ok(references)
    }
    fn note_entries(
        &mut self,
        name: &std::ffi::OsStr,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Vec<(ObjectId, ObjectId)>, String> {
        let bytes = self
            .command_os(
                &["notes".into(), "--ref".into(), name.into(), "list".into()],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        let mut entries = Vec::new();
        for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
            if entries.len() == self.policy.max_refs {
                return Err("Notes list exceeds complete count admission".into());
            }
            let split = line
                .iter()
                .position(|b| *b == b' ')
                .ok_or("Malformed notes object pair")?;
            entries.push((
                ObjectId::parse(&line[split + 1..], format)?,
                ObjectId::parse(&line[..split], format)?,
            ));
        }
        entries.sort_by_key(|(target, _)| target.hex());
        if entries.windows(2).any(|w| w[0].0 == w[1].0) {
            return Err("Notes list has duplicate targets".into());
        }
        admission.grow(entries.capacity() * std::mem::size_of::<(ObjectId, ObjectId)>())?;
        Ok(entries)
    }
    fn note_prune(
        &mut self,
        name: &std::ffi::OsStr,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<Vec<ObjectId>, String> {
        let bytes = self
            .command_os(
                &[
                    "notes".into(),
                    "--ref".into(),
                    name.into(),
                    "prune".into(),
                    "--dry-run".into(),
                ],
                Phase::Checking,
                false,
                None,
            )?
            .0;
        let mut objects = Vec::new();
        for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
            if objects.len() == self.policy.max_refs {
                return Err("Notes prune exceeds complete count admission".into());
            }
            objects.push(ObjectId::parse(line, format)?);
        }
        admission.grow(objects.capacity() * std::mem::size_of::<ObjectId>())?;
        Ok(objects)
    }
    pub(super) fn prepare_notes(
        &mut self,
        family: Family,
        args: &mut [OsString],
        source: &Arc<Sequence>,
        admission: &mut Lease,
    ) -> Result<Notes, String> {
        let name = args[2].clone();
        self.check_notes_source(&name, source)?;
        let configuration = self.transport_config(admission)?;
        let references =
            self.note_refs(&name, source.refs.observation.identity.format, admission)?;
        let entries =
            self.note_entries(&name, source.refs.observation.identity.format, admission)?;
        let target = self.note_target(
            family,
            args,
            &entries,
            source.refs.observation.identity.format,
        )?;
        let message = self.note_message(family, args)?;
        let prune = if family == Family::NotesPrune {
            self.note_prune(&name, source.refs.observation.identity.format, admission)?
        } else {
            Vec::new()
        };
        Ok(Notes {
            configuration,
            name,
            target,
            references,
            entries,
            prune,
            message,
        })
    }
    pub(super) fn execute_notes(
        &mut self,
        plan: &Arc<FamilyPlan>,
        admission: &mut Lease,
    ) -> Result<Candidate, String> {
        let FamilyProof::Notes(state) = &plan.proof else {
            return Err("Notes proof unavailable".into());
        };
        let format = plan
            .observation
            .as_ref()
            .expect("notes observation")
            .identity
            .format;
        self.check_notes_review(plan.family, state, format, admission)?;
        let (executed, facts) = self.execute_change(&plan.args, None);
        let failed = executed.is_err();
        if self.retained.is_some() {
            self.restore_execution(facts, true);
            return Err("Notes cleanup remains owned; no replay or rollback".into());
        }
        let verified = self.verify_notes(plan, state, executed, admission);
        let object = match verified {
            Ok(object) => object,
            Err(e) => {
                self.operation_failure = Some(format!(
                    "Notes result is unverified: {e}; inspect refreshed facts; no retry or rollback"
                ));
                None
            }
        };
        let result = self.family_observed_result(std::sync::Arc::clone(plan), object, admission);
        self.restore_execution(facts, failed);
        result
    }
    fn check_notes_source(
        &mut self,
        name: &std::ffi::OsStr,
        source: &Sequence,
    ) -> Result<(), String> {
        if !name.as_encoded_bytes().starts_with(b"refs/notes/") {
            return Err("Notes requires one explicit full refs/notes name".into());
        }
        self.command_os(
            &["check-ref-format".into(), name.to_owned()],
            Phase::Checking,
            false,
            None,
        )?;
        if source.family.is_some() || source.refs.completeness != Completeness::Complete {
            return Err(
                "Notes requires complete refs, no active sequence and a direct notes ref".into(),
            );
        }
        Ok(())
    }
    fn note_target(
        &mut self,
        family: Family,
        args: &mut [OsString],
        entries: &[(ObjectId, ObjectId)],
        format: ObjectFormat,
    ) -> Result<Option<ObjectId>, String> {
        let target = if family == Family::NotesPrune {
            None
        } else {
            let index = args.len() - 1;
            let object = self.family_object(&args[index], format)?;
            self.command(&["cat-file", "-e", &object.hex()], Phase::Checking, false)?;
            args[index] = object.hex().into();
            let exists = entries.iter().any(|(target, _)| *target == object);
            if (family == Family::NotesAdd) == exists {
                return Err(
                    "Notes add refuses overwrite; remove requires an existing exact target note"
                        .into(),
                );
            }
            Some(object)
        };
        Ok(target)
    }
    fn note_message(&mut self, family: Family, args: &[OsString]) -> Result<Vec<u8>, String> {
        let message = if family == Family::NotesAdd {
            let input = crate::process::input::stage(Some(args[5].as_encoded_bytes()))?;
            let bytes = self
                .command_argv(
                    self.git_invocation(&["stripspace".into()], input),
                    Phase::Checking,
                    false,
                    None,
                )?
                .0;
            if bytes.is_empty() {
                return Err("Notes message is empty after Git whitespace cleanup".into());
            }
            bytes
        } else {
            Vec::new()
        };
        Ok(message)
    }
    fn check_notes_review(
        &mut self,
        family: Family,
        state: &Notes,
        format: ObjectFormat,
        admission: &mut Lease,
    ) -> Result<(), String> {
        if self.transport_config(admission)? != state.configuration
            || self.note_refs(&state.name, format, admission)? != state.references
            || self.note_entries(&state.name, format, admission)? != state.entries
            || (family == Family::NotesPrune
                && self.note_prune(&state.name, format, admission)? != state.prune)
        {
            return Err(
                "Notes configuration, objects or prune candidates changed during review".into(),
            );
        }
        Ok(())
    }
    fn verify_notes(
        &mut self,
        plan: &FamilyPlan,
        state: &Notes,
        executed: Result<(), String>,
        admission: &mut Lease,
    ) -> Result<Option<ObjectId>, String> {
        let format = plan
            .observation
            .as_ref()
            .expect("notes observation")
            .identity
            .format;
        executed?;
        self.family_current(plan, Some(&state.name), admission)?;
        let actual = self.note_entries(&state.name, format, admission)?;
        let references = self.note_refs(&state.name, format, admission)?;
        if !references
            .iter()
            .filter(|r| r.name != state.name.as_encoded_bytes())
            .eq(state
                .references
                .iter()
                .filter(|r| r.name != state.name.as_encoded_bytes()))
        {
            return Err("Notes command changed another notes ref".into());
        }
        let NoteEffect {
            entries: expected,
            object,
        } = self.note_effect(plan, state, &actual)?;
        if actual != expected || self.transport_config(admission)? != state.configuration {
            return Err("Notes result changed another note or effective configuration".into());
        }
        self.effects = Effects::Verified;
        Ok(object)
    }
    fn note_effect(
        &mut self,
        plan: &FamilyPlan,
        state: &Notes,
        actual: &[(ObjectId, ObjectId)],
    ) -> Result<NoteEffect, String> {
        let mut expected = state.entries.clone();
        let object = match plan.family {
            Family::NotesAdd => {
                let target = state.target.expect("note target");
                let note = actual
                    .iter()
                    .find(|(t, _)| *t == target)
                    .map(|(_, n)| *n)
                    .ok_or("Added note is absent")?;
                let bytes =
                    self.command(&["cat-file", "blob", &note.hex()], Phase::Verifying, false)?;
                if bytes != state.message {
                    return Err(
                        "Added note bytes differ from the reviewed Git-cleaned message".into(),
                    );
                }
                expected.push((target, note));
                Some(note)
            }
            Family::NotesRemove => {
                expected.retain(|(t, _)| Some(*t) != state.target);
                None
            }
            Family::NotesPrune => {
                expected.retain(|(t, _)| !state.prune.contains(t));
                None
            }
            _ => unreachable!("typed notes family"),
        };
        expected.sort_by_key(|(target, _)| target.hex());
        Ok(NoteEffect {
            entries: expected,
            object,
        })
    }
}

/// Exact target-note postconditions and the one newly created object.
struct NoteEffect {
    entries: Vec<(ObjectId, ObjectId)>,
    object: Option<ObjectId>,
}

fn note_reference(line: &[u8], format: ObjectFormat) -> Result<NotesRef, String> {
    let line = line
        .strip_suffix(b"\0\n")
        .ok_or("Notes ref lacks its exact record boundary")?;
    let fields: Vec<_> = line.split(|b| *b == 0).collect();
    if fields.len() != 3
        || fields.iter().any(|f| f.len() > 4096)
        || !fields[0].starts_with(b"refs/notes/")
    {
        return Err("Malformed notes ref fields".into());
    }
    Ok(NotesRef {
        name: fields[0].to_vec(),
        object: ObjectId::parse(fields[1], format)?,
        symbolic: fields[2].to_vec(),
    })
}
