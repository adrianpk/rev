use super::*;
use crate::git::{ObjectId, TagKind};

pub(in crate::session) enum Prompt {
    Name {
        refs: Arc<Refs>,
        target: Option<ObjectId>,
    },
    Kind {
        refs: Arc<Refs>,
        target: Option<ObjectId>,
        name: String,
    },
    Target {
        refs: Arc<Refs>,
        kind: TagKind,
        name: String,
    },
}
impl Session {
    pub(in crate::session::git) fn prompt_git_tag(&mut self) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select a current Refs observation before tag creation");
        };
        if let Err(e) = self.git.engine.check_refs(&refs) {
            return self.error(&e);
        }
        let target = self
            .git
            .refs
            .selected
            .as_ref()
            .and_then(|n| refs.reference(n))
            .map(|r| r.object)
            .or(refs.head);
        let default = self
            .git
            .engine
            .tag_intent(refs.observation.context)
            .filter(|i| self.git.engine.tag_draft(i).is_some())
            .map_or("", |i| i.name.as_str());
        if let Err(e) = self.input.argument("New local tag name: ", default) {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::GitTagPrompt(Box::new(Prompt::Name {
            refs,
            target,
        })));
        Action::Redraw
    }
    pub(in crate::session) fn git_tag_argument(&mut self, prompt: Prompt, value: &str) -> Action {
        match prompt {
            Prompt::Name { refs, target } => self.git_tag_name(refs, target, value),
            Prompt::Kind { refs, target, name } => {
                let kind = match value {
                    "" | "lightweight" => TagKind::Lightweight,
                    "annotated" => TagKind::Annotated,
                    _ => {
                        return self.message(
                            "Tag kind must be lightweight or annotated; start creation again",
                        );
                    }
                };
                let default = target.map_or(String::new(), |id| id.hex());
                if let Err(e) = self
                    .input
                    .argument("Full exact tag target object: ", &default)
                {
                    return self.error(&e.to_string());
                }
                self.pending = Some(Pending::GitTagPrompt(Box::new(Prompt::Target {
                    refs,
                    kind,
                    name,
                })));
                Action::Redraw
            }
            Prompt::Target { refs, kind, name } => {
                let target =
                    match ObjectId::parse(value.as_bytes(), refs.observation.identity.format) {
                        Ok(id) => id,
                        Err(e) => return self.error(&e),
                    };
                match self.git.engine.prepare_tag(refs, &name, kind, target) {
                    Ok(id) => {
                        self.git.refs.tag_preparing = Some(id);
                        self.message("Checking local tag name, target, kind and signing policy")
                    }
                    Err(e) => self.error(&e),
                }
            }
        }
    }
    fn git_tag_name(&mut self, refs: Arc<Refs>, target: Option<ObjectId>, value: &str) -> Action {
        if value.is_empty() || value.len() > 4086 || value.contains('\0') {
            return self.message("Tag name requires 1..=4086 non-NUL bytes");
        }
        if let Err(e) = self
            .input
            .argument("Tag kind (lightweight or annotated): ", "lightweight")
        {
            return self.error(&e.to_string());
        }
        self.pending = Some(Pending::GitTagPrompt(Box::new(Prompt::Kind {
            refs,
            target,
            name: value.into(),
        })));
        Action::Redraw
    }
    pub(in crate::session::git) fn publish_git_tag_preparation(&mut self, update: &Update) {
        if self
            .git
            .engine
            .progress()
            .any(|p| p.operation == update.operation)
        {
            return;
        }
        self.git.refs.tag_preparing = None;
        if !self.git.visible
            || self.git.page != Page::Refs
            || self.pending.is_some()
            || self.input.focus() != Focus::Editing
        {
            return;
        }
        let intent = update
            .context
            .filter(|_| update.published)
            .and_then(|id| self.git.engine.tag_intent(id))
            .cloned();
        let Some(intent) = intent else {
            let diagnostic = self
                .git
                .engine
                .record(update.operation)
                .and_then(|r| r.diagnostic.as_deref())
                .unwrap_or("inspect git-processes");
            self.message(&format!(
                "Tag preparation failed: {}",
                label(diagnostic.as_bytes())
            ));
            return;
        };
        if intent.kind == TagKind::Annotated {
            let text = self.git.engine.tag_draft(&intent).unwrap_or("").to_owned();
            match self.install_git_message(
                std::sync::Arc::clone(&intent.refs.observation),
                None,
                Some(intent),
                &text,
            ) {
                Ok(()) => {
                    self.message("Compose tag annotation; Return inserts newline; C-Enter opens exact tag review; C-g retains draft");
                }
                Err(e) => {
                    self.error(&e);
                }
            }
        } else {
            let message = format!(
                "Create local lightweight tag {} at {}? y=accept, n=cancel",
                label(intent.name.as_bytes()),
                intent.target.hex()
            );
            let review = format!(
                "Local tag creation review\nKind: lightweight\nExact target:\n{}\nTarget type: {}\nTag name:\n{}\nLocal ref only. y accepts; n or C-g cancels.",
                intent.target.hex(),
                label(&intent.target_type),
                label(intent.name.as_bytes())
            );
            self.confirm_git_ref(
                Proposal {
                    refs: std::sync::Arc::clone(&intent.refs),
                    action: RefAction::CreateTag(intent),
                    sources: None,
                },
                message,
                review,
            );
        }
    }
    pub(in crate::session::git) fn confirm_git_tag_delete(&mut self) -> Action {
        let Some(refs) =
            self.git.refs().cloned().filter(|_| {
                self.git.visible && self.git.page == Page::Refs && !self.git.refs.failed
            })
        else {
            return self.message("Select a current local tag in Refs first");
        };
        if let Err(e) = self.git.engine.check_refs(&refs) {
            return self.error(&e);
        }
        let Some(reference) = self
            .git
            .refs
            .selected
            .as_ref()
            .and_then(|n| refs.reference(n))
            .filter(|r| r.kind == RefKind::Tag && r.symbolic.is_empty())
        else {
            return self.message("Tag deletion requires one exact nonsymbolic local tag");
        };
        let message = format!(
            "Delete local tag {} at {}? y=accept, n=cancel",
            label(&reference.name),
            reference.object.hex()
        );
        let review = format!(
            "Local tag deletion review\nExact tag/ref object:\n{}\nTag name:\n{}\nPeeled target:\n{}\nKind: {}\nLocal ref only. y accepts; n or C-g cancels.",
            reference.object.hex(),
            label(&reference.name),
            reference
                .peeled
                .map_or("direct object".into(), |id| id.hex()),
            if reference.object_type == b"tag" {
                "annotated tag object"
            } else {
                "lightweight"
            }
        );
        let name = reference.name.clone();
        self.confirm_git_ref(
            Proposal {
                refs,
                action: RefAction::DeleteTag(name),
                sources: None,
            },
            message,
            review,
        )
    }
    pub(in crate::session::git) fn cancel_git_tag_annotation(&mut self) -> Action {
        let result = (|| {
            let text = self.commit_text()?;
            let intent = self
                .git
                .composer
                .as_ref()
                .and_then(|c| c.tag.clone())
                .ok_or("No annotated tag composer")?;
            self.git.engine.retain_tag_draft(&intent, &text)?;
            self.leave_git_commit()
        })();
        match result {
            Ok(()) => {
                self.message("Tag composition canceled; annotation retained for this session")
            }
            Err(e) => self.error(&e),
        }
    }
    pub(in crate::session::git) fn submit_git_tag_annotation(&mut self) -> Action {
        let result = self.submit_git_annotation();
        match result {
            Ok(false)=>self.message("Review annotated tag kind, name, full target and annotation; C-Enter or C-c C-c accepts"),
            Ok(true)=>self.message("Creating checked local annotated tag; annotation retained until verification"),
            Err(e)=>self.error(&e),
        }
    }
    fn submit_git_annotation(&mut self) -> Result<bool, String> {
        let text = self.commit_text()?;
        if text.trim().is_empty() {
            return Err("Annotated tag requires a message".into());
        }
        let state = self
            .git
            .composer
            .as_ref()
            .ok_or("No annotated tag composer")?;
        let intent = state.tag.clone().ok_or("No annotated tag intent")?;
        let text = intent.annotation(&text)?;
        self.git.engine.retain_tag_draft(&intent, &text)?;
        if !state.review {
            let state = self.git.composer.as_mut().expect("tag composer");
            state.review = true;
            state.top = 0;
            self.git.visible = true;
            self.git.page = Page::Refs;
            self.git.painted = None;
            self.input.cancel();
            self.sync_input_context();
            return Ok(false);
        }
        let id = self.git.engine.create_tag(intent, &text)?;
        self.git.refs.request = Some(id);
        self.leave_git_commit()?;
        Ok(true)
    }
    pub(in crate::session::git) fn git_tag_review_frame(
        &mut self,
    ) -> Result<Frame, crate::view::Error> {
        let state = self.git.composer.as_ref().expect("tag review");
        let intent = state.tag.as_ref().expect("tag intent");
        let buffer = self.buffer()?;
        let text = buffer
            .slice(&buffer.span(0..buffer.len_chars())?)?
            .to_string();
        let text = intent
            .annotation(&text)
            .expect("validated exact tag review");
        let metadata = format!(
            "Local tag review\nKind: annotated\nExact target:\n{}\nTarget type: {}\nTag name:\n{}\nGit signing policy: {}\nAnnotation: {} bytes; final newline: {}\nAccepted annotation:",
            intent.target.hex(),
            label(&intent.target_type),
            label(intent.name.as_bytes()),
            if intent.signing {
                "signing required"
            } else {
                "retained"
            },
            text.len(),
            text.ends_with('\n')
        );
        let width = usize::from(self.size.width.max(1));
        let mut rows = metadata
            .lines()
            .chain(text.split('\n'))
            .flat_map(|line| Wrapped {
                text: label(line.as_bytes()),
                offset: Some(0),
                width,
            })
            .skip(state.top);
        let admission = self
            .git
            .engine
            .presentation(self.size.width, self.size.height)?;
        let mut frame = Frame::pending(self.buffer()?, self.size, self.buffers.palette())?.git(
            &self.echo,
            " Git Tag Review  C-Enter / C-c C-c accept  q / C-g retain  Up/Down scroll",
            admission,
            |_| (rows.next().unwrap_or_default(), false),
        )?;
        self.prompt_frame(&mut frame, None)?;
        Ok(frame)
    }
}
