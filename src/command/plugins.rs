/// Native package management requests; runtime availability is checked by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginCommand {
    List,
    Activate,
    Deactivate,
    Restart,
}
impl PluginCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::List => "plugin-list",
            Self::Activate => "plugin-activate",
            Self::Deactivate => "plugin-deactivate",
            Self::Restart => "plugin-restart",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::List => "Inspect configured experimental plugin packages and their availability.",
            Self::Activate => "Activate a configured experimental plugin package.",
            Self::Deactivate => "Deactivate a configured experimental plugin package.",
            Self::Restart => "Restart a configured experimental plugin package.",
        }
    }
}

/// Bounded qualified identity preserves the existing Copy command representation.
/// It neither changes native MacroName grammar nor interns a process callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PluginName {
    bytes: [u8; 130],
    len: u8,
}
impl PluginName {
    pub fn new(name: &str) -> Result<Self, rev_plugin_sdk::identity::InvalidIdentity> {
        let id = rev_plugin_sdk::identity::CommandId::new(name)?;
        let mut value = Self {
            bytes: [0; 130],
            len: u8::try_from(id.as_str().len()).expect("SDK command identities fit 130 bytes"),
        };
        value.bytes[..usize::from(value.len)].copy_from_slice(id.as_str().as_bytes());
        Ok(value)
    }
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.bytes[..usize::from(self.len)])
            .expect("validated ASCII plugin identity")
    }
}

/// Explicit package-to-native policy. Adding a command requires its own typed,
/// noninteractive admission; ordinary user dispatch never consults this policy.
#[derive(Clone, Copy)]
pub(crate) enum Invocation {
    Motion(super::Motion),
    Consent,
    Unsupported,
}
impl super::Command {
    pub(crate) fn invocation(self) -> Invocation {
        match self {
            Self::Motion(motion) => Invocation::Motion(motion),
            Self::SaveBuffer
            | Self::WriteFile
            | Self::Buffer(super::BufferCommand::Kill)
            | Self::Quit
            | Self::QuitAlias => Invocation::Consent,
            _ => Invocation::Unsupported,
        }
    }
}
