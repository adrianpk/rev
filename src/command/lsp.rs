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

    pub(crate) fn description(self) -> &'static str {
        if let Some(description) = self.management_description() {
            return description;
        }
        match self {
            Self::Mode => "Toggle optional language-server support for this session.",
            Self::Status => "Show language-server status for the current file.",
            Self::Connect => {
                "Restore this buffer eligibility without enabling globally disabled LSP."
            }
            Self::Disconnect => {
                "Exclude this canonical buffer across all its views until restored or closed."
            }
            Self::Restart => {
                "Restart the eligible current root; preserve global mode and buffer exclusions."
            }
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

    fn management_description(self) -> Option<&'static str> {
        Some(match self {
            Self::Setup => "Set up an installed or managed language server.",
            Self::ReloadSettings => {
                "Reload LSP settings and associations while preserving runtime choices."
            }
            Self::Servers => {
                "List configured language servers, availability and installation prerequisites."
            }
            Self::SelectServer => {
                "Persist a compatible configured source, ready managed installation or native-only choice."
            }
            Self::InstallServer => "Install or select a supported managed language server.",
            Self::UpdateServer => {
                "Verify a replacement before retargeting selected managed profiles; retain the old payload."
            }
            Self::RemoveServer => {
                "Remove a confirmed installer-owned payload only when unreferenced and stopped."
            }
            Self::CancelInstallation => {
                "Request cancellation of the active server management operation."
            }
            _ => return None,
        })
    }
}
