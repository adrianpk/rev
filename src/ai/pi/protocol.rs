use std::os::unix::net::UnixStream;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::{Value, json};

use super::{
    Control, Output, frame,
    io::{self, Inbox},
};
use crate::config::Pi;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Generated {
    version: u8,
    text: String,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    inbox: &Inbox,
    input: &mut UnixStream,
    control: &Control,
    policy: &Pi,
    message: &str,
    epoch: u64,
    deadline: Instant,
    admission: Option<&Value>,
) -> Result<Output, String> {
    if let Some(admission) = admission {
        admit(inbox, input, control, epoch, deadline, admission)?;
    }
    let state_id = format!("{epoch}:state");
    let prompt_id = format!("{epoch}:prompt");
    io::send(input, &json!({"id":state_id,"type":"get_state"}))?;
    let model = loop {
        let Some(record) = io::next(inbox, control, deadline)? else {
            continue;
        };
        if record["type"] == "extension_ui_request" {
            return Err("Pi extension UI is unavailable; authenticate in Pi and retry".into());
        }
        if record["type"] != "response" {
            return Err("Pi emitted unexpected startup activity".into());
        }
        response(&record, &state_id, "get_state")?;
        let state = &record["data"];
        if state["isCompacting"] != false
            || state["isStreaming"] != false
            || state["messageCount"] != 0
            || state["pendingMessageCount"] != 0
            || state["sessionFile"].as_str().is_some()
        {
            return Err("Pi did not admit a fresh session".into());
        }
        let provider = state["model"]["provider"]
            .as_str()
            .ok_or("Pi has no model; configure a model and authenticate in Pi, then retry")?;
        let id = state["model"]["id"]
            .as_str()
            .ok_or("Pi model identity is unavailable")?;
        if provider.len() + id.len() > 256 {
            return Err("Pi model identity exceeds 256 bytes".into());
        }
        break format!("{provider}/{id}");
    };
    io::send(
        input,
        &json!({"id":prompt_id,"type":"prompt","message":message}),
    )?;
    let mut accepted = false;
    let mut settled = false;
    let mut final_text: Option<String> = None;
    let mut turns = 0;
    let mut streamed = 0usize;
    let mut tools = Tools::default();
    loop {
        if accepted && settled {
            if !tools.active.is_empty() {
                return Err("Pi settled with unfinished tools".into());
            }
            let raw = final_text.ok_or("Pi settled without a completed assistant result")?;
            let generated: Generated = serde_json::from_str(&raw)
                .map_err(|_| "Pi final result must contain only version=1 and text")?;
            if generated.version != 1
                || generated.text.len() > policy.max_result_bytes
                || generated.text.contains('\0')
            {
                return Err(
                    "Pi generated result has an invalid version, NUL or excessive text".into(),
                );
            }
            return Ok(Output {
                text: generated.text,
                model,
                directory: admission.is_some(),
            });
        }
        let Some(record) = io::next(inbox, control, deadline)? else {
            continue;
        };
        match record["type"].as_str() {
            Some("response") => {
                if accepted {
                    return Err("Pi emitted a duplicate command response".into());
                }
                response(&record, &prompt_id, "prompt")?;
                if record["data"]["disposition"] != "started" {
                    return Err("Pi did not start the explicit prompt".into());
                }
                accepted = true;
                control.accepted.store(true, Ordering::Release);
            }
            Some("agent_settled") => {
                if settled || record["aborted"] != false {
                    return Err("Pi generation was aborted or settled twice".into());
                }
                settled = true;
            }
            Some("turn_start") => {
                turns += 1;
                if turns > 8 {
                    return Err("Pi exceeded eight model turns".into());
                }
                final_text = None;
                streamed = 0;
            }
            Some("message_start") if record["message"]["role"] == "assistant" => {
                final_text = None;
                streamed = 0;
            }
            Some("message_update") => {
                if let Some(delta) = record["assistantMessageEvent"]["delta"].as_str() {
                    streamed = streamed.saturating_add(delta.len());
                    if streamed > frame::RECORD {
                        return Err("Pi streamed assistant content exceeds 2 MiB".into());
                    }
                }
            }
            Some("message_end") if record["message"]["role"] == "assistant" => {
                let message = &record["message"];
                if message["stopReason"] == "toolUse" && admission.is_some() {
                    let blocks = message["content"]
                        .as_array()
                        .ok_or("Invalid Pi tool turn")?;
                    if blocks.iter().any(|b| {
                        !matches!(b["type"].as_str(), Some("text" | "thinking"))
                            && (b["type"] != "toolCall" || !allowed(&b["name"]))
                    }) {
                        return Err("Pi proposed an unadmitted tool".into());
                    }
                    final_text = None;
                    continue;
                }
                if message["stopReason"] != "stop" {
                    return Err(
                        "Pi provider did not complete; check Pi authentication/model and retry"
                            .into(),
                    );
                }
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
                        _ => {
                            return Err(
                                "Pi input-only result contains an unadmitted content block".into(),
                            );
                        }
                    }
                }
                final_text = Some(text);
            }
            Some("tool_execution_start" | "tool_execution_update" | "tool_execution_end") => {
                if admission.is_none() {
                    return Err("Pi attempted a tool under an input-only grant".into());
                }
                tools.event(&record)?;
            }
            Some("extension_ui_request") => {
                return Err("Pi requested unavailable UI; authenticate in Pi and retry".into());
            }
            Some("extension_error") if admission.is_some() => {
                return Err("Pi guarded discovery failed; retry with an admitted root and within invocation limits".into());
            }
            Some(
                "agent_start"
                | "agent_end"
                | "turn_end"
                | "message_start"
                | "message_end"
                | "auto_retry_start"
                | "auto_retry_end"
                | "auto_compaction_start"
                | "auto_compaction_end"
                | "compaction_start"
                | "compaction_end"
                | "summarization_retry_scheduled"
                | "summarization_retry_attempt_start"
                | "summarization_retry_finished",
            ) => {}
            _ => return Err("Pi emitted an unsupported RPC event".into()),
        }
    }
}

