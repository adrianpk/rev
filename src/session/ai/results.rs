use super::{Ready, Request};
use crate::{
    ai::{pi, target::Output},
    buffer::edit::Replacement,
    input::Focus,
    session::{Action, Session},
    view::View,
};

impl Session {
    pub(super) fn ai_present(&mut self, request: Request, output: pi::Output) -> Action {
        let target = &request.target;
        let edits = target.output.edits();
        let label = if edits {
            "*AI Result*"
        } else {
            "*AI Explanation*"
        };
        let condition = self
            .buffers
            .get(target.source.stamp.buffer)
            .map_err(|e| e.to_string())
            .and_then(|b| target.current(b))
            .err()
            .or_else(|| {
                request
                    .conflict
                    .as_ref()
                    .and_then(|c| c.current(&self.buffers).err())
            })
            .unwrap_or_default();
        let document = super::presentation::document(&request, &output, &condition);
        let result = self
            .buffers
            .org_result(label, &document.text)
            .and_then(|id| self.buffers.get(id)?.span(document.body));
        let payload = match result {
            Ok(payload) => payload,
            Err(error) => return self.error(&format!("Pi result: {error}")),
        };
        let show = self.windows.selected_id() == request.view
            && self.view().point.stamp == target.source.stamp
            && self.input.focus() == Focus::Editing
            && self.pending.is_none()
            && !self.tree.focused()
            && !self.windows.retained_view_active()
            && request
                .conflict
                .as_ref()
                .is_none_or(|c| self.git_explanation_origin(&c.capture));
        let id = payload.stamp.buffer;
        let conflict = request.conflict.is_some();
        self.ai.last = Some(request.definition.clone());
        self.ai.ready = Some(Ready {
            request,
            payload,
            generated: output.text,
            drawer: document.drawer,
            drawer_body: document.drawer_body,
        });
        if show {
            if conflict {
                self.release_git_explanation();
            }
            self.activate(id);
        }
        if edits {
            self.message("AI proposal ready in *AI Result*; ai-apply-result or ai-discard-result")
        } else {
            self.message(
                "Pi explanation ready in *AI Explanation* (captured native input; read-only)",
            )
        }
    }

    pub(in crate::session) fn ai_apply_result(&mut self) -> Action {
        match self.ai_publish() {
            Ok(mut home) => {
                home.retain_heading_presentation(self.view());
                // Publication already succeeded. Consume authority before any
                // presentation/cleanup step so an error cannot enable a replay.
                let ready = self.ai.ready.take().expect("published AI result");
                *self.windows.selected_mut() = home;
                self.invalidate_completion();
                self.published_repeat_boundary();
                self.activate_lsp();
                self.observe_files();
                self.ai_close_result(
                    ready,
                    "AI result applied; one undo step; source remains unsaved",
                )
            }
            Err(error) => self.error(&error),
        }
    }

