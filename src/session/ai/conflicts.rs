//! Copied Git context enters the existing optional AI owner as explanation
//! data. Its qualified native point retains return ownership, never edit rights.
use super::*;
use crate::buffer::{Buffer, Buffers};
use crate::{
    config::{AiCommand, AiContext, AiInput, AiOutput},
    git::{Completeness, ConflictExplanation, ConflictRole},
};
use serde_json::{Value, json};
use std::{io::Write, path::Path, sync::Arc};

pub(super) struct Input {
    pub(super) capture: Arc<ConflictExplanation>,
    pub(super) text: String,
    source: Option<Span>,
    _admission: crate::git::budget::Lease,
}
impl Input {
    pub(super) fn current(&self, buffers: &crate::buffer::Buffers) -> Result<(), String> {
        if let Some(source) = &self.source {
            buffers
                .get(source.stamp.buffer)
                .map_err(|_| "Captured visiting conflict buffer was closed")?
                .slice(source)
                .map_err(
                    |_| "Visiting conflict source changed; explanation is a historical snapshot",
                )?;
        }
        self.capture.current()
    }
    pub(super) fn path(&self) -> String {
        crate::display::git::label(
            self.capture
                .conflict
                .change
                .path
                .as_os_str()
                .as_encoded_bytes(),
        )
    }
    pub(super) fn complete(&self) -> bool {
        self.capture.conflict.completeness == Completeness::Complete
            && self.capture.saved.is_ok()
            && self.capture.conflict.sides.iter().flatten().all(|s| s.text)
    }
}

// Serialize directly into the admitted capacity. Reached bounds reject the
// whole request rather than truncating source or allocating an unbounded tree.
struct ContextBytes {
    bytes: Vec<u8>,
    cap: usize,
}
impl Write for ContextBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.cap.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "Conflict explanation context exceeds configured complete input admission",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Session {
    pub(in crate::session) fn ai_explain_conflict(
        &mut self,
        capture: Arc<ConflictExplanation>,
    ) -> Result<Action, String> {
        use super::capture::Capture;
        self.ai_reap_idle()?;
        let definition = AiCommand {
            prompt: String::new(),
            input: AiInput::Point,
            context: AiContext::Input,
            output: AiOutput::Explanation,
            model: None,
        };
        let mut request = self.ai_capture(Capture::Saved(definition))?;
        let conflict = &capture.conflict;
        let root = conflict
            .sequence
            .refs
            .observation
            .identity
            .worktree
            .as_ref()
            .ok_or("Conflict explanation requires a captured worktree")?
            .clone();
        let target = root.join(&conflict.change.path);
        let input = self.ai_conflict_input(capture, &target)?;
        request.source_name = format!("Git conflict: {}", input.path());
        request.language = "plain".into();
        request.root = root;
        request.conflict = Some(input);
        self.ai_start(request, "Explain this captured Git conflict, including exact operation-relative Base/Current/Applied roles and available saved or unsaved source. Present manual resolution choices without editing, staging, continuing or executing commands.")?;
        Ok(self.message("Pi conflict explanation starting with copied input only; manual Git actions remain available"))
    }

    fn ai_conflict_input(
        &self,
        capture: Arc<ConflictExplanation>,
        target: &Path,
    ) -> Result<Input, String> {
        let visiting = visiting(&self.buffers, target);
        let cap = self
            .buffers
            .snapshot()
            .settings()
            .git
            .max_detail_bytes
            .min(self.buffers.snapshot().settings().ai.pi.max_input_bytes);
        let native_bytes = visiting.map_or(0, |b| {
            b.slice(&b.span(0..b.len_chars()).expect("whole native buffer span"))
                .expect("whole native buffer slice")
                .len_bytes()
        });
        if native_bytes > cap {
            return Err(
                "Visiting conflict source exceeds complete explanation input admission".into(),
            );
        }
        let source_bytes = captured_bytes(&capture) + native_bytes;
        let admission = self
            .git
            .engine
            .source_admission(cap + source_bytes.saturating_mul(12) + 131072)?;
        let source = visiting
            .map(|b| b.span(0..b.len_chars()).map_err(|e| e.to_string()))
            .transpose()?;
        let native = native(visiting, source.as_ref())?;
        let body = body(&capture, &native)?;
        let text = encode(&body, cap)?;
        Ok(Input {
            capture,
            text,
            source,
            _admission: admission,
        })
    }
}

