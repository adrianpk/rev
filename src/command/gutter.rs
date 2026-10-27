/// Compiled presentation operations, available through M-x or editor mappings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GutterCommand {
    Toggle,
    CycleMode,
    Absolute,
    Relative,
    CycleCurrentLine,
    ToggleSide,
    Inspect,
    Reset,
}

impl GutterCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Toggle => "gutter-toggle-line-numbers",
            Self::CycleMode => "gutter-cycle-line-number-mode",
            Self::Absolute => "gutter-set-line-numbers-absolute",
            Self::Relative => "gutter-set-line-numbers-relative",
            Self::CycleCurrentLine => "gutter-cycle-current-line",
            Self::ToggleSide => "gutter-toggle-line-number-side",
            Self::Inspect => "gutter-inspect-numbering",
            Self::Reset => "gutter-reset-numbering",
        }
    }
}
