use super::*;
use crate::{
    buffer::{Span, edit::Replacement},
    org::links::{Edit, Target},
};
pub(in crate::session) struct Prompt {
    request: Request,
    span: Span,
    target: String,
    description: String,
    describing: bool,
}
impl Session {
    pub(super) fn begin_link_edit(
        &mut self,
        mut edit: Edit,
        request: Request,
    ) -> Result<(), String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let mut span = buffer
            .byte_span(edit.bytes.clone())
            .map_err(|e| e.to_string())?;
        if edit.bytes.is_empty() {
            if let Some(region) = buffer
                .region(request.origin.point)
                .map_err(|e| e.to_string())?
            {
                edit.description = buffer
                    .slice(&region)
                    .map_err(|e| e.to_string())?
                    .to_string();
                span = region;
            }
            if let Some(stored) = self.links.stored.front() {
                edit.target = stored.target.clone();
                if edit.description.is_empty() {
                    edit.description = stored.description.clone();
                }
            }
        }
        let prompt = Prompt {
            request,
            span,
            target: edit.target,
            description: edit.description,
            describing: false,
        };
        self.prompt_link(prompt)
    }
    fn prompt_link(&mut self, prompt: Prompt) -> Result<(), String> {
        let (label, draft) = if prompt.describing {
            ("Link description: ", &prompt.description)
        } else {
            ("Org link target: ", &prompt.target)
        };
        self.input
            .argument(label, draft)
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Link(Box::new(super::Prompt::Insert(prompt))));
        self.message("Enter literal link data; TAB shows stored links; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn link_argument(&mut self, mut prompt: Prompt, draft: &str) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.error("Link source or view changed; source unchanged");
        }
        if !prompt.describing {
            if let Err(error) = Target::parse(draft) {
                return self.error(&error);
            }
            prompt.target = draft.to_owned();
            prompt.describing = true;
            return match self.prompt_link(prompt) {
                Ok(()) => Action::Redraw,
                Err(error) => self.error(&error),
            };
        }
        prompt.description = draft.to_owned();
        let result = link_text(&prompt.target, &prompt.description).and_then(|text| {
            self.publish_link_edit(Replacement {
                source: prompt.span,
                text,
            })
        });
        match result {
            Ok(()) => {
                self.links.stored.retain(|l| l.target != prompt.target);
                self.message("Org link inserted; source undo available")
            }
            Err(error) => self.error(&error),
        }
    }
    pub(in crate::session) fn complete_link(&mut self) -> Action {
        if matches!(&self.pending,Some(Pending::Link(p)) if matches!(p.as_ref(),super::Prompt::Viewer(_)))
        {
            return self.message("Enter yes to launch the configured viewer, or no/C-g to cancel");
        }
        let labels = self
            .links
            .stored
            .iter()
            .take(8)
            .map(|s| s.target.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        self.message(&format!(
            "Stored links: {labels}; literal file:, id:, attachment:, #custom-id or target"
        ))
    }
}
fn link_text(target: &str, description: &str) -> Result<String, String> {
    Target::parse(target)?;
    if description.len() > 4096 || description.chars().any(char::is_control) {
        return Err("Link description requires at most 4096 bytes without controls".into());
    }
    let target = escaped(target);
    if description.contains("]]") {
        return Err("Link description contains a closing delimiter".into());
    }
    Ok(if description.is_empty() {
        format!("[[{target}]]")
    } else {
        format!("[[{target}][{description}]]")
    })
}

fn escaped(value: &str) -> String {
    let mut chars = value.chars().peekable();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            out.push('\\');
            if chars.peek().is_some_and(|c| "[]\\".contains(*c)) {
                out.push(chars.next().expect("escaped character"));
            } else {
                out.push('\\');
            }
        } else {
            if "[]".contains(c) {
                out.push('\\');
            }
            out.push(c);
        }
    }
    out
}

impl Session {
    fn publish_link_edit(&mut self, edit: Replacement) -> Result<(), String> {
        let cursor =
            crate::coordinates::Scalar(edit.source.range.start + edit.text.chars().count());
        let buffer = self
            .buffers
            .get_mut(edit.source.stamp.buffer)
            .map_err(|e| e.to_string())?;
        let points = self.windows.points(buffer).map_err(|e| e.to_string())?;
        let selected = crate::buffer::edit::Cursor {
            index: points.initiator,
            position: cursor,
        };
        let mut prepared = buffer
            .prepare(&[edit], &points.values, Some(selected))
            .map_err(|e| e.to_string())?;
        prepared.set_origin(self.windows.selected_id());
        self.windows
            .publish(buffer, prepared, points)
            .map_err(|e| e.to_string())?;
        buffer.deactivate_region();
        self.windows.selected_mut().reset_navigation();
        Ok(())
    }
}
