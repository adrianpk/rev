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
    text_document_sync: Option<Value>,
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
    let sync = value
        .text_document_sync
        .as_ref()
        .and_then(|sync| {
            sync.as_u64()
                .or_else(|| sync.get("change").and_then(Value::as_u64))
        })
        .unwrap_or(0);
    if !matches!(sync, 1 | 2) {
        return Err("LSP server lacks full or incremental text synchronization".into());
    }
    Ok(Capabilities {
        encoding,
        sync,
        completion: value.completion_provider.is_some_and(|v| v.is_object()),
    })
}

pub(super) fn initialize(server: &Server, root: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{
        "processId": std::process::id(), "clientInfo":{"name":"rev"},
        "rootUri":root, "workspaceFolders":[{"uri":root,"name":"root"}],
        "capabilities":{"general":{"positionEncodings":["utf-8","utf-16"]},
            "workspace":{"configuration":true,"workspaceFolders":true},
            "textDocument":{"completion":{"dynamicRegistration":false,
                "completionItem":{"snippetSupport":false}}}},
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
