use crate::config::{Preference, Server, SettingsEdit, Snapshot};
use crate::session::{Action, Pending, Session};

enum Stage {
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
}

impl Session {
    pub(in crate::session) fn begin_lsp_setup(&mut self) -> Action {
        let result = self.lsp_settings_root().and_then(|root| {
            let edit = SettingsEdit::capture(&root).map_err(|e| e.to_string())?;
            let snapshot = edit.snapshot().map_err(|e| e.to_string())?;
            Ok(Prompt {
                edit,
                snapshot,
                stage: Stage::Language,
                language: String::new(),
                name: String::new(),
                server: None,
            })
        });
        match result {
            Ok(prompt) => self.lsp_setup_prompt(
                prompt,
                "",
                "Choose a language identity; C-g cancels",
            ),
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
            Stage::Language => (
                "LSP language: ",
                self.snapshot
                    .settings()
                    .lsp
                    .servers
                    .values()
                    .flat_map(|server| server.languages.clone())
                    .chain(self.snapshot.settings().language.keys().cloned())
                    .collect(),
            ),
            Stage::Profile => (
                "Server name (existing, new, or native-only): ",
                self.snapshot
                    .settings()
                    .lsp
                    .servers
                    .iter()
                    .filter(|(_, server)| server.languages.contains(&self.language))
                    .map(|(name, _)| name.clone())
                    .chain(std::iter::once("native-only".into()))
                    .collect(),
            ),
            Stage::Executable => (
                "Installed executable (literal path or PATH name): ",
                Vec::new(),
            ),
            Stage::Arguments => (
                "Arguments (JSON array; empty keeps inherited): ",
                Vec::new(),
            ),
            Stage::Markers => (
                "Root markers (JSON array; empty keeps inherited): ",
                Vec::new(),
            ),
            Stage::Patterns => ("File patterns (JSON array): ", Vec::new()),
        }
    }

    fn accept(&mut self, draft: &str) -> Result<bool, String> {
        match self.stage {
            Stage::Language => {
                if !crate::config::language_id(draft) {
                    return Err("Enter a lowercase language ID starting with a letter".into());
                }
                self.language = draft.into();
                self.stage = Stage::Profile;
            }
            Stage::Profile => return self.profile(draft),
            Stage::Executable => {
                if draft.trim().is_empty() {
                    return Err("Enter an installed executable".into());
                }
                self.server.as_mut().expect("custom server stage").command = draft.into();
                self.stage = Stage::Arguments;
            }
            Stage::Arguments => {
                if !draft.is_empty() {
                    self.server.as_mut().expect("custom server stage").args = array(draft)?;
                }
                self.stage = Stage::Markers;
            }
            Stage::Markers => {
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
                return self.associations();
            }
            Stage::Patterns => {
                let patterns = array(draft)?;
                if patterns.is_empty() {
                    return Err("Enter at least one basename or *.suffix pattern".into());
                }
                for pattern in patterns {
                    self.edit.associate(&pattern, &self.language);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn profile(&mut self, draft: &str) -> Result<bool, String> {
        if draft == "native-only" {
            self.edit.prefer(&self.language, &Preference::Native(false));
            return self.associations();
        }
        if draft.trim().is_empty() || draft.len() > 128 {
            return Err("Enter a server name of 1..=128 bytes".into());
        }
        if let Some(server) = self.snapshot.settings().lsp.servers.get(draft) {
            if !server.languages.contains(&self.language) {
                return Err("That server does not support this language identity".into());
            }
            self.edit
                .prefer(&self.language, &Preference::Server(draft.into()));
            return self.associations();
        }
        self.name = draft.into();
        self.server = Some(
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
                }),
        );
        self.stage = Stage::Executable;
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
