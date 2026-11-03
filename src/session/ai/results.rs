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
        let actions = if edits {
            "Proposal only: ai-apply-result accepts; ai-discard-result discards."
        } else {
            "Explanation only: no apply action; ai-discard-result returns to source."
        };
        let condition = self
            .buffers
            .get(target.source.stamp.buffer)
            .map_err(|e| e.to_string())
            .and_then(|b| target.current(b))
            .err()
            .unwrap_or_default();
        let mut text = format!(
            "Pi {}\nContext: {}; fresh ephemeral history.\nModel: {}\nSource: buffer {} revision {}, scalars {}..{} (captured snapshot).\n{}\n{}\n\n",
            target.output.name(),
            if output.directory {
                "guarded directory read/list/search; captured native snapshots override disk"
            } else {
                "input only; directory discovery disabled"
            },
            output.model,
            target.source.stamp.buffer,
            target.source.stamp.revision,
            target.source.range.start,
            target.source.range.end,
            actions,
            condition,
        );
        if output.directory {
            text.push_str(&format!("Directory root: {}\n\n", request.root.display()));
        }
        let start = text.chars().count();
        text.push_str(&output.text);
        let result = self.buffers.result(label, &text).and_then(|id| {
            self.buffers
                .get(id)?
                .span(start..self.buffers.get(id)?.len_chars())
        });
        let payload = match result {
            Ok(payload) => payload,
            Err(error) => return self.error(&format!("Pi result: {error}")),
        };
        let show = self.windows.selected_id() == request.view
            && self.view().point.stamp == target.source.stamp
            && self.input.focus() == Focus::Editing
            && self.pending.is_none()
            && !self.tree.focused()
            && !self.windows.retained_view_active();
        let id = payload.stamp.buffer;
        self.ai.ready = Some(Ready { request, payload });
        if show {
            self.activate(id);
        }
        if edits {
            self.message("AI proposal ready in *AI Result*; ai-apply-result or ai-discard-result")
        } else {
            self.message("Pi explanation ready in *AI Explanation* (captured selection; read-only)")
        }
    }

    pub(in crate::session) fn ai_apply_result(&mut self) -> Action {
        match self.ai_publish() {
            Ok(home) => {
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
        let text = result
            .slice(&ready.payload)
            .map_err(|e| e.to_string())?
            .to_string();
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
            )
            .map_err(|e| e.to_string())?;
        buffer.deactivate_region();
        Ok(home)
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
            && let Some(home) = self.ai_home(&ready.request)
        {
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

    fn proposal() -> (Session, BufferId) {
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.create("source").unwrap();
        let buffer = buffers.get(id).unwrap();
        let mut view = View::new(buffer);
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
            target,
            return_to,
            root: std::env::temp_dir(),
            view: session.windows.selected_id(),
            home: session.view().clone(),
        };
        session.ai_present(
            request,
            pi::Output {
                text: "generated".into(),
                model: "controlled".into(),
                directory: true,
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
