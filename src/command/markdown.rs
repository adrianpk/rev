/// Native Markdown presentation and checked source actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkdownCommand {
    ContinueItem,
    IndentItem,
    OutdentItem,
    Return,
    MetaReturn,
    Pretty,
    ResetPresentation,
}

impl MarkdownCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::ContinueItem => "markdown-continue-item",
            Self::IndentItem => "markdown-indent-item",
            Self::OutdentItem => "markdown-outdent-item",
            Self::Return => "markdown-return",
            Self::MetaReturn => "markdown-meta-return",
            Self::Pretty => "markdown-pretty-mode",
            Self::ResetPresentation => "markdown-reset-presentation",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::ContinueItem => "Continue this Markdown item or exit one empty list level.",
            Self::IndentItem => {
                "Indent this Markdown item with its children and continuation text."
            }
            Self::OutdentItem => {
                "Outdent this Markdown item with its children and continuation text."
            }
            Self::Return => "Continue a Markdown list item or insert an ordinary newline.",
            Self::MetaReturn => "Insert a Markdown list sibling or an ordinary newline.",
            Self::Pretty => {
                "Toggle this view's Markdown heading and list symbols without changing source."
            }
            Self::ResetPresentation => {
                "Restore this view's startup Markdown presentation without writing settings."
            }
        }
    }
}
