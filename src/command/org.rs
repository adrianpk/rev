/// Implemented native Org outline operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum OrgCommand {
    Cycle,
    GlobalCycle,
    Overview,
    Contents,
    ShowAll,
    HideSubtree,
    ShowChildren,
    ShowSubtree,
    HideDrawer,
    ShowDrawer,
    ToggleDrawer,
    InsertHeading,
    NextHeading,
    PreviousHeading,
    ForwardHeading,
    BackwardHeading,
    ParentHeading,
    MetaReturn,
    PromoteSubtree,
    DemoteSubtree,
    MoveSubtreeUp,
    MoveSubtreeDown,
    MetaLeft,
    MetaRight,
    MetaUp,
    MetaDown,
    InsertItem,
    IndentItem,
    OutdentItem,
    IndentItemTree,
    OutdentItemTree,
    MoveItemUp,
    MoveItemDown,
    RenumberList,
}
impl OrgCommand {
    pub const ALL: [Self; 34] = [
        Self::Cycle,
        Self::GlobalCycle,
        Self::Overview,
        Self::Contents,
        Self::ShowAll,
        Self::HideSubtree,
        Self::ShowChildren,
        Self::ShowSubtree,
        Self::HideDrawer,
        Self::ShowDrawer,
        Self::ToggleDrawer,
        Self::InsertHeading,
        Self::NextHeading,
        Self::PreviousHeading,
        Self::ForwardHeading,
        Self::BackwardHeading,
        Self::ParentHeading,
        Self::MetaReturn,
        Self::PromoteSubtree,
        Self::DemoteSubtree,
        Self::MoveSubtreeUp,
        Self::MoveSubtreeDown,
        Self::MetaLeft,
        Self::MetaRight,
        Self::MetaUp,
        Self::MetaDown,
        Self::InsertItem,
        Self::IndentItem,
        Self::OutdentItem,
        Self::IndentItemTree,
        Self::OutdentItemTree,
        Self::MoveItemUp,
        Self::MoveItemDown,
        Self::RenumberList,
    ];
    pub fn name(self) -> &'static str {
        [
            "org-cycle",
            "org-shifttab",
            "org-overview",
            "org-content",
            "org-show-all",
            "org-hide-subtree",
            "org-show-children",
            "org-show-subtree",
            "org-hide-drawer",
            "org-show-drawer",
            "org-toggle-drawer",
            "org-insert-heading",
            "org-next-visible-heading",
            "org-previous-visible-heading",
            "org-forward-heading-same-level",
            "org-backward-heading-same-level",
            "org-up-heading-safe",
            "org-meta-return",
            "org-promote-subtree",
            "org-demote-subtree",
            "org-move-subtree-up",
            "org-move-subtree-down",
            "org-metaleft",
            "org-metaright",
            "org-metaup",
            "org-metadown",
            "org-insert-item",
            "org-indent-item",
            "org-outdent-item",
            "org-indent-item-tree",
            "org-outdent-item-tree",
            "org-move-item-up",
            "org-move-item-down",
            "org-renumber-list",
        ][self as usize]
    }

    pub(crate) fn list_insertion(self) -> bool {
        matches!(self, Self::MetaReturn | Self::InsertItem)
    }
    pub(crate) fn list_indentation(self) -> bool {
        matches!(
            self,
            Self::IndentItem
                | Self::OutdentItem
                | Self::IndentItemTree
                | Self::OutdentItemTree
                | Self::MetaLeft
                | Self::MetaRight
        )
    }
    pub(crate) fn list_movement(self) -> bool {
        matches!(
            self,
            Self::MoveItemUp | Self::MoveItemDown | Self::MetaUp | Self::MetaDown
        )
    }
    pub(crate) fn list_only(self) -> bool {
        matches!(
            self,
            Self::InsertItem
                | Self::IndentItem
                | Self::OutdentItem
                | Self::IndentItemTree
                | Self::OutdentItemTree
                | Self::MoveItemUp
                | Self::MoveItemDown
                | Self::RenumberList
        )
    }
    pub(crate) fn contextual_list(self) -> bool {
        matches!(
            self,
            Self::MetaReturn | Self::MetaLeft | Self::MetaRight | Self::MetaUp | Self::MetaDown
        )
    }
    pub(crate) fn subtree(self) -> Option<Self> {
        match self {
            Self::MetaLeft => Some(Self::PromoteSubtree),
            Self::MetaRight => Some(Self::DemoteSubtree),
            Self::MetaUp => Some(Self::MoveSubtreeUp),
            Self::MetaDown => Some(Self::MoveSubtreeDown),
            command if command.edits_subtree() => Some(command),
            _ => None,
        }
    }
    pub(crate) fn edits_subtree(self) -> bool {
        matches!(
            self,
            Self::PromoteSubtree
                | Self::DemoteSubtree
                | Self::MoveSubtreeUp
                | Self::MoveSubtreeDown
        )
    }
}
