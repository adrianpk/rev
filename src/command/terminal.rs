#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCommand {
    Yank,
    YankPop,
    Clear,
    ClearScrollback,
    BeginningOfLine,
    EndOfLine,
    ResetCursorPoint,
    Undo,
    Send(TerminalKey),
    Open,
    Toggle,
    NewBuffer,
    MoveToBuffer,
    MoveToPanel,
    New,
    List,
    Rename,
    OpenHere,
    OpenOtherPane,
    SendNextKey,
    CopyMode,
    CopyModeDone,
}

impl TerminalCommand {
    pub fn name(self) -> &'static str {
        match self {
            Self::CopyMode => return "terminal-copy-mode",
            Self::CopyModeDone => return "terminal-copy-mode-done",
            _ => {}
        }
        if self == Self::SendNextKey {
            return "terminal-send-next-key";
        }
        self.transfer_name()
            .unwrap_or_else(|| self.placement_name())
    }

    fn transfer_name(self) -> Option<&'static str> {
        Some(match self {
            Self::Yank => "terminal-yank",
            Self::YankPop => "terminal-yank-pop",
            Self::Clear => "terminal-clear",
            Self::ClearScrollback => "terminal-clear-scrollback",
            Self::BeginningOfLine => "terminal-beginning-of-line",
            Self::EndOfLine => "terminal-end-of-line",
            Self::ResetCursorPoint => "terminal-reset-cursor-point",
            Self::Undo => "terminal-undo",
            Self::Send(key) => key.name(),
            _ => return None,
        })
    }

    fn placement_name(self) -> &'static str {
        match self {
            Self::Open => "terminal-open",
            Self::Toggle => "terminal-toggle",
            Self::NewBuffer => "terminal-new-buffer",
            Self::MoveToBuffer => "terminal-move-to-buffer",
            Self::MoveToPanel => "terminal-move-to-panel",
            Self::New => "terminal-new",
            Self::List => "terminal-list",
            Self::Rename => "terminal-rename",
            _ => self.entry_name(),
        }
    }

    fn entry_name(self) -> &'static str {
        match self {
            Self::OpenHere => "terminal-open-here",
            Self::OpenOtherPane => "terminal-open-other-pane",
            _ => unreachable!("directory entry commands only"),
        }
    }
}

/// The complete admitted explicit-send inventory uses the live VT key encoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalKey {
    Return,
    Tab,
    Escape,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Prior,
    Next,
    Space,
}
impl TerminalKey {
    pub fn name(self) -> &'static str {
        match self {
            Self::Return => "terminal-send-return",
            Self::Tab => "terminal-send-tab",
            Self::Escape => "terminal-send-escape",
            Self::Backspace => "terminal-send-backspace",
            Self::Delete => "terminal-send-delete",
            _ => self.movement_name(),
        }
    }
    fn movement_name(self) -> &'static str {
        match self {
            Self::Up => "terminal-send-up",
            Self::Down => "terminal-send-down",
            Self::Left => "terminal-send-left",
            Self::Right => "terminal-send-right",
            Self::Prior => "terminal-send-prior",
            Self::Next => "terminal-send-next",
            Self::Space => "terminal-send-space",
            _ => unreachable!("movement/space send names"),
        }
    }
    pub(crate) fn code(self) -> crossterm::event::KeyCode {
        use crossterm::event::KeyCode;
        match self {
            Self::Return => KeyCode::Enter,
            Self::Tab => KeyCode::Tab,
            Self::Escape => KeyCode::Esc,
            Self::Backspace => KeyCode::Backspace,
            Self::Delete => KeyCode::Delete,
            _ => self.movement_code(),
        }
    }
    fn movement_code(self) -> crossterm::event::KeyCode {
        use crossterm::event::KeyCode;
        match self {
            Self::Up => KeyCode::Up,
            Self::Down => KeyCode::Down,
            Self::Left => KeyCode::Left,
            Self::Right => KeyCode::Right,
            Self::Prior => KeyCode::PageUp,
            Self::Next => KeyCode::PageDown,
            Self::Space => KeyCode::Char(' '),
            _ => unreachable!("movement/space send keys"),
        }
    }
}
