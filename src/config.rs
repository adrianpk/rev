//! Typed configuration data, independent of files, terminals and editor mutation.
//!
//! `Settings::default()` supplies the complete accepted global defaults without
//! reading a sample. `Document` retains absent leaves until source and language
//! resolution; it must never be deserialized through the complete defaults.
//! Signed input integers retain invalid negative values for later diagnostics.
//! Effective sizes count their documented units, not allocation capacities.
//! Executable startup publishes one admitted snapshot to its existing consumers.

pub(crate) mod capture;
pub use capture::{CaptureKind, CaptureTarget, CaptureTemplate};
mod export;
pub use export::{Export, ExportBackend, PartialExport};
mod agenda;
pub use agenda::{AgendaBlock, AgendaKind, AgendaSort, AgendaView};
mod associations;
mod diagnostic;
mod load;
mod merge;
mod palette;
mod parse;
mod project;
mod publication;
mod resolve;
mod startup;
mod template;
mod validate;
mod xdg;

pub use diagnostic::{Diagnostic, Position};
pub use load::{Source, Sources, load};
pub use palette::{
    GutterPalette, OrgPalette, Palette, PartialGutterPalette, PartialOrgPalette, PartialPalette,
    PartialSyntaxPalette, Rgb, SyntaxPalette,
};
pub use parse::parse;
pub use resolve::{Snapshot, resolve};
pub use startup::{Startup, prepare};
pub use template::default_template;
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
    pub gutter: crate::gutter::Settings,
    pub mouse: Mouse,
    pub minibuffer: Minibuffer,
    pub history: History,
    pub kill_ring: KillRing,
    pub tree: Tree,
    pub syntax: Syntax,
    pub org: Org,
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
    pub gutter: PartialGutter,
    pub mouse: PartialMouse,
    pub minibuffer: PartialMinibuffer,
    pub history: PartialHistory,
    pub kill_ring: PartialKillRing,
    pub tree: PartialTree,
    pub syntax: PartialSyntax,
    pub org: PartialOrg,
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

/// Native Org availability; individual buffers retain explicit mode choices.
#[derive(Clone, Debug, PartialEq)]
pub struct Org {
    pub enabled: bool,
    pub properties: Properties,
    pub tags: Tags,
    pub todo: Todo,
    pub priority: Priority,
    pub duration: Duration,
    pub time: Time,
    pub clock: Clock,
    pub timer: Timer,
    pub habit: Habit,
    pub agenda: Agenda,
    pub capture: Capture,
    pub refile: Refile,
    pub archive: Archive,
    pub attachments: Attachments,
    pub tools: Tools,
    pub tables: Tables,
    pub export: Export,
    pub limits: OrgLimits,
}

