//! A captured update replaces only selected profiles still owned by the old
//! command/arguments. Whole-source comparison also protects their user options.
use super::{Installation, Outcome, State, Storage};
use crate::config::{Server, SettingsEdit};
use crate::org::Cancellation;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

struct Target {
    name: String,
    launch: String,
    server: Server,
}

pub struct Update {
    previous: Installation,
    targets: Vec<Target>,
}

impl Update {
    pub fn capture(
        edit: &SettingsEdit,
        previous: &Installation,
        data: &Path,
    ) -> Result<Self, String> {
        if previous.status != State::Ready
            || !super::SUPPORTED_RECIPES.contains(&previous.recipe.as_str())
        {
            return Err("update requires a ready supported installation".into());
        }
        let snapshot = edit.snapshot().map_err(|e| e.to_string())?;
        let policy = &snapshot.settings().lsp;
        let selected: BTreeSet<_> = policy
            .servers
            .values()
            .flat_map(|s| &s.languages)
            .filter_map(|language| policy.selected(language).map(|(name, _)| name.clone()))
            .collect();
        let payload = data.join(&previous.payload);
        let targets = selected
            .into_iter()
            .filter_map(|name| {
                let server = &policy.servers[&name];
                previous
                    .launches
                    .iter()
                    .find(|launch| owns(launch, server, &payload))
                    .map(|launch| Target {
                        name,
                        launch: launch.id.clone(),
                        server: server.clone(),
                    })
            })
            .collect();
        Ok(Self {
            previous: previous.clone(),
            targets,
        })
    }

    pub fn profiles(&self) -> Vec<&str> {
        self.targets.iter().map(|t| t.name.as_str()).collect()
    }

    pub fn run(
        self,
        storage: &mut Storage,
        edit: SettingsEdit,
        cancel: &Cancellation,
    ) -> Result<Outcome, String> {
        retained(storage, &self.previous)?;
        edit.check_unchanged().map_err(|e| e.to_string())?;
        let language = self
            .previous
            .launches
            .first()
            .and_then(|l| l.languages.first())
            .ok_or("old installation has no launch language")?;
        let mut outcome = super::install(
            storage,
            &self.previous.recipe,
            language,
            self.inherited(),
            cancel,
        )?;
        if cancel.is_cancelled() {
            outcome
                .message
                .push_str("; replacement ready, not selected after cancellation");
            return Ok(outcome);
        }
        if self.targets.is_empty() {
            outcome
                .message
                .push_str("; no selected profile referenced the old installation");
            return Ok(outcome);
        }
        let result = self.select(storage, edit, &outcome.installation, cancel);
        match result {
            Ok(publication) => {
                outcome.message.push_str(&format!(
                    "; updated {} selected profiles; prior version retained",
                    self.targets.len()
                ));
                if let Some(warning) = &publication.warning {
                    outcome.message.push_str(&format!("; {warning}"));
                }
                outcome.selection = Some(publication);
            }
            Err(error) => outcome
                .message
                .push_str(&format!("; replacement ready, not selected: {error}")),
        }
        Ok(outcome)
    }

    fn inherited(&self) -> BTreeMap<String, Server> {
        let mut inherited = BTreeMap::new();
        for target in &self.targets {
            for language in &target.server.languages {
                inherited.insert(language.clone(), target.server.clone());
            }
        }
        inherited
    }

    fn select(
        &self,
        storage: &Storage,
        mut edit: SettingsEdit,
        next: &Installation,
        cancel: &Cancellation,
    ) -> Result<crate::config::Publication, String> {
        self.retarget(storage, &mut edit, next)?;
        if cancel.is_cancelled() {
            return Err("cancelled before settings publication".into());
        }
        storage.select(&edit)
    }

    fn retarget(
        &self,
        storage: &Storage,
        edit: &mut SettingsEdit,
        next: &Installation,
    ) -> Result<(), String> {
        retained(storage, &self.previous)?;
        for target in &self.targets {
            let launch = next
                .launches
                .iter()
                .find(|l| l.id == target.launch)
                .ok_or("replacement lacks a referenced launch profile")?;
            let mut server = target.server.clone();
            server.command = storage
                .roots()
                .data
                .join(&next.payload)
                .join(&launch.executable)
                .to_string_lossy()
                .into_owned();
            server.args = launch.args.clone();
            super::apply_runtime(next, launch, &mut server)?;
            edit.register(&target.name, &server);
        }
        Ok(())
    }
}

fn owns(launch: &super::Launch, server: &Server, payload: &Path) -> bool {
    Path::new(&server.command) == payload.join(&launch.executable) && server.args == launch.args
}

pub(super) fn retained(storage: &Storage, previous: &Installation) -> Result<(), String> {
    if storage
        .registry()
        .installations
        .iter()
        .find(|i| i.id == previous.id)
        != Some(previous)
    {
        return Err("installation changed since the request; inspect inventory and retry".into());
    }
    Ok(())
}
