//! Configuration keymap contracts. Dispatch activation belongs to the focused
//! input owners; parsing uses the same terminal aliases as runtime keys.
mod availability;
mod cancellation;
mod defaults;
mod map;
mod sequence;
mod vim;
mod vim_editing;
mod vim_navigation;

use std::collections::BTreeMap;

pub use map::{Error, Map, Site};
pub use sequence::{Sequence, SyntaxError};

/// Independent inherited maps; the minibuffer owner retains its stage-specific
/// argument, search, command, inventory and decision admission rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Context {
    VimNormal,
    VimInsert,
    VimReplace,
    VimVisualCharacter,
    VimVisualLine,
    VimVisualBlock,

    Completion,
    Editor,
    Minibuffer,
    Tree,
    BookmarkList,
    Git,
    GitCommit,
    Org,
    OrgAgenda,
    OrgCapture,
    OrgCalendar,
    OrgSourceEdit,
}

impl Context {
    pub const ALL: [Self; 18] = [
        Self::VimNormal,
        Self::VimInsert,
        Self::VimReplace,
        Self::VimVisualCharacter,
        Self::VimVisualLine,
        Self::VimVisualBlock,
        Self::Completion,
        Self::Editor,
        Self::Minibuffer,
        Self::Tree,
        Self::BookmarkList,
        Self::Git,
        Self::GitCommit,
        Self::Org,
        Self::OrgAgenda,
        Self::OrgCapture,
        Self::OrgCalendar,
        Self::OrgSourceEdit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Completion => "completion",
            Self::Editor => "editor",
            Self::Minibuffer => "minibuffer",
            Self::Tree => "tree",
            Self::BookmarkList => "bookmark_list",
            Self::Git => "git",
            Self::GitCommit => "git_commit",
            _ => self.modal_name(),
        }
    }

    fn modal_name(self) -> &'static str {
        match self {
            Self::VimNormal => "vim_normal",
            Self::VimInsert => "vim_insert",
            Self::VimReplace => "vim_replace",
            Self::VimVisualCharacter => "vim_visual_character",
            Self::VimVisualLine => "vim_visual_line",
            Self::VimVisualBlock => "vim_visual_block",
            _ => self.org_name(),
        }
    }

    fn org_name(self) -> &'static str {
        match self {
            Self::Org => "org",
            Self::OrgAgenda => "org_agenda",
            Self::OrgCapture => "org_capture",
            Self::OrgCalendar => "org_calendar",
            Self::OrgSourceEdit => "org_source_edit",
            _ => unreachable!("non-Org context has a separate name owner"),
        }
    }

    /// Complete factory bindings, including inherited editor entries. Literal
    /// text, decisions and Escape timing remain with their existing owners.
    pub fn defaults(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::GitCommit => defaults::commit_complete(),
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
    vim: BTreeMap<Context, Map>,
    vim_org: BTreeMap<(Context, Context), Map>,
    completion: Map,
    editor: Map,
    minibuffer: Map,
    tree: Map,
    bookmark_list: Map,
    git: Map,
    git_commit: Map,
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
        if let Some(map) = self.vim.get(&context) {
            return map;
        }
        self.native_map(context)
    }
    fn native_map(&self, context: Context) -> &Map {
        match context {
            Context::Completion => &self.completion,
            Context::Editor => &self.editor,
            Context::Minibuffer => &self.minibuffer,
            Context::Tree => &self.tree,
            Context::BookmarkList => &self.bookmark_list,
            Context::Git => &self.git,
            Context::GitCommit => &self.git_commit,
            _ => self.org_map(context),
        }
    }
    fn org_map(&self, context: Context) -> &Map {
        match context {
            Context::Org => &self.org,
            Context::OrgAgenda => &self.org_agenda,
            Context::OrgCapture => &self.org_capture,
            Context::OrgCalendar => &self.org_calendar,
            Context::OrgSourceEdit => &self.org_source_edit,
            _ => unreachable!("non-Org map resolved by its owner"),
        }
    }
}

/// Exact normalized user chords replace inherited entries; all other defaults
/// survive. No map escapes this boundary before context and prefix validation.
pub fn compile(keys: &crate::config::Keymaps) -> Result<Effective, Error> {
    compile_for(keys, crate::config::InteractionProfile::NonModal)
}

pub fn compile_for(
    keys: &crate::config::Keymaps,
    profile: crate::config::InteractionProfile,
) -> Result<Effective, Error> {
    compile_ai(keys, profile, Some(&std::collections::BTreeMap::new()))
}

// None is source-local validation: AI references resolve after include layering.
pub(crate) fn compile_ai(
    keys: &crate::config::Keymaps,
    profile: crate::config::InteractionProfile,
    commands: Option<&std::collections::BTreeMap<String, crate::config::AiCommand>>,
) -> Result<Effective, Error> {
    let (editor, minibuffer, org_calendar) = base_maps(keys, commands)?;
    let (org, org_capture, org_source_edit) = org_editing_maps(&editor, keys, commands)?;
    let vim = vim::maps(keys, commands)?;
    let vim_org = vim::org_maps(keys, &vim, profile, commands)?;
    Ok(Effective {
        vim,
        vim_org,
        completion: Map::compile(Context::Completion, &keys.completion, commands)?,
        org,
        org_capture,
        org_source_edit,
        git_commit: Map::overlay(&editor, Context::GitCommit, &keys.git_commit, commands)?,
        editor,
        minibuffer,
        tree: Map::compile(Context::Tree, &keys.tree, commands)?,
        bookmark_list: Map::compile(Context::BookmarkList, &keys.bookmark_list, commands)?,
        git: Map::compile(Context::Git, &keys.git, commands)?,
        org_agenda: Map::compile(Context::OrgAgenda, &keys.org_agenda, commands)?,
        org_calendar,
    })
}

fn base_maps(
    keys: &crate::config::Keymaps,
    commands: Option<&std::collections::BTreeMap<String, crate::config::AiCommand>>,
) -> Result<(Map, Map, Map), Error> {
    let editor = Map::compile(Context::Editor, &keys.editor, commands)?;
    let minibuffer = Map::compile(Context::Minibuffer, &keys.minibuffer, commands)?;
    let org_calendar = Map::inherit(
        &minibuffer,
        Context::OrgCalendar,
        &keys.org_calendar,
        commands,
    )?;
    Ok((editor, minibuffer, org_calendar))
}

fn org_editing_maps(
    editor: &Map,
    keys: &crate::config::Keymaps,
    commands: Option<&std::collections::BTreeMap<String, crate::config::AiCommand>>,
) -> Result<(Map, Map, Map), Error> {
    Ok((
        Map::inherit(editor, Context::Org, &keys.org, commands)?,
        Map::inherit(editor, Context::OrgCapture, &keys.org_capture, commands)?,
        Map::inherit(
            editor,
            Context::OrgSourceEdit,
            &keys.org_source_edit,
            commands,
        )?,
    ))
}
