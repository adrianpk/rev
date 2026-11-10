//! RPC traffic is scoped to one captured invocation identity and deadline.
mod generation;

use super::{
    Control, Output, Request, frame,
    io::{self, Inbox},
};
use crate::config::Pi;
use serde_json::{Value, json};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

pub(super) struct Exchange<'a> {
    pub inbox: &'a Inbox,
    pub input: &'a mut UnixStream,
    pub control: &'a Control,
    pub request: &'a Request,
}

pub(super) struct Generation<'a> {
    pub policy: &'a Pi,
    pub message: &'a str,
    pub model: String,
    pub directory: bool,
    pub history: String,
    pub lease: Option<&'a mut super::history::Lease>,
}

impl Exchange<'_> {
    pub fn generate(&mut self, mut run: Generation<'_>) -> Result<Output, String> {
        let prompt_id = format!("{}:prompt", self.request.epoch);
        io::send(
            self.input,
            &json!({"id":prompt_id,"type":"prompt","message":run.message}),
        )?;
        let mut state = generation::State::default();
        loop {
            if state.accepted && state.settled {
                return Ok(Output {
                    text: state.finish(run.policy.max_result_bytes)?,
                    model: run.model,
                    directory: run.directory,
                    history: run.history,
                });
            }
            if let Some(lease) = run.lease.as_deref_mut() {
                lease.check(false)?;
            }
            let Some(record) = io::next(self.inbox, self.control, self.request.deadline)? else {
                continue;
            };
            state.event(&record, &prompt_id, run.directory, self.control)?;
        }
    }

    pub fn request(&mut self, command: &str, data: Value) -> Result<Value, String> {
        let id = format!("{}:{command}", self.request.epoch);
        let mut record = data;
        record["id"] = json!(id);
        record["type"] = json!(command);
        io::send(self.input, &record)?;
        loop {
            let Some(record) = io::next(self.inbox, self.control, self.request.deadline)? else {
                continue;
            };
            if record["type"] != "response" {
                return Err("Pi emitted unexpected control activity".into());
            }
            response(&record, &id, command)?;
            return Ok(record["data"].clone());
        }
    }

    pub fn admit(&mut self, grant: &Value) -> Result<(), String> {
        let id = format!("{}:admit", self.request.epoch);
        let message = format!("/rev-discovery {grant}");
        io::send(
            self.input,
            &json!({"id":id,"type":"prompt","message":message}),
        )?;
        let deadline = self
            .request
            .deadline
            .min(Instant::now() + Duration::from_secs(5));
        let mut admission = Admission {
            acknowledged: false,
            ready: false,
            epoch: self.request.epoch,
        };
        while !admission.acknowledged || !admission.ready {
            let Some(record) = io::next(self.inbox, self.control, deadline)? else {
                continue;
            };
            admission.event(&record, &id)?;
        }
        Ok(())
    }

    // Cancellation uses declared RPC commands in their documented order. Its
    // finite deadline is independent of generation; late records never publish.
    pub fn cancel(&mut self) {
        let mut cancellation = Cancellation {
            clear: format!("{}:clear", self.request.epoch),
            abort: format!("{}:abort", self.request.epoch),
            cleared: false,
            aborted: false,
        };
        if io::send(
            self.input,
            &json!({"id":cancellation.clear,"type":"clear_queue"}),
        )
        .is_err()
        {
            return;
        }
        if io::send(self.input, &json!({"id":cancellation.abort,"type":"abort"})).is_err() {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let bytes = match self.inbox.next(Duration::from_millis(20)) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => continue,
                Err(_) => return,
            };
            let Ok(record) = serde_json::from_slice::<Value>(&bytes) else {
                return;
            };
            if cancellation.observe(&record) {
                return;
            }
        }
    }
}

pub(super) fn response(record: &Value, id: &str, command: &str) -> Result<(), String> {
    if record["id"] != id || record["command"] != command {
        return Err("Pi response correlation failed".into());
    }
    if record["success"] != true {
        return Err("Pi rejected the RPC command; check Pi authentication/model and retry".into());
    }
    Ok(())
}

struct Admission {
    acknowledged: bool,
    ready: bool,
    epoch: u64,
}
impl Admission {
    fn event(&mut self, record: &Value, id: &str) -> Result<(), String> {
        match record["type"].as_str() {
            Some("response") if !self.acknowledged => {
                response(record, id, "prompt")?;
                if record["data"]["disposition"] != "handled" {
                    return Err("Pi did not handle discovery admission".into());
                }
                self.acknowledged = true;
            }
            Some("extension_ui_request") if !self.ready && record["method"] == "notify" => {
                self.notification(record)?;
                self.ready = true;
            }
            _ => return Err("Pi emitted unexpected discovery startup activity".into()),
        }
        Ok(())
    }

