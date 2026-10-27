//! Typed configuration data, independent of files, terminals and editor mutation.
//!
//! `Settings::default()` supplies the complete accepted global defaults without
//! reading a sample. `Document` retains absent leaves until source and language
//! resolution; it must never be deserialized through the complete defaults.
//! Signed input integers retain invalid negative values for later diagnostics.
//! Effective sizes count their documented units, not allocation capacities.
//! These records are preparatory: startup loading and consumers are separate.

mod diagnostic;
mod load;
mod merge;
mod palette;
mod parse;
mod project;
mod resolve;
mod validate;
mod xdg;

pub use diagnostic::{Diagnostic, Position};
pub use load::{Source, Sources, load};
pub use palette::{Palette, PartialPalette, PartialSyntaxPalette, Rgb, SyntaxPalette};
pub use parse::parse;
pub use resolve::{Snapshot, resolve};
pub use xdg::Roots;

use std::collections::BTreeMap;

use serde::Deserialize;

/// Complete global defaults plus optional language and user mapping overrides.
/// Built-in file recognition and context bindings remain owned by their dispatchers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub editor: Editor,
    pub scroll: Scroll,
    pub windows: Windows,
    pub minibuffer: Minibuffer,
    pub history: History,
    pub kill_ring: KillRing,
    pub tree: Tree,
    pub syntax: Syntax,
    pub files: Files,
    pub language: BTreeMap<String, Language>,
    pub keys: Keymaps,
    pub palette: Palette,
}

/// One source's data, before validation or inheritance. Empty tables do not
/// reset siblings; absent leaves differ from false, zero and empty collections.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Document {
    /// Absent means schema 1; explicit versions require validation.
    pub version: Option<i64>,
    /// Source-local traversal order; None differs from an explicit empty array.
    pub include: Option<Vec<String>>,
    pub editor: PartialEditor,
    pub scroll: PartialScroll,
    pub windows: PartialWindows,
    pub minibuffer: PartialMinibuffer,
    pub history: PartialHistory,
    pub kill_ring: PartialKillRing,
    pub tree: PartialTree,
    pub syntax: PartialSyntax,
    pub files: PartialFiles,
    pub language: BTreeMap<String, Language>,
    pub keys: Keymaps,
    pub palette: PartialPalette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Match {
    PrefixThenSubstring,
    Prefix,
    Substring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TreeRoot {
    Auto,
    Cwd,
    FileParent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Newline {
    Lf,
    Crlf,
}

/// Only the existing bundled Rust grammar or plain text may be selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Grammar {
    Rust,
    Plain,
}

/// Buffer policy: widths are 1..=32 display columns/cells, respectively.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    pub indent_style: IndentStyle,
    pub indent_width: usize,
    pub tab_width: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces,
            indent_width: 4,
            tab_width: 4,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialEditor {
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<i64>,
    pub tab_width: Option<i64>,
}

/// Only these four leaves are permitted in a language table. Missing editing
/// leaves inherit globals; grammar defaults to Rust for rust, plain otherwise.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Language {
    pub indent_style: Option<IndentStyle>,
    pub indent_width: Option<i64>,
    pub tab_width: Option<i64>,
    pub grammar: Option<Grammar>,
}

/// Page overlap is 0..=1000 rows, constrained by geometry to permit progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scroll {
    pub page_overlap: usize,
}

impl Default for Scroll {
    fn default() -> Self {
        Self { page_overlap: 2 }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialScroll {
    pub page_overlap: Option<i64>,
}

/// Initial split share: finite 0.1..=0.9, constrained by layout minima.
#[derive(Clone, Debug, PartialEq)]
pub struct Windows {
    pub split_ratio: f64,
}

impl Default for Windows {
    fn default() -> Self {
        Self { split_ratio: 0.5 }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialWindows {
    pub split_ratio: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Minibuffer {
    /// 1..=32 candidate rows, clipped to available geometry.
    pub candidate_limit: usize,
    pub r#match: Match,
    /// 1..=4096 accepted command entries in session memory.
    pub history_limit: usize,
    /// 1..=65536 Unicode scalars, with visible excess-input rejection.
    pub input_limit: usize,
}

impl Default for Minibuffer {
    fn default() -> Self {
        Self {
            candidate_limit: 8,
            r#match: Match::PrefixThenSubstring,
            history_limit: 128,
            input_limit: 4096,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialMinibuffer {
    pub candidate_limit: Option<i64>,
    pub r#match: Option<Match>,
    pub history_limit: Option<i64>,
    pub input_limit: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    /// 1..=65536 complete groups; never split an operation.
    pub group_limit: usize,
    /// 1..=1073741824 logical payload bytes; preserve the newest coherent group
    /// even when it alone exceeds this soft target.
    pub payload_target_bytes: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            group_limit: 512,
            payload_target_bytes: 67_108_864,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialHistory {
    pub group_limit: Option<i64>,
    pub payload_target_bytes: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KillRing {
    /// 1..=4096 linear entries; rectangular text has a separate owner.
    pub capacity: usize,
    pub coalesce: bool,
}

impl Default for KillRing {
    fn default() -> Self {
        Self {
            capacity: 32,
            coalesce: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialKillRing {
    pub capacity: Option<i64>,
    pub coalesce: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tree {
    pub visible: bool,
    /// 1..=512 preferred columns, constrained by available geometry.
    pub width: usize,
    /// Finite 0.1..=0.5 maximum frame share; small-frame hiding still applies.
    pub max_fraction: f64,
    pub root: TreeRoot,
    pub show_hidden: bool,
    pub directories_first: bool,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            visible: false,
            width: 24,
            max_fraction: 0.3333333333333333,
            root: TreeRoot::Auto,
            show_hidden: true,
            directories_first: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTree {
    pub visible: Option<bool>,
    pub width: Option<i64>,
    pub max_fraction: Option<f64>,
    pub root: Option<TreeRoot>,
    pub show_hidden: Option<bool>,
    pub directories_first: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Syntax {
    pub enabled: bool,
    /// 0..=1073741824 UTF-8 bytes; zero means no configured syntax cutoff.
    pub max_bytes: usize,
}

impl Default for Syntax {
    fn default() -> Self {
        Self {
            enabled: true,
            max_bytes: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialSyntax {
    pub enabled: Option<bool>,
    pub max_bytes: Option<i64>,
}

/// Format defaults apply only to new buffers; associations are user overrides
/// of the built-in basename/suffix map, not grammar installation requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Files {
    pub newline: Newline,
    pub bom: bool,
    pub associations: BTreeMap<String, String>,
}

impl Default for Files {
    fn default() -> Self {
        Self {
            newline: Newline::Lf,
            bom: false,
            associations: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialFiles {
    pub newline: Option<Newline>,
    pub bom: Option<bool>,
    pub associations: BTreeMap<String, String>,
}

/// User sequence-to-command overrides only. Empty maps inherit built-in context
/// dispatch; values require later validation against fixed command identities.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Keymaps {
    pub editor: BTreeMap<String, String>,
    pub minibuffer: BTreeMap<String, String>,
    pub tree: BTreeMap<String, String>,
}
