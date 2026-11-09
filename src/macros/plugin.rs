//! Inert qualified calls. Session admission checks the current declaration;
//! no process, handler, source handle or generation is part of a definition.
use rev_plugin_sdk::identity::CommandId;

#[derive(Clone)]
pub(crate) struct PluginCall {
    pub command: CommandId,
    pub arguments: serde_json::Value,
    pub prefix: Option<i64>,
}
impl PluginCall {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.arguments.is_object() {
            return Err("plugin argument object");
        }
        let bytes = serde_json::to_vec(&self.arguments).map_err(|_| "plugin arguments")?;
        if bytes.len() > super::DEFINITION_BYTES {
            return Err("plugin argument bytes");
        }
        Ok(())
    }
    pub fn bytes(&self) -> usize {
        serde_json::to_string(&self.arguments).map_or(usize::MAX, |arguments| {
            super::capture::text_bytes(&arguments)
                .saturating_add(self.command.as_str().len())
                .saturating_add(128)
        })
    }
}
