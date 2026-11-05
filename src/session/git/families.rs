use super::*;
use crate::git::{Family, FamilyPlan, OperationKind, Update};
use std::ffi::OsString;

pub(in crate::session::git) struct Draft {
    pub(in crate::session::git) was_visible: bool,
    family: Family,
    directory: PathBuf,
    context: Option<ContextId>,
    plan: Option<Arc<FamilyPlan>>,
}
impl Session {
    pub(in crate::session::git) fn compose_git_family(&mut self, family: Family) -> Action {
        if self.git.engine.busy() {
            return self.message("Prepared composition waits for owned Git work");
        }
        let observation = self
            .git
            .state
            .context
            .filter(|_| self.git.visible)
            .and_then(|id| self.git.engine.context(id))
            .map(|c| c.observation.clone());
        let directory = observation
            .as_ref()
            .map(|o| {
                o.identity
                    .worktree
                    .as_ref()
                    .unwrap_or(&o.identity.git_directory)
                    .clone()
            })
            .or_else(|| self.tree.chosen_root().map(PathBuf::from))
            .or_else(|| {
                self.buffer()
                    .ok()?
                    .file()?
                    .path()
                    .parent()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| self.git.launch.clone());
        let context = observation.as_ref().map(|o| o.context);
        let text = serde_json::to_string_pretty(&family.seed()).expect("string argument array");
        let draft = Draft {
            was_visible: self.git.visible,
            family,
            directory,
            context,
            plan: None,
        };
        match self.install_git_text(observation, None, None, Some(draft), &text) {
            Ok(()) => self.message("Edit the JSON argument array; fill empty parameters; C-Enter or C-c C-c prepares exact review; C-g cancels"),
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn cancel_git_family(&mut self) -> Action {
        match self.leave_git_commit() {
            Ok(()) => self.message("Prepared command canceled before execution; source retained"),
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn submit_git_family(&mut self) -> Action {
        let result: Result<bool, String> = (|| {
            if self.git.family_preparing.is_some() {
                return Err("Prepared arguments are still checking".into());
            }
            let draft = self
                .git
                .composer
                .as_ref()
                .and_then(|c| c.family.as_ref())
                .ok_or("No prepared argument composer")?;
            if let Some(plan) = &draft.plan {
                let id = self.git.engine.execute_family(plan.clone())?;
                self.git.family_request = Some(id);
                self.leave_git_commit()?;
                return Ok(true);
            }
            let text = self.commit_text()?;
            if text.len() > crate::git::MAX_COMMIT_BYTES {
                return Err("Prepared draft exceeds 64 KiB".into());
            }
            let args: Vec<String> = serde_json::from_str(&text).map_err(|e| {
                format!("Prepared input requires one JSON array of literal arguments: {e}")
            })?;
            if args.len() > 64 || args.iter().any(|a| a.len() > 4096 || a.contains('\0')) {
                return Err(
                    "Prepared input exceeds its argument count/byte bounds or contains NUL".into(),
                );
            }
            let id = self.git.engine.prepare_family(
                &draft.directory,
                draft.context,
                draft.family,
                args.into_iter().map(OsString::from).collect(),
            )?;
            self.git.family_preparing = Some(id);
            Ok(false)
        })();
        match result {
            Ok(true) => self.message("Executing accepted Git arguments; owned work continues on q; $ inspects actual results"),
            Ok(false) => self.message("Checking prepared arguments and explicit destination; no changing child authorized"),
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn publish_git_family(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        if update.kind == OperationKind::FamilyPrepare {
            self.git.family_preparing = None;
            if !update.published {
                self.message("Prepared arguments rejected; draft retained; git-processes inspects diagnostics");
                return;
            }
            let Some(plan) = self.git.engine.family_plan().cloned() else {
                return;
            };
            let Some(composer) = self.git.composer.as_mut() else {
                return;
            };
            let Some(draft) = composer.family.as_mut() else {
                return;
            };
            if draft.family != plan.family
                || self.pending.is_some()
                || self.input.focus() != Focus::Editing
            {
                return;
            }
            draft.plan = Some(plan);
            composer.review = true;
            composer.top = 0;
            self.git.visible = true;
            self.git.painted = None;
            self.input.cancel();
            self.sync_input_context();
            self.message(
                "Exact prepared Git review; C-Enter or C-c C-c executes; q or C-g cancels",
            );
            return;
        }
        self.git.family_request = None;
        if update.published {
            self.git.inspection = comparisons::Inspection::default();
            self.git.reconcile(self.git_extent());
            self.observe_files();
        }
        if self.git.visible && self.pending.is_none() && self.input.focus() == Focus::Editing {
            self.message(if update.published { "Prepared Git destination created and verified; $ inspects the accepted operation" }
                else { "Prepared Git command failed or effects remain uncertain; inspect destination and $ diagnostics" });
        }
    }
    pub(in crate::session::git) fn git_family_review_frame(
        &mut self,
    ) -> Result<Frame, crate::view::Error> {
        let composer = self.git.composer.as_ref().expect("prepared review");
        let plan = composer
            .family
            .as_ref()
            .and_then(|d| d.plan.as_ref())
            .expect("prepared plan");
        let mut metadata = format!(
            "Prepared Git review: {}\nMode: literal Git argv; no shell evaluation\nProgram: {}\nDirectory: {}\nExplicit destination: {}\nAccepted argv (one argument per line):\n--no-optional-locks\n",
            plan.family.label(),
            label(plan.program.as_bytes()),
            label(plan.directory.as_os_str().as_encoded_bytes()),
            label(plan.destination.as_os_str().as_encoded_bytes())
        );
        for (index, arg) in plan.args.iter().enumerate() {
            metadata.push_str(&format!("{index}: {}\n", label(arg.as_encoded_bytes())));
        }
        metadata.push_str("C-Enter / C-c C-c executes this captured input. q / C-g cancels. No retry, implicit save or destination deletion.");
        let width = usize::from(self.size.width.max(1));
        let mut lines = metadata
            .lines()
            .flat_map(move |s| Wrapped {
                text: s.into(),
                offset: Some(0),
                width,
            })
            .skip(composer.top);
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let buffer = self.buffer().map_err(|_| crate::view::Error::GitCapacity)?;
        let mut frame = Frame::git(
            buffer,
            self.size,
            self.buffers.palette(),
            &self.echo,
            " Git Argument Review C-Enter / C-c C-c execute q / C-g cancel",
            admission,
            |_| (lines.next().unwrap_or_default(), false),
        )?;
        frame.message_kind(self.echo_kind);
        Ok(frame)
    }
    pub(in crate::session::git) fn visit_git_ignore(&mut self, exclude: bool) -> Action {
        let Some(observation) = self.git.observation() else {
            return self.message("Open checked Git Status before visiting an ignore file");
        };
        let Some(root) = observation.identity.worktree.as_ref() else {
            return self.message("Ignore-file visits require a working tree");
        };
        let path = if exclude {
            observation.identity.common_directory.join("info/exclude")
        } else {
            root.join(".gitignore")
        };
        match self.load_file(&path) {
            Ok(()) => {
                self.git.visible = false;
                self.sync_input_context();
                self.message("Visited the ordinary ignore file; no pattern or save was inserted")
            }
            Err(e) => self.error(&e.to_string()),
        }
    }
}
