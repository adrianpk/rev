//! Configuration keymap contracts. Dispatch activation belongs to the focused
//! input owners; parsing uses the same terminal aliases as runtime keys.
mod availability;
mod cancellation;
pub(crate) mod contributions;
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
    MacroDefinition,
    MacroMenu,
    Git,
    GitCommit,
    GitHistory,
    GitInspect,
    GitRefs,
    GitSequence,
    GitWorktree,
    GitStash,
    GitConflict,
    Markdown,
    Org,
    OrgAgenda,
    OrgCapture,
    OrgCalendar,
    OrgSourceEdit,
}

impl Context {
    pub const ALL: [Self; 28] = [
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
        Self::MacroDefinition,
        Self::MacroMenu,
        Self::Git,
        Self::GitCommit,
        Self::GitHistory,
        Self::GitInspect,
        Self::GitRefs,
        Self::GitSequence,
        Self::GitWorktree,
        Self::GitStash,
        Self::GitConflict,
        Self::Markdown,
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
            Self::MacroDefinition => "macro_definition",
            Self::MacroMenu => "macro_menu",
            _ => self.git_name(),
        }
    }

    fn git_name(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::GitCommit => "git_commit",
            Self::GitHistory => "git_history",
            Self::GitInspect => "git_inspect",
            Self::GitRefs => "git_refs",
            Self::GitSequence => "git_sequence",
            Self::GitWorktree => "git_worktree",
            Self::GitStash => "git_stash",
            Self::GitConflict => "git_conflict",
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
            Self::Markdown => "markdown",
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
            Self::MacroDefinition => defaults::macro_definition_complete(),
            Self::GitCommit => defaults::commit_complete(),
            Self::Markdown => defaults::markdown_complete(),
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
    macro_definition: Map,
    macro_menu: Map,
    git: Map,
    git_commit: Map,
    git_history: Map,
    git_inspect: Map,
    git_refs: Map,
    git_sequence: Map,
    git_worktree: Map,
    git_stash: Map,
    git_conflict: Map,
    markdown: Map,
    vim_markdown: Option<Map>,
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
            Context::MacroDefinition => &self.macro_definition,
            Context::MacroMenu => &self.macro_menu,
            _ => self.git_map(context),
        }
    }
    fn git_map(&self, context: Context) -> &Map {
        match context {
            Context::Git => &self.git,
            Context::GitCommit => &self.git_commit,
            Context::GitHistory => &self.git_history,
            Context::GitInspect => &self.git_inspect,
            Context::GitRefs => &self.git_refs,
            Context::GitSequence => &self.git_sequence,
            Context::GitWorktree => &self.git_worktree,
            Context::GitStash => &self.git_stash,
            Context::GitConflict => &self.git_conflict,
            _ => self.org_map(context),
        }
    }
    fn org_map(&self, context: Context) -> &Map {
        match context {
            Context::Markdown => &self.markdown,
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
    let editing = EditingMaps::compile(keys, profile, commands)?;
    let completion = Map::compile(Context::Completion, &keys.completion, commands)?;
    let git_commit = Map::overlay(
        &editing.editor,
        Context::GitCommit,
        &keys.git_commit,
        commands,
    )?;
    let macro_definition = Map::overlay(
        &editing.editor,
        Context::MacroDefinition,
        &keys.macro_definition,
        commands,
    )?;
    let macro_menu = Map::compile(Context::MacroMenu, &keys.macro_menu, commands)?;
    let tree = Map::compile(Context::Tree, &keys.tree, commands)?;
    let bookmark_list = Map::compile(Context::BookmarkList, &keys.bookmark_list, commands)?;
    let repositories = RepositoryMaps::compile(keys, commands)?;
    let org_agenda = Map::compile(Context::OrgAgenda, &keys.org_agenda, commands)?;
    Ok(Effective {
        vim: editing.vim,
        vim_org: editing.vim_org,
        completion,
        markdown: editing.markdown,
        vim_markdown: editing.vim_markdown,
        org: editing.org,
        org_capture: editing.org_capture,
        org_source_edit: editing.org_source_edit,
        git_commit,
        macro_definition,
        macro_menu,
        editor: editing.editor,
        minibuffer: editing.minibuffer,
        tree,
        bookmark_list,
        git: repositories.git,
        git_history: repositories.git_history,
        git_inspect: repositories.git_inspect,
        git_refs: repositories.git_refs,
        git_sequence: repositories.git_sequence,
        git_worktree: repositories.git_worktree,
        git_stash: repositories.git_stash,
        git_conflict: repositories.git_conflict,
        org_agenda,
        org_calendar: editing.org_calendar,
    })
}

/// Inherited editing layers precede every independent native map. Their owned
/// maps are moved into Effective only after the complete candidate is valid.
struct EditingMaps {
    editor: Map,
    minibuffer: Map,
    org_calendar: Map,
    markdown: Map,
    vim_markdown: Option<Map>,
    org: Map,
    org_capture: Map,
    org_source_edit: Map,
    vim: BTreeMap<Context, Map>,
    vim_org: BTreeMap<(Context, Context), Map>,
}

impl EditingMaps {
    fn compile(
        keys: &crate::config::Keymaps,
        profile: crate::config::InteractionProfile,
        commands: Option<&BTreeMap<String, crate::config::AiCommand>>,
    ) -> Result<Self, Error> {
        let (editor, minibuffer, org_calendar) = base_maps(keys, commands)?;
        let (org, org_capture, org_source_edit) = org_editing_maps(&editor, keys, commands)?;
        let vim = vim::maps(keys, commands)?;
        let vim_org = vim::org_maps(keys, &vim, profile, commands)?;
        let markdown = Map::contextual(&editor, Context::Markdown, &keys.markdown, commands)?;
        let vim_markdown = vim::markdown_map(keys, &vim, profile, commands)?;
        Ok(Self {
            editor,
            minibuffer,
            org_calendar,
            markdown,
            vim_markdown,
            org,
            org_capture,
            org_source_edit,
            vim,
            vim_org,
        })
    }
}

/// Git's independent map family follows transient editors, tree and bookmarks
/// in the existing validation order; the commit overlay is compiled earlier.
struct RepositoryMaps {
    git: Map,
    git_history: Map,
    git_inspect: Map,
    git_refs: Map,
    git_sequence: Map,
    git_worktree: Map,
    git_stash: Map,
    git_conflict: Map,
}

impl RepositoryMaps {
    fn compile(
        keys: &crate::config::Keymaps,
        commands: Option<&BTreeMap<String, crate::config::AiCommand>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            git: Map::compile(Context::Git, &keys.git, commands)?,
            git_history: Map::compile(Context::GitHistory, &keys.git_history, commands)?,
            git_inspect: Map::compile(Context::GitInspect, &keys.git_inspect, commands)?,
            git_refs: Map::compile(Context::GitRefs, &keys.git_refs, commands)?,
            git_sequence: Map::compile(Context::GitSequence, &keys.git_sequence, commands)?,
            git_worktree: Map::compile(Context::GitWorktree, &keys.git_worktree, commands)?,
            git_stash: Map::compile(Context::GitStash, &keys.git_stash, commands)?,
            git_conflict: Map::compile(Context::GitConflict, &keys.git_conflict, commands)?,
        })
    }
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
