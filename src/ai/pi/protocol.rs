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

pub(super) fn run(
    inbox: &Inbox,
    input: &mut UnixStream,
    control: &Control,
    policy: &Pi,
    message: &str,
    epoch: u64,
    deadline: Instant,
) -> Result<Output, String> {
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
            return Err("Pi did not admit a fresh input-only session".into());
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
    loop {
        if accepted && settled {
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
                return Err("Pi attempted a tool under an input-only grant".into());
            }
            Some("extension_ui_request") => {
                return Err("Pi requested unavailable UI; authenticate in Pi and retry".into());
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
                | "auto_compaction_end",
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
