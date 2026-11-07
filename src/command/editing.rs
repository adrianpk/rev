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
    Repeat,
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

    pub(crate) fn description(self) -> &'static str {
        match self.action {
            Action::Delete => "Delete one checked target into the shared kill ring.",
            Action::Change => {
                "Retain/delete the target, then enter Insert after successful publication."
            }
            Action::Copy => {
                "Copy the target to the shared kill ring without editing or moving point."
            }
            Action::Indent => {
                "Indent each intersecting logical line by one active indentation unit."
            }
            Action::Outdent => {
                "Outdent each intersecting logical line by one active indentation unit."
            }
            Action::DeleteEnd => {
                "Delete through the logical content-end boundary; a count selects following lines."
            }
            Action::ChangeEnd => {
                "Change through the logical content-end boundary, then enter Insert."
            }
            Action::CopyEnd => {
                "Copy through the logical content-end boundary, retaining linear shape."
            }
            _ => self.direct_description(),
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
            Action::Repeat => "modal-repeat-last-change",
            _ => unreachable!("operator names delegate before direct edit lookup"),
        }
    }

    fn direct_description(self) -> &'static str {
        match self.action {
            Action::DeleteForward => {
                "Delete counted whole graphemes within this logical line, preserving retained text."
            }
            Action::DeleteBackward => {
                "Delete counted preceding whole graphemes within this logical line, preserving retained text."
            }
            Action::Replace => {
                "Replace counted complete graphemes within this logical line with one printable Unicode scalar."
            }
            Action::PasteAfter => {
                "Paste retained text after the current grapheme, or after this logical line for linewise text."
            }
            Action::PasteBefore => {
                "Paste retained text at point, or before this logical line for linewise text."
            }
            Action::Join => "Join logical lines, adjusting the whitespace between them.",
            Action::SelectInner => {
                "Select a counted inner word or paragraph through native endpoints."
            }
            Action::SelectAround => {
                "Select a counted word or paragraph with its surrounding whitespace."
            }
            Action::Repeat => {
                "Resolve and repeat the latest admitted semantic change; a count overrides its count."
            }
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
