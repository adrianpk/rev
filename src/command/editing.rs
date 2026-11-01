use super::Navigation;
use crate::buffer::edit::Point;
use crate::coordinates::Scalar;

/// One complete edit intent; the command stays Copy for native deferred input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Editing {
    pub action: Action,
    pub count: Option<u32>,
    pub operand: Option<Operand>,
    pub character: Option<char>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Delete,
    Change,
    Copy,
    Indent,
    Outdent,
    DeleteEnd,
    ChangeEnd,
    CopyEnd,
    DeleteForward,
    DeleteBackward,
    Replace,
    PasteAfter,
    PasteBefore,
    Join,
    SelectInner,
    SelectAround,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    Motion(Navigation),
    Lines,
    Object(Object),
    Resolved { origin: Point, end: Scalar },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Object {
    Word { big: bool, around: bool },
    Paragraph { around: bool },
    Pair { open: char, around: bool },
    Quote { quote: char, around: bool },
}

impl Editing {
    pub const fn new(action: Action) -> Self {
        Self {
            action,
            count: None,
            operand: None,
            character: None,
        }
    }

    pub fn amount(self) -> usize {
        self.count.unwrap_or(1).max(1) as usize
    }

    pub fn name(self) -> &'static str {
        match self.action {
            Action::Delete => "modal-delete",
            Action::Change => "modal-change",
            Action::Copy => "modal-copy",
            Action::Indent => "modal-indent",
            Action::Outdent => "modal-outdent",
            Action::DeleteEnd => "modal-delete-line-end",
            Action::ChangeEnd => "modal-change-line-end",
            Action::CopyEnd => "modal-copy-line-end",
            _ => self.direct_name(),
        }
    }

    fn direct_name(self) -> &'static str {
        match self.action {
            Action::DeleteForward => "modal-delete-forward",
            Action::DeleteBackward => "modal-delete-backward",
            Action::Replace => "modal-replace-character",
            Action::PasteAfter => "modal-paste-after",
            Action::PasteBefore => "modal-paste-before",
            Action::Join => "modal-join-lines",
            Action::SelectInner => "modal-select-inner-object",
            Action::SelectAround => "modal-select-around-object",
            _ => unreachable!("operator names delegate before direct edit lookup"),
        }
    }

    pub fn pending(self) -> bool {
        self.action.operator() && self.operand.is_none()
    }
}

impl Action {
    pub fn selects(self) -> bool {
        matches!(self, Self::SelectInner | Self::SelectAround)
    }

    pub fn operator(self) -> bool {
        matches!(
            self,
            Self::Delete | Self::Change | Self::Copy | Self::Indent | Self::Outdent
        )
    }
    pub fn changes(self) -> bool {
        matches!(self, Self::Change | Self::ChangeEnd)
    }
    pub fn copies(self) -> bool {
        matches!(self, Self::Copy | Self::CopyEnd)
    }
    pub fn shifts(self) -> bool {
        matches!(self, Self::Indent | Self::Outdent)
    }
}
