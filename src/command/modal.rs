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
            Self::InsertAfter => "modal-enter-insert-after",
            Self::InsertLineFirst => "modal-enter-insert-line-first",
            Self::InsertLineEnd => "modal-enter-insert-line-end",
            Self::OpenBelow => "modal-open-line-below",
            Self::OpenAbove => "modal-open-line-above",
            Self::VisualCharacter => "modal-enter-visual-character",
            Self::VisualLine => "modal-enter-visual-line",
            Self::VisualBlock => "modal-enter-visual-block",
        }
    }
}
