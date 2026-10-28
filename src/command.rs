mod binding;
mod buffer;
mod gutter;
mod kill;
mod linear;
mod motion;
mod org;
pub use org::OrgCommand;
mod rectangle;
mod region;
mod scroll;
mod split;
mod tree;
mod window;

pub use crate::buffer::edit::History;
pub use crate::buffer::{Mode, Whitespace};
pub use crate::window::Split;
pub use binding::{Binding, PromptAction};
pub use buffer::BufferCommand;
pub use gutter::GutterCommand;
pub use kill::KillCommand;
pub use linear::LinearCommand;
pub use motion::Motion;
pub use rectangle::RectangleCommand;
pub use region::RegionCommand;
pub use scroll::Scroll;
pub use tree::TreeCommand;
pub use window::WindowCommand;

/// Only commands with a working handler belong in the public registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Tree(TreeCommand),
    QueryReplace,
    SearchForward,
    SearchBackward,
    ExecuteExtended,
    Quit,
    FindFile,
    GotoLine,
    MouseMode,
    Mode(Mode),
    Org(OrgCommand),
    Gutter(GutterCommand),
    Whitespace(Whitespace),
    SaveBuffer,
    WriteFile,
    KeyboardQuit,
    Newline,
    InsertTab,
    BackwardDeleteChar,
    DeleteChar,
    Motion(Motion),
    Scroll(Scroll),
    History(History),
    Buffer(BufferCommand),
    Split(Split),
    Window(WindowCommand),
    Region(RegionCommand),
    Rectangle(RectangleCommand),
    Kill(KillCommand),
    Linear(LinearCommand),
}

