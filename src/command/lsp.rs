#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LspCommand {
    Mode,
    Status,
}

impl LspCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Mode => "lsp-mode",
            Self::Status => "lsp-status",
        }
    }
}