    fn notification(&self, record: &Value) -> Result<(), String> {
        let data: Value = serde_json::from_str(
            record["message"]
                .as_str()
                .ok_or("Pi discovery admission missing")?,
        )
        .map_err(|_| "Pi discovery admission invalid")?;
        let mut names = data["tools"].as_array().cloned().unwrap_or_default();
        names.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        if data["type"] != "rev-discovery-v1"
            || data["nonce"] != self.epoch
            || data["admitted"] != true
            || names != vec![json!("rev_list"), json!("rev_read"), json!("rev_search")]
        {
            return Err("Pi rejected the guarded discovery profile".into());
        }
        Ok(())
    }
}

struct Cancellation {
    clear: String,
    abort: String,
    cleared: bool,
    aborted: bool,
}
impl Cancellation {
    fn observe(&mut self, record: &Value) -> bool {
        if record["type"] == "agent_settled" {
            return true;
        }
        if record["type"] == "response" {
            if response(record, &self.clear, "clear_queue").is_ok() {
                self.cleared = true;
            }
            if response(record, &self.abort, "abort").is_ok() {
                self.aborted = true;
            }
            if self.cleared && self.aborted {
                return true;
            }
        }
        false
    }
}

const NAMES: [&str; 3] = ["rev_read", "rev_list", "rev_search"];
fn allowed(name: &Value) -> bool {
    name.as_str().is_some_and(|name| NAMES.contains(&name))
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
            Some("tool_execution_start") => self.start(id, name),
            Some("tool_execution_end") => self.end(record, id, name),
            _ => Err("Pi discovery emitted unsupported incremental tool output".into()),
        }
    }

    fn start(&mut self, id: &str, name: &str) -> Result<(), String> {
        if !self.seen.insert(id.into()) || self.seen.len() > 32 {
            return Err("Pi exceeded 32 discovery tool calls or repeated an identity".into());
        }
        self.active.insert(id.into(), name.into());
        Ok(())
    }

    fn end(&mut self, record: &Value, id: &str, name: &str) -> Result<(), String> {
        if self.active.remove(id).as_deref() != Some(name) {
            return Err("Pi tool completion correlation failed".into());
        }
        let result = &record["result"];
        if result["details"]["fatal"] == true {
            return Err("Pi discovery exhausted its resource budget".into());
        }
        self.content(
            result["content"]
                .as_array()
                .ok_or("Pi tool result is invalid")?,
        )
    }

    fn content(&mut self, content: &[Value]) -> Result<(), String> {
        for block in content {
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
        Ok(())
    }
}

pub(super) fn model(state: &Value) -> Result<String, String> {
    let provider = state["model"]["provider"]
        .as_str()
        .ok_or("Pi has no model; select and authenticate a model in Pi")?;
    let id = state["model"]["id"]
        .as_str()
        .ok_or("Pi model identity unavailable")?;
    if provider.is_empty()
        || id.is_empty()
        || provider.len() + id.len() > 255
        || provider.chars().chain(id.chars()).any(char::is_control)
    {
        return Err("Pi model identity invalid".into());
    }
    Ok(format!("{provider}/{id}"))
}
pub(super) fn state(
    data: &Value,
    path: Option<&std::path::Path>,
    fresh: bool,
) -> Result<String, String> {
    if data["isStreaming"] != false
        || data["isCompacting"] != false
        || data["pendingMessageCount"] != 0
        || data["messageCount"].as_u64().is_none()
        || (fresh && data["messageCount"] != 0)
    {
        return Err("Pi session was not admitted idle with the expected history".into());
    }
    if data["sessionFile"].as_str().map(std::path::Path::new) != path {
        return Err("Pi selected an unexpected session file".into());
    }
    model(data)
}
pub(super) fn models(data: &Value) -> Result<Vec<super::Model>, String> {
    let models = data["models"]
        .as_array()
        .ok_or("Pi model catalog invalid")?;
    if models.len() > 4096 {
        return Err("Pi model catalog exceeds 4096 entries".into());
    }
    let mut result = Vec::new();
    let mut names = std::collections::BTreeSet::new();
    for value in models {
        let wrapper = json!({"model":value});
        let reference = model(&wrapper)?;
        if !names.insert(reference) {
            continue;
        }
        result.push(super::Model {
            provider: value["provider"].as_str().unwrap().into(),
            id: value["id"].as_str().unwrap().into(),
        });
    }
    Ok(result)
}