    fn ai_publish(&mut self) -> Result<View, String> {
        if self.input.focus() != Focus::Editing
            || self.pending.is_some()
            || self.tree.focused()
            || self.windows.retained_view_active()
        {
            return Err("Editing must own AI result acceptance".into());
        }
        let ready = self.ai.ready.as_ref().ok_or("No AI result is ready")?;
        if ready.request.target.output == Output::Explanation {
            return Err("Explanation results have no apply action".into());
        }
        if self.view().point.stamp.buffer != ready.payload.stamp.buffer {
            return Err("Select the AI result before applying its captured target".into());
        }
        let result = self
            .buffers
            .get(ready.payload.stamp.buffer)
            .map_err(|e| e.to_string())?;
        if !result.is_readonly() {
            return Err("AI result is no longer read-only; source unchanged".into());
        }
        // The display revision is still authoritative, but its escaped Org
        // representation is never interpreted as generated source text.
        result.slice(&ready.payload).map_err(|e| e.to_string())?;
        let text = ready.generated.clone();
        if text.len() > self.buffers.snapshot().settings().ai.pi.max_result_bytes {
            return Err("AI result exceeds max_result_bytes".into());
        }
        let target = &ready.request.target;
        let buffer = self
            .buffers
            .get_mut(target.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        target.current(buffer)?;
        let home = self
            .windows
            .publish_observed(
                buffer,
                Replacement {
                    source: target.source.clone(),
                    text,
                },
                ready.request.home.clone(),
                self.windows.selected_id(),
            )
            .map_err(|e| e.to_string())?;
        buffer.deactivate_region();
        Ok(home)
    }

    pub(in crate::session) fn ai_result_buffer(&self, id: crate::buffer::BufferId) -> bool {
        self.ai
            .ready
            .as_ref()
            .is_some_and(|ready| ready.payload.stamp.buffer == id)
    }

    pub(in crate::session) fn ai_result_visibility(&mut self) {
        let Some(ready) = &self.ai.ready else { return };
        let Ok(buffer) = self.buffers.get(ready.payload.stamp.buffer) else {
            return;
        };
        if !buffer.is_org() || buffer.stamp() != ready.payload.stamp {
            return;
        }
        let views: Vec<_> = self
            .windows
            .iter()
            .filter(|(_, view)| view.point.stamp == buffer.stamp())
            .map(|(id, _)| id)
            .collect();
        for id in views {
            if let Ok(view) = self.windows.get_mut(id) {
                let pending = self
                    .org
                    .request
                    .as_mut()
                    .filter(|request| request.view == id && request.origin == *view);
                let _ = view.initial_org_drawer(
                    buffer,
                    buffer.org_outline().cloned(),
                    ready.drawer,
                    ready.drawer_body.clone(),
                );
                // Initial folding changes derived projection only. Preserve a
                // queued command whose complete origin still matched beforehand.
                if let Some(pending) = pending {
                    pending.origin = view.clone();
                }
            }
        }
    }

    pub(in crate::session) fn ai_discard_result(&mut self) -> Action {
        if self.input.focus() != Focus::Editing || self.pending.is_some() || self.tree.focused() {
            return self.error("Editing must own AI result discard");
        }
        let Some(ready) = self.ai.ready.take() else {
            if self.cancel_ai() {
                return self
                    .message("AI generation discarded; source unchanged; stopping owned child");
            }
            return self.message("No AI result is ready");
        };
        if self.view().point.stamp.buffer == ready.payload.stamp.buffer
            && let Some(mut home) = self.ai_home(&ready.request)
        {
            home.retain_heading_presentation(self.view());
            *self.windows.selected_mut() = home;
        }
        self.ai_close_result(ready, "AI result discarded; source unchanged")
    }

    fn ai_home(&self, request: &Request) -> Option<View> {
        let buffer = self.buffers.get(request.target.source.stamp.buffer).ok()?;
        if request.home.point.stamp == buffer.stamp() {
            return Some(request.home.clone());
        }
        let mut view = View::new(buffer);
        let anchor = request.return_to.source();
        if !anchor.closed && anchor.position.is_some() && buffer.stamp() == anchor.point.stamp {
            view.point = anchor.point;
        }
        Some(view)
    }

    fn ai_close_result(&mut self, ready: Ready, message: &str) -> Action {
        let id = ready.payload.stamp.buffer;
        if let Ok(source) = self.buffers.get(ready.request.target.source.stamp.buffer) {
            self.windows.replace_buffer(id, source);
        }
        match self.buffers.prepare_kill(id) {
            Ok(kill) => {
                self.close_buffer(kill);
                if self.buffers.get(id).is_ok() {
                    let detail = self.echo().to_owned();
                    return self.error(&format!("{message}; result cleanup failed: {detail}"));
                }
            }
            Err(_) => {
                self.input.cancel();
                self.sync_input_context();
            }
        }
        self.message(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ai::target::{Kind, Target},
        buffer::{BufferId, Buffers},
        coordinates::Scalar,
        view::Size,
    };

    mod fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/configuration/fixture.rs"
        ));
    }

    // Real readonly Org results admit presentation controls in both profiles.
    // Apply/discard restore source anchors but retain the current view choice;
    // escaped display bytes never substitute for the original generated text.
    #[test]
    fn heading_presentation() {
        use crate::command::{Command, OrgCommand};
        let generated = "* Literal\r\n#+end_src\n** Child\n";
        for profile in ["non-modal", "modal"] {
            for apply in [false, true] {
                let fixture = fixture::Fixture::new();
                let settings = crate::config::prepare(&fixture.write(
                    "config.toml",
                    &format!("editor.interaction_profile='{profile}'"),
                ))
                .snapshot;
                let (mut session, source) = generated_with_settings(generated, settings);
                let result = session.view().point.stamp.buffer;
                let buffer = session.buffers.get_mut(result).unwrap();
                assert!(buffer.is_org() && buffer.is_readonly());
                let scan =
                    crate::org::ColorScan::new(buffer, crate::org::Limits::default()).unwrap();
                buffer
                    .install_org_coloring(scan.run(&crate::org::Cancellation::default()))
                    .unwrap();
                session.ai_result_visibility();
                let original = text_of(session.buffer().unwrap());
                let point = session.view().point;
                session.execute(Command::Org(OrgCommand::PrettyHeadings));
                assert!(!session.view().pretty_headings());
                session.execute(Command::Org(OrgCommand::ResetHeadingPresentation));
                assert!(session.view().pretty_headings());
                session.execute(Command::Org(OrgCommand::PrettyHeadings));
                assert_eq!(session.view().point, point);
                assert_eq!(text_of(session.buffer().unwrap()), original);
                assert!(session.buffer().unwrap().history().is_empty());
                if apply {
                    session.execute(Command::AiApplyResult);
                    assert_eq!(text(&session, source), format!("sou{generated}rce"));
                    assert_eq!(session.buffer().unwrap().history().len(), 1);
                    session.execute(Command::History(crate::buffer::edit::History::Undo));
                } else {
                    session.execute(Command::AiDiscardResult);
                    assert!(session.buffer().unwrap().history().is_empty());
                }
                assert_eq!(text(&session, source), "source");
                assert_eq!(
                    session.view().heading_presentation_override(),
                    Some(false),
                    "{profile}, apply={apply}"
                );
                assert!(session.ai.ready.is_none());
            }
        }
    }

    fn proposal() -> (Session, BufferId) {
        generated_proposal("generated")
    }

    // General startup cannot override deliberately folded REQUEST presentation,
    // even with an overview preference. Later explicit full reveal stays final.
    #[test]
    fn startup_visibility() {
        use crate::command::{Command, OrgCommand};
        for policy in ["overview", "content", "showall", "showeverything"] {
            let fixture = fixture::Fixture::new();
            let settings = crate::config::prepare(
                &fixture.write("config.toml", &format!("org.startup_folded='{policy}'")),
            )
            .snapshot;
            let (mut session, _) = generated_with_settings("generated", settings);
            let buffer = session
                .buffers
                .get_mut(session.view().point.stamp.buffer)
                .unwrap();
            let scan = crate::org::ColorScan::new(buffer, crate::org::Limits::default()).unwrap();
            buffer
                .install_org_coloring(scan.run(&crate::org::Cancellation::default()))
                .unwrap();
            session.refresh_org();
            let buffer = session.buffer().unwrap();
            let folded = session.view().visible_lines(buffer);
            assert!(folded < buffer.len_lines(), "{policy}");
            assert!(
                session
                    .frame()
                    .unwrap()
                    .rows()
                    .iter()
                    .any(|r| r.text().contains("generated")),
                "{policy}"
            );
            session.execute(Command::Org(OrgCommand::ShowAll));
            session.refresh_org();
            let buffer = session.buffer().unwrap();
            assert_eq!(
                session.view().visible_lines(buffer),
                buffer.len_lines(),
                "{policy}"
            );
            assert!(buffer.history().is_empty());
        }
    }

    fn generated_proposal(generated: &str) -> (Session, BufferId) {
        generated_with_settings(generated, crate::config::Snapshot::default())
    }

    fn generated_with_settings(
        generated: &str,
        settings: crate::config::Snapshot,
    ) -> (Session, BufferId) {
        let mut buffers = Buffers::with_settings(settings.clone()).unwrap();
        let id = buffers.create("source").unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::with_settings(buffer, &settings);
        view.point.position = Scalar(3);
        let mut session = Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap();
        let source = session.buffer().unwrap();
        let target = Target::capture(source, session.view().point, Kind::Implement).unwrap();
        let return_to = session
            .buffers
            .locations
            .lock()
            .unwrap()
            .capture(source, session.view().point)
            .unwrap();
        let request = Request {
            conflict: None,
            source_name: "report.rs".into(),
            language: "rust".into(),
            definition: crate::config::AiCommand {
                prompt: "fixture".into(),
                input: crate::config::AiInput::Point,
                context: crate::config::AiContext::Directory,
                output: Output::Insert,
                model: None,
            },
            target,
            return_to,
            root: std::env::temp_dir(),
            view: session.windows.selected_id(),
            home: session.view().clone(),
        };
        session.ai_present(
            request,
            pi::Output {
                text: generated.into(),
                model: "controlled".into(),
                directory: true,
                history: "controlled".into(),
            },
        );
        (session, id)
    }
    fn text(session: &Session, id: BufferId) -> String {
        let b = session.buffers.get(id).unwrap();
        b.slice(&b.span(0..b.len_chars()).unwrap())
            .unwrap()
            .to_string()
    }

    // Display escaping and line-ending normalization must never alter accepted
    // code. Native Org must still recognize the trailing request drawer.
    #[test]
    fn literal_proposals() {
        for generated in [
            "fn café() {}\r\n",
            "* Heading\n#+end_src\n:REQUEST:\n:END:\n",
            "  ,#+end_example\n** child\nlast line",
            "\u{301}👩‍💻\r\nno final newline",
        ] {
            let (mut session, source) = generated_proposal(generated);
            let result = session.view().point.stamp.buffer;
            let buffer = session.buffers.get_mut(result).unwrap();
            assert!(buffer.is_org());
            assert!(
                text_of(buffer)
                    .starts_with("* report.rs — insertion proposal\n\n#+begin_src rust\n")
            );
            let scan = crate::org::ColorScan::new(buffer, crate::org::Limits::default()).unwrap();
            buffer
                .install_org_coloring(scan.run(&crate::org::Cancellation::default()))
                .unwrap();
            session.ai_result_visibility();
            let buffer = session.buffer().unwrap();
            assert!(session.view().visible_lines(buffer) < buffer.len_lines());
            session.ai_apply_result();
            assert_eq!(text(&session, source), format!("sou{generated}rce"));
            assert_eq!(session.buffer().unwrap().history().len(), 1);
            session.execute(crate::command::Command::History(
                crate::buffer::edit::History::Undo,
            ));
            assert_eq!(text(&session, source), "source");
        }
    }

    fn text_of(buffer: &crate::buffer::Buffer) -> String {
        buffer
            .slice(&buffer.span(0..buffer.len_chars()).unwrap())
            .unwrap()
            .to_string()
    }

    // TAB can arrive before analysis completes. Initial projection must retain
    // that exact pending view and toggle the initially closed drawer just once.
    #[test]
    fn pending_drawer() {
        let (mut session, _) = proposal();
        session.ai_result_visibility();
        let point = session.ai.ready.as_ref().unwrap().drawer;
        session.windows.selected_mut().point.position = point;
        session.org.request = Some(crate::session::outline::Request {
            view: session.windows.selected_id(),
            origin: session.view().clone(),
            command: crate::command::OrgCommand::Cycle,
        });
        let buffer = session
            .buffers
            .get_mut(session.view().point.stamp.buffer)
            .unwrap();
        let scan = crate::org::ColorScan::new(buffer, crate::org::Limits::default()).unwrap();
        buffer
            .install_org_coloring(scan.run(&crate::org::Cancellation::default()))
            .unwrap();
        session.ai_result_visibility();
        session.finish_org_request();
        assert!(session.org.request.is_none());
        assert_eq!(
            session.view().visible_lines(session.buffer().unwrap()),
            session.buffer().unwrap().len_lines()
        );
    }

    // A target becoming read-only after generation must fail live admission;
    // the same still-current proposal remains available after permission returns.
    #[test]
    fn readonly_at_application() {
        let (mut session, source) = proposal();
        session.buffers.get_mut(source).unwrap().set_readonly(true);
        session.ai_apply_result();
        assert!(session.echo().contains("read-only"));
        assert_eq!(text(&session, source), "source");
        assert!(session.ai.ready.is_some());
        session.buffers.get_mut(source).unwrap().set_readonly(false);
        session.ai_apply_result();
        assert_eq!(text(&session, source), "sougeneratedrce");
        assert!(session.ai.ready.is_none());
    }

    // Even an internal caller altering a result through native editing cannot
    // reuse its old payload span as settled model authority for source publication.
    #[test]
    fn result_revision_guard() {
        let (mut session, source) = proposal();
        let id = session.view().point.stamp.buffer;
        let buffer = session.buffers.get_mut(id).unwrap();
        buffer.set_readonly(false);
        let edit = Replacement {
            source: buffer.span(0..0).unwrap(),
            text: "changed".into(),
        };
        let prepared = buffer.prepare(&[edit], &[], None).unwrap();
        buffer.publish(prepared, &mut []).unwrap();
        buffer.set_readonly(true);
        session.ai_apply_result();
        assert_eq!(text(&session, source), "source");
        assert!(session.ai.ready.is_some());
    }
}
