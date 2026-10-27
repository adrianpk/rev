//! Configuration keymap contracts. Dispatch activation belongs to the focused
//! input owners; parsing uses the same terminal aliases as runtime keys.
mod defaults;
mod sequence;

pub use sequence::{Sequence, SyntaxError};

/// Independent inherited maps; the minibuffer owner retains its stage-specific
/// argument, search, command, inventory and decision admission rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Editor,
    Minibuffer,
    Tree,
}

impl Context {
    pub const ALL: [Self; 3] = [Self::Editor, Self::Minibuffer, Self::Tree];

    pub fn name(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Minibuffer => "minibuffer",
            Self::Tree => "tree",
        }
    }

    /// Complete configurable defaults. Literal text, decisions and Escape's
    /// finite Meta timing remain operations of their existing owners.
    pub fn defaults(self) -> &'static [(&'static str, &'static str)] {
        defaults::bindings(self)
    }
}
