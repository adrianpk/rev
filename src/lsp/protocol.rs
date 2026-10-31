use serde::Deserialize;
use serde_json::{Value, json};

use crate::config::Server;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    Utf8,
    Utf16,
}

/// Negotiated capabilities for the admitted client subset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub encoding: Encoding,
    pub sync: u64,
    pub completion: bool,
    pub triggers: Vec<String>,
    pub open_close: bool,
    pub save: Save,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Save {
    None,
    Notification,
    Text,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TextSync {
    Kind(u64),
    Options(SyncOptions),
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SyncOptions {
    open_close: bool,
    change: u64,
    save: Option<SaveSetting>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SaveSetting {
    Enabled(bool),
    Options {
        #[serde(default, rename = "includeText")]
        include_text: bool,
    },
}

fn synchronization(value: Option<TextSync>) -> Result<(u64, bool, Save), String> {
    let (kind, open_close, save) = match value {
        Some(TextSync::Kind(kind)) => (kind, true, None),
        Some(TextSync::Options(options)) => (options.change, options.open_close, options.save),
        None => (0, false, None),
    };
    if !matches!(kind, 1 | 2) || !open_close {
        return Err(
            "LSP server lacks admitted open/close and full/incremental synchronization".into(),
        );
    }
    let save = match save {
        Some(SaveSetting::Enabled(true)) => Save::Notification,
        Some(SaveSetting::Options { include_text: true }) => Save::Text,
        Some(SaveSetting::Options {
            include_text: false,
        }) => Save::Notification,
        _ => Save::None,
    };
    Ok((kind, open_close, save))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeResult {
    capabilities: ServerCapabilities,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerCapabilities {
    position_encoding: Option<String>,
    text_document_sync: Option<TextSync>,
    completion_provider: Option<Value>,
}

pub(super) fn capabilities(value: Value) -> Result<Capabilities, String> {
    let response: InitializeResult =
        serde_json::from_value(value).map_err(|e| format!("LSP initialize result: {e}"))?;
    let value = response.capabilities;
    let encoding = match value.position_encoding.as_deref().unwrap_or("utf-16") {
        "utf-8" => Encoding::Utf8,
        "utf-16" => Encoding::Utf16,
        _ => return Err("LSP server selected unsupported position encoding".into()),
    };
    let (sync, open_close, save) = synchronization(value.text_document_sync)?;
    Ok(Capabilities {
        encoding,
        sync,
        open_close,
        save,
        completion: value
            .completion_provider
            .as_ref()
            .is_some_and(|v| v.is_object()),
        triggers: triggers(value.completion_provider.as_ref()),
    })
}

pub(super) fn initialize(server: &Server, root: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{
        "processId": std::process::id(), "clientInfo":{"name":"rev"},
        "rootUri":root, "workspaceFolders":[{"uri":root,"name":"root"}],
        "capabilities":{"general":{"positionEncodings":["utf-8","utf-16"]},
            "workspace":{"configuration":true,"workspaceFolders":true},
            "textDocument":{"synchronization":{"dynamicRegistration":false,"willSave":false,"willSaveWaitUntil":false,"didSave":true},"completion":{"dynamicRegistration":false,
                "contextSupport":true,"completionList":{"itemDefaults":["editRange","insertTextFormat","insertTextMode"]},"completionItem":{"snippetSupport":false,"insertReplaceSupport":true,"insertTextModeSupport":{"valueSet":[1]}}}}},
        "initializationOptions":server.initialization_options
    }})
}

pub(super) fn request(message: &super::frame::Message, server: &Server, root: &str) -> Value {
    let id = &message.id;
    let result = match message.method.as_deref() {
        Some("workspace/configuration") => {
            let items = message.params.get("items").and_then(Value::as_array);
            if items.is_some_and(|items| items.len() > 64) {
                return json!({"jsonrpc":"2.0","id":id,"error":{"code":-32602,"message":"Configuration item limit exceeded"}});
            }
            let values: Vec<Value> = items
                .into_iter()
                .flatten()
                .take(64)
                .map(|item| {
                    let section = item.get("section").and_then(Value::as_str).unwrap_or("");
                    let settings = serde_json::to_value(&server.settings).unwrap_or(Value::Null);
                    section
                        .split('.')
                        .filter(|part| !part.is_empty())
                        .fold(settings, |value, part| {
                            value.get(part).cloned().unwrap_or(Value::Null)
                        })
                })
                .collect();
            json!(values)
        }
        Some("workspace/workspaceFolders") => json!([{"uri":root,"name":"root"}]),
        Some("window/workDoneProgress/create") => Value::Null,
        Some("workspace/applyEdit") => {
            json!({"applied":false,"failureReason":"Server edits are not admitted"})
        }
        Some("window/showMessageRequest") => Value::Null,
        _ => {
            return json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"Method not supported"}});
        }
    };
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

fn triggers(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(|value| value.get("triggerCharacters"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(64)
        .filter_map(Value::as_str)
        .filter(|text| text.chars().count() == 1)
        .map(str::to_owned)
        .collect()
}
