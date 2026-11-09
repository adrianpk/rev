//! Strict JSON-RPC envelopes and stable application error kinds.

use crate::identity::{Handle, OpaqueId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub jsonrpc: Version,
    pub id: OpaqueId,
    pub method: String,
    #[serde(default = "empty_params")]
    pub params: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notification {
    pub jsonrpc: Version,
    pub method: String,
    #[serde(default = "empty_params")]
    pub params: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Success {
    pub jsonrpc: Version,
    pub id: OpaqueId,
    pub result: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub jsonrpc: Version,
    #[serde(deserialize_with = "required_id")]
    pub id: Option<OpaqueId>,
    pub error: RpcError,
}

fn required_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<OpaqueId>, D::Error> {
    Option::<OpaqueId>::deserialize(deserializer)
}

/// No batch form exists. Requests, notifications and responses cannot overlap.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
    Success(Success),
    Failure(Failure),
}

fn empty_params() -> Value {
    Value::Object(Default::default())
}

impl Message {
    pub fn validate(&self) -> bool {
        match self {
            Self::Request(Request { method, params, .. })
            | Self::Notification(Notification { method, params, .. }) => {
                !method.is_empty()
                    && method.len() <= 128
                    && !method.chars().any(char::is_control)
                    && (params.is_object() || params.is_array())
            }
            Self::Success(_) => true,
            Self::Failure(failure) => failure.error.message.len() <= 4096,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    UnsupportedApi,
    UnsupportedMethod,
    InvalidArguments,
    NamespaceConflict,
    CommandUnavailable,
    MissingBuffer,
    UnsupportedResource,
    StaleRevision,
    ReadOnly,
    InvalidRange,
    EditConflict,
    ConfirmationRequired,
    Cancelled,
    LimitExceeded,
    Disconnected,
    Busy,
    StateConflict,
    UnsupportedInvocation,
}

/// Context is bounded and contains identities, never unrelated source text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorData {
    pub kind: ErrorKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<OpaqueId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ErrorData>,
}

impl RpcError {
    pub fn standard(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.to_owned(),
            data: None,
        }
    }
    pub fn application(kind: ErrorKind) -> Self {
        let code = match kind {
            ErrorKind::UnsupportedMethod => -32601,
            ErrorKind::InvalidArguments => -32602,
            _ => -32000,
        };
        Self {
            code,
            message: format!("{kind:?}"),
            data: Some(ErrorData {
                kind,
                request: None,
                handle: None,
            }),
        }
    }
    pub fn for_request(mut self, id: &OpaqueId) -> Self {
        if let Some(data) = &mut self.data {
            data.request = Some(id.clone());
        }
        self
    }
    pub fn kind(&self) -> Option<ErrorKind> {
        self.data.as_ref().map(|data| data.kind)
    }
}
impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.code)
    }
}
impl std::error::Error for RpcError {}
