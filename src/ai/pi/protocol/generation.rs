//! Correlated generation events and bounded assistant/tool observations.
use super::{Control, Tools, Value, allowed, frame, response};
use serde::Deserialize;
use std::sync::atomic::Ordering;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generated {
    version: u8,
    text: String,
}

#[derive(Default)]
pub(super) struct State {
    pub accepted: bool,
    pub settled: bool,
    final_text: Option<String>,
    turns: usize,
    streamed: usize,
    tools: Tools,
}

const LIFECYCLE: [&str; 14] = [
    "agent_start",
    "agent_end",
    "turn_end",
    "message_start",
    "message_end",
    "auto_retry_start",
    "auto_retry_end",
    "auto_compaction_start",
    "auto_compaction_end",
    "compaction_start",
    "compaction_end",
    "summarization_retry_scheduled",
    "summarization_retry_attempt_start",
    "summarization_retry_finished",
];

impl State {
    pub fn finish(self, limit: usize) -> Result<String, String> {
        if !self.tools.active.is_empty() {
            return Err("Pi settled with unfinished tools".into());
        }
        let raw = self
            .final_text
            .ok_or("Pi settled without a completed assistant result")?;
        let generated: Generated = serde_json::from_str(&raw)
            .map_err(|_| "Pi final result must contain only version=1 and text")?;
        if generated.version != 1 || generated.text.len() > limit || generated.text.contains('\0') {
            return Err("Pi generated result has an invalid version, NUL or excessive text".into());
        }
        Ok(generated.text)
    }

    pub fn event(
        &mut self,
        record: &Value,
        id: &str,
        directory: bool,
        control: &Control,
    ) -> Result<(), String> {
        match record["type"].as_str() {
            Some("response") => self.accept(record, id, control),
            Some("agent_settled") => self.settle(record),
            Some("turn_start") => self.turn(),
            Some(kind) if ["message_start", "message_update", "message_end"].contains(&kind) => {
                self.message(record, directory)
            }
            Some(kind)
                if [
                    "tool_execution_start",
                    "tool_execution_update",
                    "tool_execution_end",
                ]
                .contains(&kind) =>
            {
                self.tool(record, directory)
            }
            _ => lifecycle_event(record, directory),
        }
    }

    fn accept(&mut self, record: &Value, id: &str, control: &Control) -> Result<(), String> {
        if self.accepted {
            return Err("Pi emitted a duplicate command response".into());
        }
        response(record, id, "prompt")?;
        if record["data"]["disposition"] != "started" {
            return Err("Pi did not start the explicit prompt".into());
        }
        self.accepted = true;
        control.accepted.store(true, Ordering::Release);
        Ok(())
    }

    fn settle(&mut self, record: &Value) -> Result<(), String> {
        if self.settled || record["aborted"] != false {
            return Err("Pi generation was aborted or settled twice".into());
        }
        self.settled = true;
        Ok(())
    }

    fn turn(&mut self) -> Result<(), String> {
        self.turns += 1;
        if self.turns > 8 {
            return Err("Pi exceeded eight model turns".into());
        }
        self.clear_message();
        Ok(())
    }

    fn clear_message(&mut self) {
        self.final_text = None;
        self.streamed = 0;
    }

    fn message(&mut self, record: &Value, directory: bool) -> Result<(), String> {
        match record["type"].as_str() {
            Some("message_start") if record["message"]["role"] == "assistant" => {
                self.clear_message();
                Ok(())
            }
            Some("message_update") => self.update(record),
            Some("message_end") if record["message"]["role"] == "assistant" => {
                self.complete(&record["message"], directory)
            }
            _ => Ok(()),
        }
    }

    fn update(&mut self, record: &Value) -> Result<(), String> {
        if let Some(delta) = record["assistantMessageEvent"]["delta"].as_str() {
            self.streamed = self.streamed.saturating_add(delta.len());
            if self.streamed > frame::RECORD {
                return Err("Pi streamed assistant content exceeds 2 MiB".into());
            }
        }
        Ok(())
    }

    fn complete(&mut self, message: &Value, directory: bool) -> Result<(), String> {
        if message["stopReason"] == "toolUse" && directory {
            tool_turn(message)?;
            self.final_text = None;
            return Ok(());
        }
        if message["stopReason"] != "stop" {
            return Err(
                "Pi provider did not complete; check Pi authentication/model and retry".into(),
            );
        }
        self.final_text = Some(assistant_text(message)?);
        Ok(())
    }

    fn tool(&mut self, record: &Value, directory: bool) -> Result<(), String> {
        if !directory {
            return Err("Pi attempted a tool under an input-only grant".into());
        }
        self.tools.event(record)
    }
}

fn lifecycle_event(record: &Value, directory: bool) -> Result<(), String> {
    match record["type"].as_str() {
        Some("extension_ui_request") => {
            Err("Pi requested unavailable UI; authenticate in Pi and retry".into())
        }
        Some("extension_error") if directory => Err(
            "Pi guarded discovery failed; retry with an admitted root and within invocation limits"
                .into(),
        ),
        Some(kind) if LIFECYCLE.contains(&kind) => Ok(()),
        _ => Err("Pi emitted an unsupported RPC event".into()),
    }
}

fn tool_turn(message: &Value) -> Result<(), String> {
    let blocks = message["content"]
        .as_array()
        .ok_or("Invalid Pi tool turn")?;
    if blocks.iter().any(|b| {
        !matches!(b["type"].as_str(), Some("text" | "thinking"))
            && (b["type"] != "toolCall" || !allowed(&b["name"]))
    }) {
        return Err("Pi proposed an unadmitted tool".into());
    }
    Ok(())
}

fn assistant_text(message: &Value) -> Result<String, String> {
    let content = message["content"]
        .as_array()
        .ok_or("Pi assistant content is invalid")?;
    let mut text = String::new();
    for block in content {
        match block["type"].as_str() {
            Some("text") => text.push_str(
                block["text"]
                    .as_str()
                    .ok_or("Pi assistant text is invalid")?,
            ),
            Some("thinking") => {}
            _ => return Err("Pi input-only result contains an unadmitted content block".into()),
        }
    }
    Ok(text)
}
