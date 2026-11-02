use crate::config::{Preference, Server, SettingsEdit, Snapshot};
use crate::session::{Action, Pending, Session};

enum Stage {
    Recipe,
    Language,
    Profile,
    Executable,
    Arguments,
    Markers,
    Patterns,
}

pub(in crate::session) struct Prompt {
    edit: SettingsEdit,
    snapshot: Snapshot,
    stage: Stage,
    language: String,
    name: String,
    server: Option<Server>,
    select_only: bool,
    management: Option<super::managed::Choice>,
}

impl Session {
    pub(in crate::session) fn begin_lsp_setup(&mut self) -> Action {
        self.begin_lsp_choice(false)
    }

    pub(in crate::session) fn begin_lsp_selection(&mut self) -> Action {
        self.begin_lsp_choice(true)
    }

    fn begin_lsp_choice(&mut self, select_only: bool) -> Action {
        self.begin_lsp_management(select_only, false)
    }

    pub(in crate::session) fn begin_lsp_installation(&mut self) -> Action {
        self.begin_lsp_management(false, true)
    }

    fn begin_lsp_management(&mut self, select_only: bool, install_only: bool) -> Action {
        let result = self.lsp_settings_root().and_then(|root| {
            let edit = SettingsEdit::capture(&root).map_err(|e| e.to_string())?;
            let snapshot = edit.snapshot().map_err(|e| e.to_string())?;
            Ok(Prompt {
                edit,
                snapshot,
                stage: if install_only {
                    Stage::Recipe
                } else {
                    Stage::Language
                },
                language: String::new(),
                name: String::new(),
                server: None,
                select_only,
                management: None,
            })
        });
        match result {
            Ok(prompt) => {
                self.lsp_setup_prompt(prompt, "", "Choose a language identity; C-g cancels")
            }
            Err(error) => self.error(&error),
        }
    }

    fn lsp_setup_prompt(&mut self, prompt: Prompt, draft: &str, notice: &str) -> Action {
        let (label, names) = prompt.choices();
        if let Err(error) = self.input.names_argument(label, draft, names) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::LspSetup(Box::new(prompt)));
        self.message(notice)
    }

    pub(in crate::session) fn lsp_setup_argument(
        &mut self,
        mut prompt: Prompt,
        draft: &str,
    ) -> Action {
        match prompt.accept(draft) {
            Err(error) => self.lsp_setup_prompt(prompt, draft, &error),
            Ok(false) => self.lsp_setup_prompt(prompt, "", ""),
            Ok(true) => {
                if let Some(choice) = prompt.management.take() {
                    let inherited = prompt
                        .snapshot
                        .settings()
                        .lsp
                        .selected(&prompt.language)
                        .map(|(_, s)| s.clone());
                    return self.launch_installation(
                        choice,
                        prompt.edit,
                        prompt.language,
                        inherited,
                    );
                }
                let state = match crate::config::Roots::from_env() {
                    Ok(roots) => roots.state,
                    Err(error) => return self.error(&error.to_string()),
                };
                match prompt.edit.publish(&state) {
                    Ok(published) => {
                        self.apply_lsp_settings(&published.snapshot);
                        self.message(published.warning.as_deref().unwrap_or(
                            "Server choice saved and applied; lsp-mode controls activation",
                        ))
                    }
                    Err(error) => self.error(&error.to_string()),
                }
            }
        }
    }
}