fn visiting<'a>(buffers: &'a Buffers, target: &Path) -> Option<&'a Buffer> {
    let canonical = std::fs::canonicalize(target).ok();
    buffers.iter().find(|b| {
        b.file().is_some_and(|f| {
            f.path() == target
                || canonical.as_ref().is_some_and(|target| {
                    std::fs::canonicalize(f.path()).is_ok_and(|p| p == *target)
                })
        })
    })
}

fn captured_bytes(capture: &ConflictExplanation) -> usize {
    capture
        .conflict
        .sides
        .iter()
        .flatten()
        .map(|s| s.bytes.len())
        .sum::<usize>()
        + capture
            .saved
            .as_ref()
            .ok()
            .and_then(|s| s.as_ref())
            .map_or(0, |s| s.bytes.len())
}

fn native(visiting: Option<&Buffer>, source: Option<&Span>) -> Result<Value, String> {
    let Some(b) = visiting else {
        return Ok(Value::Null);
    };
    Ok(
        json!({"buffer_id":b.id().to_string(), "revision":b.stamp().revision, "dirty":b.is_modified(),
        "text":b.slice(source.expect("captured visiting span")).map_err(|e| e.to_string())?.to_string()}),
    )
}

fn stage(capture: &ConflictExplanation, role: ConflictRole) -> Value {
    let conflict = &capture.conflict;
    match conflict.side(role) {
        None => {
            json!({"role":role.label(), "stage":role.stage(), "meaning":role.meaning(conflict.sequence.family), "state":"absent"})
        }
        Some(s) => {
            json!({"role":role.label(), "stage":role.stage(), "meaning":role.meaning(conflict.sequence.family),
            "object":s.object.hex(), "mode":format!("{:06o}",s.mode), "binary":s.binary, "complete":s.completeness == Completeness::Complete,
            "state":if s.text {"complete text"} else {"unsupported or partial content"}, "captured_bytes":s.bytes.len(),
            "text":if s.text {std::str::from_utf8(&s.bytes).ok()} else {None}})
        }
    }
}

fn saved(capture: &ConflictExplanation) -> Result<Value, String> {
    Ok(match &capture.saved {
        Ok(Some(s)) => {
            json!({"state":"saved disk text", "mode":format!("{:06o}",s.mode), "text":std::str::from_utf8(&s.bytes).map_err(|_| "Captured result is not text")?})
        }
        Ok(None) => json!({"state":"absent; deletion is not staged by explanation"}),
        Err(e) => json!({"state":"unavailable", "reason":e}),
    })
}

fn body(capture: &ConflictExplanation, native: &Value) -> Result<Value, String> {
    let conflict = &capture.conflict;
    let stages = ConflictRole::ALL
        .into_iter()
        .map(|role| stage(capture, role))
        .collect::<Vec<_>>();
    let saved = saved(capture)?;
    Ok(
        json!({"version":1, "path":crate::display::git::label(conflict.change.path.as_os_str().as_encoded_bytes()),
        "kind":conflict.kind(), "operation":format!("{:?}",conflict.sequence.family), "head":conflict.sequence.refs.head.map(|h| h.hex()),
        "stages":stages, "saved_result":saved, "visiting_result":native,
        "authority":"Historical source data only. Explain roles and possible manual resolution choices. Do not edit, stage, continue or execute commands."}),
    )
}

fn encode(body: &Value, cap: usize) -> Result<String, String> {
    let mut bytes = ContextBytes {
        bytes: Vec::with_capacity(cap),
        cap,
    };
    serde_json::to_writer(&mut bytes, body)
        .map_err(|e| format!("Conflict explanation capture: {e}"))?;
    String::from_utf8(bytes.bytes).map_err(|_| "Conflict explanation encoding failed".into())
}
