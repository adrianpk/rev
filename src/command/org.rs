/// Implemented native outline/drawer presentation operations.
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
}
impl OrgCommand {
    pub const ALL: [Self; 11] = [
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
        ][self as usize]
    }
}
