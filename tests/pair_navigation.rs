#![forbid(unsafe_code)]

#[path = "pairs/editor.rs"]
mod editor;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "support/surface.rs"]
mod surface;
mod pair_navigation {
    mod acceptance;
    mod expressions;
    pub(super) mod groups;
    pub(super) mod mate;
    mod nesting;
}
