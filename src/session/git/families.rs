use super::*;
use crate::git::{Family, FamilyOutput, FamilyPlan, OperationKind, Update};
use std::ffi::OsString;
use std::path::Path;

pub(in crate::session::git) struct Draft {
    pub(in crate::session::git) was_visible: bool,
    family: Family,
    directory: PathBuf,
    context: Option<ContextId>,
    plan: Option<Arc<FamilyPlan>>,
}
impl Session {
    fn git_family_sources(
        &self,
        family: Family,
        directory: &Path,
        target: Option<&Path>,
    ) -> Result<(), String> {
        if (!family.repository_action() && !family.direct())
            || family == Family::SubmoduleList
            || family.output()
        {
            return Ok(());
        }
        for buffer in self.buffers.iter() {
            let related = buffer.file().is_some_and(|f| {
                f.path().starts_with(directory)
                    || std::fs::canonicalize(f.path()).is_ok_and(|p| p.starts_with(directory))
            });
            if (related || (family.direct() && buffer.file().is_some())) && buffer.is_modified() {
                return Err(if family.direct() {
                    "Direct command refused: a visiting file buffer has unsaved text".into()
                } else {
                    "Prepared repository action refused: a repository-backed buffer has unsaved text".into()
                });
            }
            if family == Family::SubmoduleDeinit
                && target.is_some_and(|root| {
                    buffer.file().is_some_and(|f| {
                        f.path().starts_with(root)
                            || std::fs::canonicalize(f.path()).is_ok_and(|p| p.starts_with(root))
                    })
                })
            {
                return Err("Submodule deinit is protected by a retained visiting buffer".into());
            }
        }
        if family == Family::SubmoduleDeinit
            && target.is_some_and(|root| {
                self.tree.chosen_root().is_some_and(|p| p.starts_with(root))
                    || self.terminals.retains_directory(root)
                    || self.shell.retains_directory(root)
                    || self.ai.retains_directory(root)
                    || self.lsp.retains_directory(root)
            })
        {
            return Err(
                "Submodule deinit is protected by a retained native directory owner".into(),
            );
        }
        Ok(())
    }
    pub(in crate::session::git) fn compose_git_family(&mut self, family: Family) -> Action {
        if self.git.engine.busy() {
            return self.message("Prepared composition waits for owned Git work");
        }
        let mut observation = self
            .git
            .state
            .context
            .filter(|_| self.git.visible)
            .and_then(|id| self.git.engine.context(id))
            .map(|c| std::sync::Arc::clone(&c.observation));
        let mut directory = observation
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
        if family.direct() && observation.is_none() {
            let file_parent = self
                .buffer()
                .ok()
                .and_then(|b| b.file())
                .and_then(|f| f.path().parent());
            directory = if self.tree.focused() {
                self.tree.chosen_root().or(file_parent)
            } else {
                file_parent.or_else(|| self.tree.chosen_root())
            }
            .unwrap_or(&self.git.launch)
            .to_owned();
            let canonical = std::fs::canonicalize(&directory).unwrap_or_else(|_| directory.clone());
            observation = self
                .git
                .engine
                .contexts()
                .iter()
                .filter(|c| c.failure.is_none())
                .filter(|c| {
                    canonical.starts_with(
                        c.observation
                            .identity
                            .worktree
                            .as_ref()
                            .unwrap_or(&c.observation.identity.git_directory),
                    )
                })
                .max_by_key(|c| {
                    c.observation
                        .identity
                        .worktree
                        .as_ref()
                        .unwrap_or(&c.observation.identity.git_directory)
                        .components()
                        .count()
                })
                .map(|c| std::sync::Arc::clone(&c.observation));
            if let Some(o) = &observation {
                directory = o
                    .identity
                    .worktree
                    .as_ref()
                    .unwrap_or(&o.identity.git_directory)
                    .clone();
            }
        }
        let context = observation.as_ref().map(|o| o.context);
        let text = if family.output() {
            serde_json::to_string_pretty(
                &serde_json::json!({"arguments": family.seed(), "output": ""}),
            )
            .expect("output draft")
        } else {
            serde_json::to_string_pretty(&family.seed()).expect("string argument array")
        };
        let draft = Draft {
            was_visible: self.git.visible,
            family,
            directory,
            context,
            plan: None,
        };
        match self.install_git_text(observation, None, None, Some(draft), &text) {
            Ok(()) => self.message(if family.output() { "Edit arguments and explicit output path; C-Enter or C-c C-c prepares exact review; C-g cancels" } else { "Edit the JSON argument array; fill empty parameters; C-Enter or C-c C-c prepares exact review; C-g cancels" }),
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
                self.git_family_sources(draft.family, &draft.directory, Some(&plan.destination))?;
                let id = self
                    .git
                    .engine
                    .execute_family(std::sync::Arc::clone(plan))?;
                self.git.family_request = Some(id);
                self.leave_git_commit()?;
                return Ok(true);
            }
            self.git_family_sources(draft.family, &draft.directory, None)?;
            let text = self.commit_text()?;
            if text.len() > crate::git::MAX_COMMIT_BYTES {
                return Err("Prepared draft exceeds 64 KiB".into());
            }
            let (args, output): (Vec<String>, Option<PathBuf>) = if draft.family.output() {
                #[derive(serde::Deserialize)]
                #[serde(deny_unknown_fields)]
                struct OutputDraft {
                    arguments: Vec<String>,
                    output: PathBuf,
                }
                let value: OutputDraft = serde_json::from_str(&text).map_err(|e| {
                    format!(
                        "Output input requires an arguments array and explicit output path: {e}"
                    )
                })?;
                (value.arguments, Some(value.output))
            } else {
                (
                    serde_json::from_str(&text).map_err(|e| {
                        format!("Prepared input requires one JSON array of literal arguments: {e}")
                    })?,
                    None,
                )
            };
            if args.len() > 64 || args.iter().any(|a| a.len() > 4096 || a.contains('\0')) {
                return Err(
                    "Prepared input exceeds its argument count/byte bounds or contains NUL".into(),
                );
            }
            let arguments = args.into_iter().map(OsString::from).collect();
            let id = if let Some(output) = output {
                self.git.engine.prepare_family_output(
                    &draft.directory,
                    draft.context,
                    FamilyOutput {
                        family: draft.family,
                        args: arguments,
                        output,
                    },
                )?
            } else {
                self.git.engine.prepare_family(
                    &draft.directory,
                    draft.context,
                    draft.family,
                    arguments,
                )?
            };
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
            let outcome = self.git.engine.record(update.operation).map(|r| r.outcome);
            let direct = self
                .git
                .engine
                .family_result()
                .is_some_and(|r| r.plan.family.direct());
            let message = if direct {
                "Direct command ended; Git postconditions remain unknown; $ inspects actual argv, exit and output".to_owned()
            } else if outcome == Some(crate::git::Outcome::InProgress) {
                "Prepared Git command stopped in progress; git-operation-status inspects the actual sequence".to_owned()
            } else if update.published && outcome == Some(crate::git::Outcome::Verified) {
                if let Some(output) = self
                    .git
                    .engine
                    .family_result()
                    .filter(|r| r.plan.family.output())
                {
                    format!(
                        "Prepared Git output saved and verified: {}; $ inspects the producer; find-file visits the explicit output",
                        label(output.plan.destination.as_os_str().as_encoded_bytes())
                    )
                } else if let Some(result) = self
                    .git
                    .engine
                    .family_result()
                    .filter(|r| r.created.is_none())
                {
                    result.object.map_or_else(
                        || "Prepared Git repository command verified; $ inspects the accepted operation and output".to_owned(),
                        |object| format!("Prepared Git repository command verified; object {}; $ inspects the accepted operation and output", object.hex()),
                    )
                } else {
                    "Prepared Git destination created and verified; $ inspects the accepted operation".to_owned()
                }
            } else {
                "Prepared Git command failed or effects remain uncertain; inspect destination and $ diagnostics".to_owned()
            };
            self.message(&message);
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
        let mode = if plan.family == Family::RunShellCommand {
            "explicit shell string; configured shell arguments"
        } else {
            "literal Git argv; no shell evaluation"
        };
        let mut metadata = format!(
            "Prepared Git review: {}\nMode: {mode}\nProgram: {}\nDirectory: {}\nExplicit destination: {}\nAccepted argv (one argument per line):\n",
            plan.family.label(),
            label(plan.program.as_bytes()),
            label(plan.directory.as_os_str().as_encoded_bytes()),
            label(plan.destination.as_os_str().as_encoded_bytes())
        );
        if plan.family != Family::RunShellCommand {
            metadata.push_str("--no-optional-locks\n");
        }
        if let Some((seconds, bytes)) = plan.direct_limits() {
            metadata.push_str(&format!("Limits: {seconds}s; {bytes} aggregate output bytes\nGit postconditions: unknown; stdin: closed\n"));
        }
        if let Some(object) = plan.target_object() {
            metadata.push_str(&format!("Pinned Git object: {}\n", object.hex()));
        }
        if let Some((path, bytes, hash)) = plan.input_preview() {
            metadata.push_str(&format!(
                "Input file: {}\nComplete sealed input bytes: {bytes}\nInput Git blob hash: {}\n",
                label(path.as_os_str().as_encoded_bytes()),
                hash.hex()
            ));
        }
        if let Some((bytes, hash)) = plan.output_preview() {
            metadata.push_str(&format!("Complete output bytes: {bytes}\nOutput Git blob hash: {} (no object database write)\n", hash.hex()));
        }
        for object in plan.prune_objects() {
            metadata.push_str(&format!(
                "Prune note for missing object: {}\n",
                object.hex()
            ));
        }
        if let Some(reference) = plan.remote_branch() {
            metadata.push_str(&format!(
                "Exact remote branch: {}\n",
                label(reference.as_encoded_bytes())
            ));
        }
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
        let mut frame = Frame::pending(buffer, self.size, self.buffers.palette())?.git(
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
