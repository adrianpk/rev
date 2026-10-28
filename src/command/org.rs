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
}
impl OrgCommand {
    pub const ALL: [Self; 18] = [
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
        ][self as usize]
    }
}