fn response(record: &Value, id: &str, command: &str) -> Result<(), String> {
    if record["id"] != id || record["command"] != command {
        return Err("Pi response correlation failed".into());
    }
    if record["success"] != true {
        return Err("Pi rejected the RPC command; check Pi authentication/model and retry".into());
    }
    Ok(())
}

// Cancellation uses only declared RPC commands in the documented order. Its
// finite deadline is independent of generation; late records never publish.
pub(super) fn cancel(inbox: &Inbox, input: &mut UnixStream, epoch: u64) {
    let clear = format!("{epoch}:clear");
    let abort = format!("{epoch}:abort");
    if io::send(input, &json!({"id":clear,"type":"clear_queue"})).is_err() {
        return;
    }
    if io::send(input, &json!({"id":abort,"type":"abort"})).is_err() {
        return;
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut cleared = false;
    let mut aborted = false;
    while Instant::now() < deadline {
        let bytes = match inbox.next(Duration::from_millis(20)) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(_) => return,
        };
        let Ok(record) = serde_json::from_slice::<Value>(&bytes) else {
            return;
        };
        if record["type"] == "agent_settled" {
            return;
        }
        if record["type"] == "response" {
            if response(&record, &clear, "clear_queue").is_ok() {
                cleared = true;
            }
            if response(&record, &abort, "abort").is_ok() {
                aborted = true;
            }
            if cleared && aborted {
                return;
            }
        }
    }
}

const NAMES: [&str; 3] = ["rev_read", "rev_list", "rev_search"];
fn allowed(name: &Value) -> bool {
    name.as_str().is_some_and(|name| NAMES.contains(&name))
}

fn admit(
    inbox: &Inbox,
    input: &mut UnixStream,
    control: &Control,
    epoch: u64,
    deadline: Instant,
    grant: &Value,
) -> Result<(), String> {
    let id = format!("{epoch}:admit");
    let message = format!("/rev-discovery {grant}");
    io::send(input, &json!({"id":id,"type":"prompt","message":message}))?;
    let deadline = deadline.min(Instant::now() + Duration::from_secs(5));
    let (mut acknowledged, mut ready) = (false, false);
    while !acknowledged || !ready {
        let Some(record) = io::next(inbox, control, deadline)? else {
            continue;
        };
        match record["type"].as_str() {
            Some("response") if !acknowledged => {
                response(&record, &id, "prompt")?;
                if record["data"]["disposition"] != "handled" {
                    return Err("Pi did not handle discovery admission".into());
                }
                acknowledged = true;
            }
            Some("extension_ui_request") if !ready && record["method"] == "notify" => {
                let data: Value = serde_json::from_str(
                    record["message"]
                        .as_str()
                        .ok_or("Pi discovery admission missing")?,
                )
                .map_err(|_| "Pi discovery admission invalid")?;
                let mut names = data["tools"].as_array().cloned().unwrap_or_default();
                names.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
                if data["type"] != "rev-discovery-v1"
                    || data["nonce"] != epoch
                    || data["admitted"] != true
                    || names != vec![json!("rev_list"), json!("rev_read"), json!("rev_search")]
                {
                    return Err("Pi rejected the guarded discovery profile".into());
                }
                ready = true;
            }
            _ => return Err("Pi emitted unexpected discovery startup activity".into()),
        }
    }
    Ok(())
}

#[derive(Default)]
struct Tools {
    active: std::collections::BTreeMap<String, String>,
    seen: std::collections::BTreeSet<String>,
    returned: usize,
}
impl Tools {
    fn event(&mut self, record: &Value) -> Result<(), String> {
        if !allowed(&record["toolName"]) {
            return Err("Pi executed an unadmitted tool".into());
        }
        let id = record["toolCallId"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() <= 256)
            .ok_or("Pi tool identity invalid")?;
        let name = record["toolName"].as_str().ok_or("Pi tool name invalid")?;
        match record["type"].as_str() {
            Some("tool_execution_start") => {
                if !self.seen.insert(id.into()) || self.seen.len() > 32 {
                    return Err(
                        "Pi exceeded 32 discovery tool calls or repeated an identity".into(),
                    );
                }
                self.active.insert(id.into(), name.into());
            }
            Some("tool_execution_end") => {
                if self.active.remove(id).as_deref() != Some(name) {
                    return Err("Pi tool completion correlation failed".into());
                }
                let result = &record["result"];
                if result["details"]["fatal"] == true {
                    return Err("Pi discovery exhausted its resource budget".into());
                }
                for block in result["content"]
                    .as_array()
                    .ok_or("Pi tool result is invalid")?
                {
                    if block["type"] != "text" {
                        return Err("Pi discovery returned non-text content".into());
                    }
                    self.returned = self.returned.saturating_add(
                        block["text"]
                            .as_str()
                            .ok_or("Pi discovery text invalid")?
                            .len(),
                    );
                    if self.returned > 4 * 1024 * 1024 {
                        return Err("Pi exceeded 4 MiB of discovery text".into());
                    }
                }
            }
            _ => return Err("Pi discovery emitted unsupported incremental tool output".into()),
        }
        Ok(())
    }
}