impl Prompt {
    fn choices(&self) -> (&'static str, Vec<String>) {
        match self.stage {
            Stage::Recipe => (
                "Managed server (rust-analyzer downloads; ready:<id> reuses): ",
                std::iter::once("rust-analyzer".into())
                    .chain(super::managed::choices("rust"))
                    .collect(),
            ),
            Stage::Language => {
                let languages = self
                    .snapshot
                    .languages()
                    .into_iter()
                    .chain(
                        crate::lsp::catalog::RECIPES
                            .iter()
                            .flat_map(|r| r.launches)
                            .flat_map(|p| p.languages)
                            .map(|s| (*s).to_owned()),
                    )
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                ("LSP language: ", languages)
            }
            Stage::Profile => (
                if self.select_only {
                    "Server name (existing or native-only): "
                } else {
                    "Server name (existing, recipe preset, new, or native-only): "
                },
                self.snapshot
                    .settings()
                    .lsp
                    .servers
                    .iter()
                    .filter(|(_, server)| server.languages.contains(&self.language))
                    .map(|(name, _)| name.clone())
                    .chain(
                        crate::lsp::catalog::RECIPES
                            .iter()
                            .flat_map(|r| r.launches)
                            .filter(|p| {
                                !self.select_only && p.languages.contains(&self.language.as_str())
                            })
                            .map(|p| format!("recipe:{}", p.id)),
                    )
                    .chain(super::managed::choices(&self.language))
                    .chain(
                        (!self.select_only && self.language == "rust")
                            .then_some("install:rust-analyzer".into()),
                    )
                    .chain(std::iter::once("native-only".into()))
                    .collect(),
            ),
            Stage::Executable => (
                "Installed executable (literal path or PATH name): ",
                Vec::new(),
            ),
            Stage::Arguments => ("Arguments (JSON array; empty keeps current): ", Vec::new()),
            Stage::Markers => (
                "Root markers (JSON array; empty keeps current): ",
                Vec::new(),
            ),
            Stage::Patterns => ("File patterns (JSON array): ", Vec::new()),
        }
    }

    fn accept(&mut self, draft: &str) -> Result<bool, String> {
        match self.stage {
            Stage::Recipe => self.recipe_choice(draft),
            Stage::Language => self.language(draft),
            Stage::Profile => self.profile(draft),
            Stage::Executable => self.executable(draft),
            Stage::Arguments => self.arguments(draft),
            Stage::Markers => self.markers(draft),
            Stage::Patterns => self.patterns(draft),
        }
    }

    fn language(&mut self, draft: &str) -> Result<bool, String> {
        if !crate::config::language_id(draft) {
            return Err("Enter a lowercase language ID starting with a letter".into());
        }
        self.language = draft.into();
        self.stage = Stage::Profile;
        Ok(false)
    }

    fn executable(&mut self, draft: &str) -> Result<bool, String> {
        if draft.trim().is_empty() {
            return Err("Enter an installed executable".into());
        }
        self.server.as_mut().expect("custom server stage").command = draft.into();
        self.stage = Stage::Arguments;
        Ok(false)
    }

    fn arguments(&mut self, draft: &str) -> Result<bool, String> {
        if !draft.is_empty() {
            self.server.as_mut().expect("custom server stage").args = array(draft)?;
        }
        self.stage = Stage::Markers;
        Ok(false)
    }

    fn markers(&mut self, draft: &str) -> Result<bool, String> {
        if !draft.is_empty() {
            self.server
                .as_mut()
                .expect("custom server stage")
                .root_markers = array(draft)?;
        }
        self.edit.register(
            &self.name,
            self.server.as_ref().expect("custom server stage"),
        );
        self.edit
            .prefer(&self.language, &Preference::Server(self.name.clone()));
        self.associations()
    }

    fn patterns(&mut self, draft: &str) -> Result<bool, String> {
        let patterns = array(draft)?;
        if patterns.is_empty() {
            return Err("Enter at least one basename or *.suffix pattern".into());
        }
        for pattern in patterns {
            self.edit.associate(&pattern, &self.language);
        }
        Ok(true)
    }

