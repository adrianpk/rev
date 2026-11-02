use crate::config::{Roots, SettingsEdit};
use crate::lsp::install::{self, Installation, Registry, State, Update};
use crate::session::{Action, Pending, Session};

pub(in crate::session) struct Prompt {
    edit: SettingsEdit,
    data: std::path::PathBuf,
    installations: Vec<Installation>,
    remove: bool,
    work: Option<Work>,
}

enum Work {
    Update(Update),
    Remove(Installation),
}

impl Session {
    pub(super) fn begin_lifecycle(&mut self, remove: bool) -> Action {
        if self.installation.active() {
            return self.error("another server management job is still owned");
        }
        match self.lifecycle_capture(remove) {
            Ok(prompt) => {
                self.lifecycle_prompt(prompt, "Choose a managed installation ID; C-g cancels")
            }
            Err(error) => self.error(&error),
        }
    }

    fn lifecycle_capture(&self, remove: bool) -> Result<Prompt, String> {
        let edit = SettingsEdit::capture(&self.lsp_settings_root()?).map_err(|e| e.to_string())?;
        let roots = Roots::from_env().map_err(|e| e.to_string())?;
        let registry = Registry::load(&roots.data)?;
        let installations: Vec<_> = registry
            .installations
            .into_iter()
            .filter(|i| remove || i.status == State::Ready)
            .collect();
        if installations.is_empty() {
            return Err("no eligible managed installation; inspect lsp-servers".into());
        }
        let data = roots.data.canonicalize().map_err(|e| e.to_string())?;
        Ok(Prompt {
            edit,
            data,
            installations,
            remove,
            work: None,
        })
    }

    fn lifecycle_prompt(&mut self, prompt: Prompt, notice: &str) -> Action {
        let (label, choices) = if prompt.work.is_some() {
            (
                "Confirm managed operation (yes/no): ",
                vec!["yes".into(), "no".into()],
            )
        } else {
            (
                "Managed installation ID: ",
                prompt.installations.iter().map(|i| i.id.clone()).collect(),
            )
        };
        if let Err(error) = self.input.names_argument(label, "", choices) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::LspLifecycle(Box::new(prompt)));
        self.message(notice)
    }

    pub(in crate::session) fn lifecycle_argument(
        &mut self,
        mut prompt: Prompt,
        draft: &str,
    ) -> Action {
        if prompt.work.is_some() {
            return match draft {
                "yes" => self.launch_lifecycle(prompt),
                "no" => self.message("Server management cancelled"),
                _ => self.lifecycle_prompt(prompt, "Enter yes to confirm or no to cancel"),
            };
        }
        match prompt.choose(draft) {
            Ok(notice) => self.lifecycle_prompt(prompt, &notice),
            Err(error) => self.lifecycle_prompt(prompt, &error),
        }
    }

    fn launch_lifecycle(&mut self, prompt: Prompt) -> Action {
        let roots = match Roots::from_env() {
            Ok(roots) => roots,
            Err(e) => return self.error(&e.to_string()),
        };
        let source = match self.buffer() {
            Ok(buffer) => buffer.stamp(),
            Err(e) => return self.error(&e.to_string()),
        };
        let result =
            self.installation
                .start(source, roots, move |mut storage, cancel| {
                    match prompt.work.expect("confirmed lifecycle work") {
                        Work::Update(update) => update.run(&mut storage, prompt.edit, cancel),
                        Work::Remove(installation) => {
                            install::remove(&mut storage, prompt.edit, &installation, cancel)
                        }
                    }
                });
        match result {
            Ok(()) => {
                self.installation_notice = "Checking requested lifecycle operation".into();
                self.message("Server management started; editing remains available")
            }
            Err(error) => self.error(&error),
        }
    }

    pub(super) fn cancel_installation(&mut self) -> Action {
        if self.installation.cancel() {
            self.message("Server management cancellation requested; worker remains owned until cleanup finishes")
        } else {
            self.message("No server management job is active")
        }
    }
}

impl Prompt {
    fn choose(&mut self, id: &str) -> Result<String, String> {
        let installation = self
            .installations
            .iter()
            .find(|i| i.id == id)
            .ok_or("choose an offered installation ID")?;
        let notice = if self.remove {
            let names = install::references(
                installation,
                &self.edit.snapshot().map_err(|e| e.to_string())?,
                &self.data,
            );
            if !names.is_empty() {
                return Err(format!(
                    "installation is referenced by profiles: {}; change those profiles before removal",
                    names.join(", ")
                ));
            }
            format!(
                "Remove {} {} ({}); external servers stay unchanged. Confirm yes/no",
                installation.recipe, installation.version, installation.id
            )
        } else {
            let update = Update::capture(&self.edit, installation, &self.data)?;
            let notice = format!(
                "Update {} {}; selected profiles: {}. Prior payload retained. Confirm yes/no",
                installation.recipe,
                installation.version,
                update.profiles().join(", ")
            );
            self.work = Some(Work::Update(update));
            return Ok(notice);
        };
        self.work = Some(Work::Remove(installation.clone()));
        Ok(notice)
    }
}
