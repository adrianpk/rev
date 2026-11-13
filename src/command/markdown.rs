/// Native Markdown presentation and checked source actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkdownCommand {
    ContinueItem,
    IndentItem,
    OutdentItem,
    Return,
    MetaReturn,
    TableCreate,
    TableAlign,
    TableNextField,
    TablePreviousField,
    TableNextRow,
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
            _ => self.table_name(),
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
            Self::Return => {
                "Move to the next Markdown table row, continue a list or insert a newline."
            }
            Self::MetaReturn => "Insert a Markdown list sibling or an ordinary newline.",
            Self::Pretty => {
                "Toggle this view's Markdown heading and list symbols without changing source."
            }
            Self::ResetPresentation => {
                "Restore this view's startup Markdown presentation without writing settings."
            }
            _ => self.table_description(),
        }
    }

    fn table_name(self) -> &'static str {
        match self {
            Self::TableCreate => "markdown-table-create",
            Self::TableAlign => "markdown-table-align",
            Self::TableNextField => "markdown-table-next-field",
            Self::TablePreviousField => "markdown-table-previous-field",
            Self::TableNextRow => "markdown-table-next-row",
            _ => unreachable!("table name dispatch"),
        }
    }

    fn table_description(self) -> &'static str {
        match self {
            Self::TableCreate => {
                "Create a Markdown table with explicit column and body-row counts."
            }
            Self::TableAlign => "Align a Markdown table while preserving original cell source.",
            Self::TableNextField => {
                "Move to the next Markdown table field, appending a body row when needed."
            }
            Self::TablePreviousField => {
                "Move to the previous Markdown table field, skipping the delimiter row."
            }
            Self::TableNextRow => {
                "Move to the same field in the next Markdown body row, appending when needed."
            }
            _ => unreachable!("table description dispatch"),
        }
    }
}
