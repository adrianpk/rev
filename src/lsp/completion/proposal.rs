use super::{Activation, Query};
use crate::buffer::Stamp;
use crate::lsp::DocumentVersion;
use ropey::Rope;
use serde_json::Value;
use std::ops::Range;
use std::sync::Arc;

pub(super) struct Context {
    pub token: Arc<Activation>,
    pub version: DocumentVersion,
    pub stamp: Stamp,
    pub text: Rope,
    pub point: usize,
    pub range: Range<usize>,
    pub prefix: String,
    pub uri: String,
    pub defaults: Value,
    pub resolve: bool,
}
impl Context {
    pub fn capture(query: &Query, defaults: Value, resolve: bool) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            token: query.token.clone(),
            version: query.token.version()?,
            stamp: query.stamp,
            text: query.text.clone(),
            point: query.point,
            range: query.range.clone(),
            prefix: query.prefix.clone(),
            uri: query.uri.clone(),
            defaults,
            resolve,
        }))
    }
}

pub(crate) struct Proposal {
    pub(super) context: Arc<Context>,
    item: Value,
}
impl Proposal {
    pub(super) fn new(context: Arc<Context>, item: &Value) -> Arc<Self> {
        Arc::new(Self {
            context,
            item: item.clone(),
        })
    }
    /// Materialization stays on the connection worker, including shared data
    /// defaults. The terminal only clones an Arc to the selected proposal.
    pub fn request(&self) -> Value {
        let mut item = self.item.clone();
        let object = item.as_object_mut().expect("admitted completion object");
        for key in ["data", "insertTextFormat", "insertTextMode"] {
            if let Some(value) = self.context.defaults.get(key) {
                object.entry(key).or_insert_with(|| value.clone());
            }
        }
        self.default_edit(&mut item);
        item
    }
    fn default_edit(&self, item: &mut Value) {
        if item.get("textEdit").is_some() {
            return;
        }
        let Some(range) = self.context.defaults.get("editRange") else {
            return;
        };
        let text = super::admit::default_text(item).expect("admitted default completion text");
        let mut edit = if range.get("insert").is_some() {
            range.clone()
        } else {
            serde_json::json!({"range":range})
        };
        edit["newText"] = Value::String(text.to_owned());
        item["textEdit"] = edit;
    }
    pub(super) fn merge(&self, result: Value) -> Option<Value> {
        let original = self.request();
        result.get("label")?.as_str()?;
        let mut merged = original.clone();
        for (key, value) in result.as_object()? {
            if immutable(key) && original.get(key) != Some(value) {
                return None;
            }
            merged[key] = value.clone();
        }
        Some(merged)
    }
}
fn immutable(key: &str) -> bool {
    matches!(
        key,
        "label"
            | "filterText"
            | "sortText"
            | "insertText"
            | "textEdit"
            | "textEditText"
            | "insertTextFormat"
            | "insertTextMode"
    )
}
impl std::fmt::Debug for Proposal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proposal")
            .field("version", &self.context.version)
            .finish_non_exhaustive()
    }
}
impl PartialEq for Proposal {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.context, &other.context) && self.item == other.item
    }
}
impl Eq for Proposal {}
