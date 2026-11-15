#![forbid(unsafe_code)]

#[path = "pairs/editor.rs"]
mod editor;
#[path = "configuration/fixture.rs"]
mod fixture;
mod pair_navigation {
    mod expressions;
    pub(super) mod groups;
    pub(super) mod mate;
    mod nesting;
}