    fn profile(&mut self, draft: &str) -> Result<bool, String> {
        if draft == "native-only" {
            self.edit.prefer(&self.language, &Preference::Native(false));
            return self.associations();
        }
        if draft.trim().is_empty() || draft.len() > 128 {
            return Err("Enter a server name of 1..=128 bytes".into());
        }
        if self.snapshot.settings().lsp.servers.contains_key(draft) {
            return self.configured(draft);
        }
        if let Some(choice) = self.management_choice(draft) {
            self.management = Some(choice);
            return self.associations();
        }
        if self.select_only {
            return Err("Choose a compatible configured server or native-only; lsp-setup registers another executable".into());
        }
        if let Some(id) = draft.strip_prefix("recipe:")
            && let Some(launch) = crate::lsp::catalog::launch(id, &self.language)
        {
            return self.recipe(launch);
        }
        self.name = draft.into();
        self.server = Some(self.base());
        self.stage = Stage::Executable;
        Ok(false)
    }

    fn configured(&mut self, name: &str) -> Result<bool, String> {
        if !self.snapshot.settings().lsp.servers[name]
            .languages
            .contains(&self.language)
        {
            return Err("That server does not support this language identity".into());
        }
        self.edit
            .prefer(&self.language, &Preference::Server(name.into()));
        self.associations()
    }

    fn management_choice(&self, draft: &str) -> Option<super::managed::Choice> {
        if let Some(id) = draft.strip_prefix("ready:") {
            return Some(super::managed::Choice::Ready(id.into()));
        }
        (!self.select_only && self.language == "rust" && draft == "install:rust-analyzer")
            .then_some(super::managed::Choice::InstallRust)
    }

    fn recipe_choice(&mut self, draft: &str) -> Result<bool, String> {
        self.language = "rust".into();
        self.management = Some(if draft == "rust-analyzer" {
            super::managed::Choice::InstallRust
        } else if let Some(id) = draft.strip_prefix("ready:") {
            super::managed::Choice::Ready(id.into())
        } else {
            return Err("Choose rust-analyzer or an offered ready installation".into());
        });
        self.associations()
    }

    fn base(&self) -> Server {
        self.snapshot
            .settings()
            .lsp
            .selected(&self.language)
            .map(|(_, server)| {
                let mut server = server.clone();
                server.languages = vec![self.language.clone()];
                server
            })
            .unwrap_or_else(|| Server {
                languages: vec![self.language.clone()],
                command: String::new(),
                args: Vec::new(),
                root_markers: Vec::new(),
                initialization_options: toml::Value::Table(toml::Table::new()),
                settings: toml::Value::Table(toml::Table::new()),
                env: Default::default(),
            })
    }

    fn recipe(&mut self, launch: &crate::lsp::catalog::Launch) -> Result<bool, String> {
        // A configured profile stays authoritative; presets never overwrite it.
        if self.snapshot.settings().lsp.servers.contains_key(launch.id) {
            return self.profile(launch.id);
        }
        let selected = self.snapshot.settings().lsp.selected(&self.language);
        let inherited_path = selected.and_then(|(_, s)| s.env.get("PATH"));
        let process_path = std::env::var_os("PATH");
        let path = inherited_path
            .map(std::ffi::OsStr::new)
            .or(process_path.as_deref());
        if !matches!(
            crate::lsp::catalog::executable(launch.command, path),
            crate::lsp::catalog::Executable::Available(_)
        ) {
            return Err(format!(
                "{} is unavailable or its search path needs a client root; use lsp-servers for prerequisites or register an installed absolute path",
                launch.command
            ));
        }
        let mut server = self.base();
        server.command = launch.command.into();
        server.args = launch.args.iter().map(|s| (*s).to_owned()).collect();
        if selected.is_none() {
            server.root_markers = launch
                .root_markers
                .iter()
                .map(|s| (*s).to_owned())
                .collect();
        }
        self.name = launch.id.into();
        self.server = Some(server);
        self.stage = Stage::Arguments;
        Ok(false)
    }

    fn associations(&mut self) -> Result<bool, String> {
        if !self.snapshot.patterns(&self.language).is_empty() {
            return Ok(true);
        }
        self.stage = Stage::Patterns;
        Ok(false)
    }
}

fn array(draft: &str) -> Result<Vec<String>, String> {
    serde_json::from_str(draft).map_err(|_| "Enter a JSON array of literal strings".into())
}
