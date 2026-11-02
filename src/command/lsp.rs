#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LspCommand {
    Mode,
    Status,
    Connect,
    Disconnect,
    Restart,
    Setup,
    ReloadSettings,
    Servers,
    SelectServer,
    InstallServer,
}

impl LspCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mode => "lsp-mode",
            Self::Status => "lsp-status",
            Self::Connect => "lsp-connect",
            Self::Disconnect => "lsp-disconnect",
            Self::Restart => "lsp-restart",
            Self::Setup => "lsp-setup",
            Self::ReloadSettings => "lsp-reload-settings",
            Self::Servers => "lsp-servers",
            Self::SelectServer => "lsp-select-server",
            Self::InstallServer => "lsp-install-server",
        }
    }
}