impl Default for Org {
    fn default() -> Self {
        Self {
            enabled: true,
            properties: Properties::default(),
            tags: Tags::default(),
            todo: Todo::default(),
            priority: Priority::default(),
            duration: Duration::default(),
            time: Time::default(),
            clock: Clock::default(),
            timer: Timer::default(),
            habit: Habit::default(),
            agenda: Agenda::default(),
            capture: Capture::default(),
            refile: Refile::default(),
            archive: Archive::default(),
            attachments: Attachments::default(),
            tools: Tools::default(),
            tables: Tables::default(),
            export: Export::default(),
            limits: OrgLimits::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialOrg {
    pub enabled: Option<bool>,
    pub properties: PartialProperties,
    pub tags: PartialTags,
    pub todo: PartialTodo,
    pub priority: PartialPriority,
    pub duration: PartialDuration,
    pub time: PartialTime,
    pub clock: PartialClock,
    pub timer: PartialTimer,
    pub habit: PartialHabit,
    pub agenda: PartialAgenda,
    pub capture: PartialCapture,
    pub refile: PartialRefile,
    pub archive: PartialArchive,
    pub attachments: PartialAttachments,
    pub tools: PartialTools,
    pub tables: PartialTables,
    pub export: PartialExport,
    pub limits: PartialOrgLimits,
}

/// Effective Org operation zone; fold/gap policies require explicit selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Time {
    pub zone: String,
    pub extend_today_until: usize,
    pub ambiguous: LocalTimePolicy,
    pub nonexistent: LocalTimePolicy,
}
impl Default for Time {
    fn default() -> Self {
        Self {
            zone: "local".into(),
            extend_today_until: 0,
            ambiguous: LocalTimePolicy::Prompt,
            nonexistent: LocalTimePolicy::Prompt,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalTimePolicy {
    Prompt,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTime {
    pub zone: Option<String>,
    pub extend_today_until: Option<i64>,
    pub ambiguous: Option<LocalTimePolicy>,
    pub nonexistent: Option<LocalTimePolicy>,
}

/// Ordered native priority characters, with implicit default for absent cookies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Priority {
    pub highest: String,
    pub lowest: String,
    pub default: String,
}
impl Default for Priority {
    fn default() -> Self {
        Self {
            highest: "A".into(),
            lowest: "C".into(),
            default: "B".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialPriority {
    pub highest: Option<String>,
    pub lowest: Option<String>,
    pub default: Option<String>,
}
/// Duration unit scales are elapsed work minutes, not calendar arithmetic.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Duration {
    pub units: DurationUnits,
}
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialDuration {
    pub units: PartialDurationUnits,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DurationUnits {
    pub min: f64,
    pub h: f64,
    pub d: f64,
    pub w: f64,
    pub m: f64,
    pub y: f64,
}
impl Default for DurationUnits {
    fn default() -> Self {
        Self {
            min: 1.0,
            h: 60.0,
            d: 1440.0,
            w: 10080.0,
            m: 43200.0,
            y: 525960.0,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialDurationUnits {
    pub min: Option<f64>,
    pub h: Option<f64>,
    pub d: Option<f64>,
    pub w: Option<f64>,
    pub m: Option<f64>,
    pub y: Option<f64>,
}

/// Native task defaults; source declarations override only the workflow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepeatLogging {
    None,
    #[default]
    Time,
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Todo {
    pub sequence: Vec<String>,
    pub finished: Vec<String>,
    pub log_done: bool,
    pub log_repeat: RepeatLogging,
    pub log_drawer: String,
    pub block_children: bool,
}
impl Default for Todo {
    fn default() -> Self {
        Self {
            sequence: vec!["TODO".into(), "DONE".into()],
            finished: vec!["DONE".into()],
            log_done: false,
            log_repeat: RepeatLogging::Time,
            log_drawer: String::new(),
            block_children: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTodo {
    pub sequence: Option<Vec<String>>,
    pub finished: Option<Vec<String>>,
    pub log_done: Option<bool>,
    pub log_repeat: Option<RepeatLogging>,
    pub log_drawer: Option<String>,
    pub block_children: Option<bool>,
}

/// Effective tags include ancestors and FILETAGS by factory policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tags {
    pub inherit: bool,
}
impl Default for Tags {
    fn default() -> Self {
        Self { inherit: true }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTags {
    pub inherit: Option<bool>,
}

/// Selective, case-insensitive source-property inheritance; no general inheritance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Properties {
    pub inherit: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialProperties {
    pub inherit: Option<Vec<String>>,
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
    pub org: BTreeMap<String, String>,
    pub org_agenda: BTreeMap<String, String>,
    pub org_capture: BTreeMap<String, String>,
    pub org_calendar: BTreeMap<String, String>,
    pub org_source_edit: BTreeMap<String, String>,
}

impl Keymaps {
    pub fn map(&self, context: crate::input::keymap::Context) -> &BTreeMap<String, String> {
        use crate::input::keymap::Context;
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

    pub(super) fn map_mut(
        &mut self,
        context: crate::input::keymap::Context,
    ) -> &mut BTreeMap<String, String> {
        use crate::input::keymap::Context;
        match context {
            Context::Editor => &mut self.editor,
            Context::Minibuffer => &mut self.minibuffer,
            Context::Tree => &mut self.tree,
            Context::Org => &mut self.org,
            Context::OrgAgenda => &mut self.org_agenda,
            Context::OrgCapture => &mut self.org_capture,
            Context::OrgCalendar => &mut self.org_calendar,
            Context::OrgSourceEdit => &mut self.org_source_edit,
        }
    }
}

/// Optional gutter leaves retain absence through include/root layering. Bounds
/// are validated before constructing the native view format.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialGutter {
    pub visible: Option<bool>,
    pub mode: Option<crate::gutter::Mode>,
    pub current_line: Option<crate::gutter::CurrentLine>,
    pub side: Option<crate::gutter::Side>,
    pub minimum_digits: Option<i64>,
    pub left_padding: Option<i64>,
    pub right_padding: Option<i64>,
}

/// Session-wide native pointer preference and bounded logical-row distance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mouse {
    pub enabled: bool,
    pub scroll_lines: usize,
}

impl Default for Mouse {
    fn default() -> Self {
        Self {
            enabled: true,
            scroll_lines: 3,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialMouse {
    pub enabled: Option<bool>,
    pub scroll_lines: Option<i64>,
}

/// Canonical clock source placement and zero-minute retention.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clock {
    pub persist: bool,
    pub history_limit: usize,
    pub out_when_done: bool,
    pub drawer: String,
    pub remove_zero: bool,
}
impl Default for Clock {
    fn default() -> Self {
        Self {
            persist: false,
            history_limit: 32,
            out_when_done: true,
            drawer: "LOGBOOK".into(),
            remove_zero: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialClock {
    pub persist: Option<bool>,
    pub history_limit: Option<i64>,
    pub out_when_done: Option<bool>,
    pub drawer: Option<String>,
    pub remove_zero: Option<bool>,
}

/// Explicit countdown default; constructing settings never arms a timer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timer {
    pub default: String,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            default: "0".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTimer {
    pub default: Option<String>,
}

/// Bounded native habit graph preferences; source history remains canonical.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Habit {
    pub preceding_days: usize,
    pub following_days: usize,
    pub graph_column: usize,
    pub today_only: bool,
    pub show_all_today: bool,
    pub done_always_green: bool,
}
impl Default for Habit {
    fn default() -> Self {
        Self {
            preceding_days: 21,
            following_days: 7,
            graph_column: 40,
            today_only: true,
            show_all_today: false,
            done_always_green: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialHabit {
    pub preceding_days: Option<i64>,
    pub following_days: Option<i64>,
    pub graph_column: Option<i64>,
    pub today_only: Option<bool>,
    pub show_all_today: Option<bool>,
    pub done_always_green: Option<bool>,
}

/// Explicit agenda roots. Empty roots never imply a filesystem search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agenda {
    pub sources: Vec<String>,
    pub span: AgendaSpan,
    pub week_start: usize,
    pub views: Vec<AgendaView>,
    pub include_archived: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgendaSpan {
    Day,
    #[default]
    Week,
    Month,
    Year,
    Days(u16),
}
impl Default for Agenda {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            span: AgendaSpan::Week,
            week_start: 1,
            views: Vec::new(),
            include_archived: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialAgenda {
    pub sources: Option<Vec<String>>,
    pub span: Option<AgendaSpan>,
    pub week_start: Option<i64>,
    pub views: Option<Vec<AgendaView>>,
    pub include_archived: Option<bool>,
}
/// Finite agenda discovery, numerical work and IO budgets. Other Org limits remain with their
/// existing owners until their typed operations are delivered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrgLimits {
    pub export_bytes: usize,
    pub expansion_bytes: usize,
    pub include_files: usize,
    pub include_depth: usize,
    pub formula_cells: usize,
    pub formula_steps: usize,
    pub formula_depth: usize,
    pub formula_decimal_places: usize,
    pub agenda_files: usize,
    pub agenda_rows: usize,
    pub query_bytes: usize,
    pub query_depth: usize,
    pub agenda_file_bytes: usize,
    pub agenda_total_bytes: usize,
    pub job_seconds: usize,
    pub job_output_bytes: usize,
}
impl Default for OrgLimits {
    fn default() -> Self {
        Self {
            export_bytes: 16777216,
            expansion_bytes: 1048576,
            include_files: 128,
            include_depth: 16,
            formula_cells: 100000,
            formula_steps: 1000000,
            formula_depth: 64,
            formula_decimal_places: 12,
            agenda_files: 512,
            agenda_rows: 20000,
            query_bytes: 4096,
            query_depth: 32,
            agenda_file_bytes: 8388608,
            agenda_total_bytes: 67108864,
            job_seconds: 30,
            job_output_bytes: 1048576,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialOrgLimits {
    pub export_bytes: Option<i64>,
    pub expansion_bytes: Option<i64>,
    pub include_files: Option<i64>,
    pub include_depth: Option<i64>,
    pub formula_cells: Option<i64>,
    pub formula_steps: Option<i64>,
    pub formula_depth: Option<i64>,
    pub formula_decimal_places: Option<i64>,
    pub agenda_files: Option<i64>,
    pub agenda_rows: Option<i64>,
    pub query_bytes: Option<i64>,
    pub query_depth: Option<i64>,
    pub agenda_file_bytes: Option<i64>,
    pub agenda_total_bytes: Option<i64>,
    pub job_seconds: Option<i64>,
    pub job_output_bytes: Option<i64>,
}

/// Finite binary64 output precision. Empty input and half-away rounding follow
/// the fixed native profile; only supported numeric choices are configurable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tables {
    pub significant_digits: usize,
}
impl Default for Tables {
    fn default() -> Self {
        Self {
            significant_digits: 12,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTables {
    pub significant_digits: Option<i64>,
}

/// Capture arrays replace by leaf; empty arrays select the native Task fallback.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Capture {
    pub templates: Vec<CaptureTemplate>,
    pub default_file: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialCapture {
    pub templates: Option<Vec<CaptureTemplate>>,
    pub default_file: Option<String>,
}

/// Explicit local refile roots. Empty uses admitted agenda/source headings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Refile {
    pub targets: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialRefile {
    pub targets: Option<Vec<String>>,
}

/// Literal native Org archive destination, expanded relative to the source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Archive {
    pub location: String,
}
impl Default for Archive {
    fn default() -> Self {
        Self {
            location: "%s_archive::".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialArchive {
    pub location: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachments {
    pub directory: String,
}
impl Default for Attachments {
    fn default() -> Self {
        Self {
            directory: "data".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialAttachments {
    pub directory: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tools {
    pub viewer: Vec<String>,
    pub pdf: Vec<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PartialTools {
    pub viewer: Option<Vec<String>>,
    pub pdf: Option<Vec<String>>,
}