impl Command {
    pub const ALL: [Self; 120] = [
        Self::Rectangle(RectangleCommand::Mark),
        Self::Rectangle(RectangleCommand::Copy),
        Self::Rectangle(RectangleCommand::Kill),
        Self::Rectangle(RectangleCommand::Yank),
        Self::Tree(TreeCommand::Delete),
        Self::Tree(TreeCommand::Rename),
        Self::Tree(TreeCommand::Open),
        Self::Tree(TreeCommand::CreateFile),
        Self::Tree(TreeCommand::CreateDirectory),
        Self::Tree(TreeCommand::Toggle),
        Self::Tree(TreeCommand::Focus),
        Self::Tree(TreeCommand::Previous),
        Self::Tree(TreeCommand::Next),
        Self::Tree(TreeCommand::Expand),
        Self::Tree(TreeCommand::Collapse),
        Self::Tree(TreeCommand::Activate),
        Self::Tree(TreeCommand::Refresh),
        Self::QueryReplace,
        Self::SearchForward,
        Self::SearchBackward,
        Self::ExecuteExtended,
        Self::Quit,
        Self::FindFile,
        Self::GotoLine,
        Self::MouseMode,
        Self::Org(OrgCommand::Cycle),
        Self::Org(OrgCommand::GlobalCycle),
        Self::Org(OrgCommand::Overview),
        Self::Org(OrgCommand::Contents),
        Self::Org(OrgCommand::ShowAll),
        Self::Org(OrgCommand::HideSubtree),
        Self::Org(OrgCommand::ShowChildren),
        Self::Org(OrgCommand::ShowSubtree),
        Self::Org(OrgCommand::HideDrawer),
        Self::Org(OrgCommand::ShowDrawer),
        Self::Org(OrgCommand::ToggleDrawer),
        Self::Org(OrgCommand::InsertHeading),
        Self::Org(OrgCommand::NextHeading),
        Self::Org(OrgCommand::PreviousHeading),
        Self::Org(OrgCommand::ForwardHeading),
        Self::Org(OrgCommand::BackwardHeading),
        Self::Org(OrgCommand::ParentHeading),
        Self::Org(OrgCommand::MetaReturn),
        Self::Org(OrgCommand::PromoteSubtree),
        Self::Org(OrgCommand::DemoteSubtree),
        Self::Org(OrgCommand::MoveSubtreeUp),
        Self::Org(OrgCommand::MoveSubtreeDown),
        Self::Org(OrgCommand::MetaLeft),
        Self::Org(OrgCommand::MetaRight),
        Self::Org(OrgCommand::MetaUp),
        Self::Org(OrgCommand::MetaDown),
        Self::Org(OrgCommand::InsertItem),
        Self::Org(OrgCommand::IndentItem),
        Self::Org(OrgCommand::OutdentItem),
        Self::Org(OrgCommand::IndentItemTree),
        Self::Org(OrgCommand::OutdentItemTree),
        Self::Org(OrgCommand::MoveItemUp),
        Self::Org(OrgCommand::MoveItemDown),
        Self::Org(OrgCommand::RenumberList),
        Self::Org(OrgCommand::CtrlCCtrlC),
        Self::Org(OrgCommand::ToggleCheckbox),
        Self::Org(OrgCommand::ToggleRadioButton),
        Self::Org(OrgCommand::ToggleCheckboxPresence),
        Self::Org(OrgCommand::UpdateStatisticsCookies),
        Self::Org(OrgCommand::InsertCheckboxItem),
        Self::Org(OrgCommand::SetProperty),
        Self::Org(OrgCommand::DeleteProperty),
        Self::Org(OrgCommand::InspectProperty),
        Self::Org(OrgCommand::EditPropertyOwner),
        Self::Mode(Mode::Org),
        Self::Mode(Mode::Text),
        Self::Gutter(GutterCommand::Toggle),
        Self::Gutter(GutterCommand::CycleMode),
        Self::Gutter(GutterCommand::Absolute),
        Self::Gutter(GutterCommand::Relative),
        Self::Gutter(GutterCommand::CycleCurrentLine),
        Self::Gutter(GutterCommand::ToggleSide),
        Self::Gutter(GutterCommand::Inspect),
        Self::Gutter(GutterCommand::Reset),
        Self::Whitespace(Whitespace::Tabify),
        Self::Whitespace(Whitespace::Untabify),
        Self::SaveBuffer,
        Self::WriteFile,
        Self::KeyboardQuit,
        Self::Newline,
        Self::InsertTab,
        Self::BackwardDeleteChar,
        Self::DeleteChar,
        Self::Motion(Motion::ForwardChar),
        Self::Motion(Motion::BackwardChar),
        Self::Motion(Motion::LineStart),
        Self::Motion(Motion::LineEnd),
        Self::Motion(Motion::BufferStart),
        Self::Motion(Motion::BufferEnd),
        Self::Motion(Motion::ForwardWord),
        Self::Motion(Motion::BackwardWord),
        Self::Scroll(Scroll::NextLine),
        Self::Scroll(Scroll::PreviousLine),
        Self::Scroll(Scroll::PageForward),
        Self::Scroll(Scroll::PageBackward),
        Self::Scroll(Scroll::Recenter),
        Self::History(History::Undo),
        Self::History(History::Redo),
        Self::Buffer(BufferCommand::Switch),
        Self::Buffer(BufferCommand::List),
        Self::Buffer(BufferCommand::Kill),
        Self::Split(Split::Below),
        Self::Split(Split::Right),
        Self::Window(WindowCommand::Delete),
        Self::Window(WindowCommand::Keep),
        Self::Window(WindowCommand::Other),
        Self::Region(RegionCommand::Set),
        Self::Region(RegionCommand::Exchange),
        Self::Kill(KillCommand::Line),
        Self::Kill(KillCommand::Word),
        Self::Kill(KillCommand::BackwardWord),
        Self::Linear(LinearCommand::Kill),
        Self::Linear(LinearCommand::Copy),
        Self::Linear(LinearCommand::Yank),
        Self::Linear(LinearCommand::Pop),
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Motion(motion) => motion.name(),
            Self::Scroll(scroll) => scroll.name(),
            Self::History(history) => history.name(),
            Self::Buffer(command) => command.name(),
            Self::Split(split) => split.name(),
            Self::Window(command) => command.name(),
            Self::Kill(command) => command.name(),
            Self::Linear(command) => command.name(),
            Self::Whitespace(command) => command.name(),
            _ => self.action_name(),
        }
    }

    fn action_name(self) -> &'static str {
        match self {
            Self::Rectangle(command) => command.name(),
            Self::Gutter(command) => command.name(),
            Self::ExecuteExtended => "execute-extended-command",
            Self::Quit => "save-buffers-kill-terminal",
            Self::FindFile => "find-file",
            Self::GotoLine => "goto-line",
            Self::SaveBuffer => "save-buffer",
            Self::WriteFile => "write-file",
            Self::KeyboardQuit => "keyboard-quit",
            _ => self.search_name(),
        }
    }

    fn search_name(self) -> &'static str {
        match self {
            Self::Region(command) => command.name(),
            Self::Tree(command) => command.name(),
            Self::MouseMode => "mouse-mode",
            Self::QueryReplace => "query-replace",
            Self::SearchForward => "isearch-forward",
            Self::SearchBackward => "isearch-backward",
            _ => self.edit_name(),
        }
    }

    fn edit_name(self) -> &'static str {
        match self {
            Self::Mode(mode) => mode.name(),
            Self::Org(command) => command.name(),
            Self::Newline => "newline",
            Self::InsertTab => "insert-tab",
            Self::BackwardDeleteChar => "backward-delete-char",
            Self::DeleteChar => "delete-char",
            _ => unreachable!("non-edit commands delegate their names before edit lookup"),
        }
    }

    pub fn named(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }

    /// Fixed-name matches: prefixes first, then substrings, lexical within each.
    pub fn matching(query: &str) -> Vec<Self> {
        let mut commands: Vec<_> = Self::ALL
            .into_iter()
            .filter(|command| command.name().contains(query))
            .collect();
        commands.sort_by_key(|command| (!command.name().starts_with(query), command.name()));
        commands
    }

    pub(crate) fn is_kill(self) -> bool {
        matches!(self, Self::Kill(_) | Self::Linear(LinearCommand::Kill))
    }
}

impl Command {
    /// Repetition belongs to the resolved action, independent of its chord.
    /// Prefix completions and session/prompt transitions require fresh presses.
    pub(crate) fn repeatable(self) -> bool {
        matches!(
            self,
            Self::Newline
                | Self::InsertTab
                | Self::BackwardDeleteChar
                | Self::DeleteChar
                | Self::Motion(_)
                | Self::Scroll(_)
                | Self::History(_)
                | Self::Region(_)
                | Self::Kill(_)
                | Self::Linear(_)
                | Self::SearchForward
                | Self::SearchBackward
        )
    }
}
