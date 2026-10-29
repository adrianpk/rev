//! Configuration keymap contracts. Dispatch activation belongs to the focused
//! input owners; parsing uses the same terminal aliases as runtime keys.
mod availability;
mod cancellation;
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
    Org,
    OrgAgenda,
    OrgCapture,
    OrgCalendar,
    OrgSourceEdit,
}

impl Context {
    pub const ALL: [Self; 8] = [
        Self::Editor,
        Self::Minibuffer,
        Self::Tree,
        Self::Org,
        Self::OrgAgenda,
        Self::OrgCapture,
        Self::OrgCalendar,
        Self::OrgSourceEdit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Minibuffer => "minibuffer",
            Self::Tree => "tree",
            Self::Org => "org",
            Self::OrgAgenda => "org_agenda",
            Self::OrgCapture => "org_capture",
            Self::OrgCalendar => "org_calendar",
            Self::OrgSourceEdit => "org_source_edit",
        }
    }

    /// Complete factory bindings, including inherited editor entries. Literal
    /// text, decisions and Escape timing remain with their existing owners.
    pub fn defaults(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Org => defaults::org_complete(),
            Self::OrgCalendar => defaults::calendar_complete(),
            Self::OrgCapture => defaults::capture_complete(),
            Self::OrgSourceEdit => defaults::source_complete(),
            _ => self.local_defaults(),
        }
    }

    /// Local layer only; empty Org editing overlays inherit the admitted editor
    /// map. The exhaustive sample declares these entries without duplicating it.
    pub fn local_defaults(self) -> &'static [(&'static str, &'static str)] {
        defaults::bindings(self)
    }
}

/// Complete maps published together after all candidate checks succeed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Effective {
    editor: Map,
    minibuffer: Map,
    tree: Map,
    org: Map,
    org_agenda: Map,
    org_capture: Map,
    org_calendar: Map,
    org_source_edit: Map,
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
            Context::Org => &self.org,
            Context::OrgAgenda => &self.org_agenda,
            Context::OrgCapture => &self.org_capture,
            Context::OrgCalendar => &self.org_calendar,
            Context::OrgSourceEdit => &self.org_source_edit,
        }
    }
}

/// Exact normalized user chords replace inherited entries; all other defaults
/// survive. No map escapes this boundary before context and prefix validation.
pub fn compile(keys: &crate::config::Keymaps) -> Result<Effective, Error> {
    let editor = Map::compile(Context::Editor, &keys.editor)?;
    let minibuffer = Map::compile(Context::Minibuffer, &keys.minibuffer)?;
    let org_calendar = Map::inherit(&minibuffer, Context::OrgCalendar, &keys.org_calendar)?;
    Ok(Effective {
        org: Map::inherit(&editor, Context::Org, &keys.org)?,
        org_capture: Map::inherit(&editor, Context::OrgCapture, &keys.org_capture)?,
        org_source_edit: Map::inherit(&editor, Context::OrgSourceEdit, &keys.org_source_edit)?,
        editor,
        minibuffer,
        tree: Map::compile(Context::Tree, &keys.tree)?,
        org_agenda: Map::compile(Context::OrgAgenda, &keys.org_agenda)?,
        org_calendar,
    })
}
