//! Configuration keymap contracts. Dispatch activation belongs to the focused
//! input owners; parsing uses the same terminal aliases as runtime keys.
mod availability;
mod defaults;
mod map;
mod sequence;

pub use map::{Error, Map, Site};
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

/// Three complete maps published together after all candidate checks succeed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective {
    editor: Map,
    minibuffer: Map,
    tree: Map,
}

impl Default for Effective {
    fn default() -> Self {
        compile(&crate::config::Keymaps::default()).expect("audited default keymaps")
    }
}

impl Effective {
    pub fn map(&self, context: Context) -> &Map {
        match context {
            Context::Editor => &self.editor,
            Context::Minibuffer => &self.minibuffer,
            Context::Tree => &self.tree,
        }
    }
}

/// Exact normalized user chords replace inherited entries; all other defaults
/// survive. No map escapes this boundary before context and prefix validation.
pub fn compile(keys: &crate::config::Keymaps) -> Result<Effective, Error> {
    Ok(Effective {
        editor: Map::compile(Context::Editor, &keys.editor)?,
        minibuffer: Map::compile(Context::Minibuffer, &keys.minibuffer)?,
        tree: Map::compile(Context::Tree, &keys.tree)?,
    })
}
