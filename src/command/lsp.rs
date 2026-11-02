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
    UpdateServer,
    RemoveServer,
    CancelInstallation,
}

impl LspCommand {
    pub fn name(self) -> &'static str {
        if let Some(name) = self.management_name() {
            return name;
        }
        match self {
            Self::Mode => "lsp-mode",
            Self::Status => "lsp-status",
            Self::Connect => "lsp-connect",
            Self::Disconnect => "lsp-disconnect",
            Self::Restart => "lsp-restart",
            _ => unreachable!("management names handled above"),
        }
    }

    fn management_name(self) -> Option<&'static str> {
        Some(match self {
            Self::Setup => "lsp-setup",
            Self::ReloadSettings => "lsp-reload-settings",
            Self::Servers => "lsp-servers",
            Self::SelectServer => "lsp-select-server",
            Self::InstallServer => "lsp-install-server",
            Self::UpdateServer => "lsp-update-server",
            Self::RemoveServer => "lsp-remove-server",
            Self::CancelInstallation => "lsp-cancel-installation",
            _ => return None,
        })
    }
}
