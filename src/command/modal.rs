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
}

impl ModalCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "modal-enter-normal",
            Self::Insert => "modal-enter-insert",
            Self::InsertAfter => "modal-enter-insert-after",
            Self::InsertLineFirst => "modal-enter-insert-line-first",
            Self::InsertLineEnd => "modal-enter-insert-line-end",
            Self::OpenBelow => "modal-open-line-below",
            Self::OpenAbove => "modal-open-line-above",
        }
    }
}
