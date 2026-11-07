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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Toggle => "Hide or restore numbers, retaining mode.",
            Self::CycleMode => "Alternate absolute and relative, retaining visibility.",
            Self::Absolute => "Show absolute logical line numbers.",
            Self::Relative => "Show distances from this view's logical point.",
            Self::CycleCurrentLine => "Cycle absolute, zero and hidden current labels.",
            Self::ToggleSide => "Move this view's number field left or right.",
            Self::Inspect => "Report effective policy and startup/view override sources.",
            Self::Reset => "Restore this view's immutable startup policy.",
        }
    }
}
