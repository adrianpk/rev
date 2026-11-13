/// Native Markdown view controls; these commands never edit canonical text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkdownCommand {
    Pretty,
    ResetPresentation,
}

impl MarkdownCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Pretty => "markdown-pretty-mode",
            Self::ResetPresentation => "markdown-reset-presentation",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Pretty => "Toggle this view's Markdown heading symbols without changing source.",
            Self::ResetPresentation => {
                "Restore this view's startup Markdown presentation without writing settings."
            }
        }
    }
}
