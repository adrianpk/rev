/// Implemented state-entry operations over the native point and edit owners.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalCommand {
    Normal,
    Insert,
    InsertAfter,
    InsertLineFirst,
    InsertLineEnd,
    OpenBelow,
    OpenAbove,
    VisualCharacter,
    VisualLine,
    VisualBlock,
    Once,
    Replace,
}

impl ModalCommand {
    pub(crate) fn visual(self) -> Option<crate::buffer::Visual> {
        match self {
            Self::VisualCharacter => Some(crate::buffer::Visual::Character),
            Self::VisualLine => Some(crate::buffer::Visual::Line),
            Self::VisualBlock => Some(crate::buffer::Visual::Block),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "modal-enter-normal",
            Self::Insert => "modal-enter-insert",
            Self::Replace => "modal-enter-replace",
            Self::InsertAfter => "modal-enter-insert-after",
            Self::InsertLineFirst => "modal-enter-insert-line-first",
            Self::InsertLineEnd => "modal-enter-insert-line-end",
            Self::OpenBelow => "modal-open-line-below",
            Self::OpenAbove => "modal-open-line-above",
            _ => self.visual_name(),
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Normal => "Return to Normal, preserving the native point.",
            Self::Insert => "Enter Insert at point.",
            Self::Replace => "Enter persistent Replace with an underline cursor.",
            Self::InsertAfter => {
                "Enter Insert after the current grapheme, stopping at logical EOL."
            }
            Self::InsertLineFirst => "Enter Insert at the first non-whitespace boundary.",
            Self::InsertLineEnd => "Enter Insert at logical EOL.",
            Self::OpenBelow => "Open one logical line below and enter Insert.",
            Self::OpenAbove => "Open one logical line above and enter Insert.",
            _ => self.visual_description(),
        }
    }
    fn visual_name(self) -> &'static str {
        match self {
            Self::VisualCharacter => "modal-enter-visual-character",
            Self::VisualLine => "modal-enter-visual-line",
            Self::VisualBlock => "modal-enter-visual-block",
            Self::Once => "modal-one-normal-command",
            _ => unreachable!("ordinary state names are handled first"),
        }
    }

    fn visual_description(self) -> &'static str {
        match self {
            Self::VisualCharacter => {
                "Select the current grapheme, convert shape or leave a repeated Character state."
            }
            Self::VisualLine => {
                "Select complete logical lines, convert shape or leave a repeated Line state."
            }
            Self::VisualBlock => {
                "Select inclusive logical grapheme columns, convert shape or leave a repeated Block state."
            }
            Self::Once => "Admit one complete Normal command, then resume compatible insertion.",
            _ => unreachable!("ordinary state names are handled first"),
        }
    }
}
