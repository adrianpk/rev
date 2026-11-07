#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCommand {
    NextPrompt,
    PreviousPrompt,
    NextError,
    OpenLink,
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
            Self::NextPrompt => return "terminal-next-prompt",
            Self::PreviousPrompt => return "terminal-previous-prompt",
            Self::NextError => return "terminal-next-error",
            Self::OpenLink => return "terminal-open-link",
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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::NextPrompt => return "Navigate the next captured OSC 133 prompt.",
            Self::PreviousPrompt => return "Navigate the previous captured OSC 133 prompt.",
            Self::NextError => return "Visit an actually recognized retained error location.",
            Self::OpenLink => {
                return "Explicitly activate the validated link at live or frozen point.";
            }
            Self::CopyMode => {
                return "Freeze retained output for native navigation, selection and search.";
            }
            Self::CopyModeDone => return "Resume live input without sending Return to the child.",
            _ => {}
        }
        if self == Self::SendNextKey {
            return "Send the next key directly to the terminal process.";
        }
        self.transfer_description()
            .unwrap_or_else(|| self.placement_description())
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

    fn transfer_description(self) -> Option<&'static str> {
        Some(match self {
            Self::Yank => "Paste the newest retained text using active bracket framing.",
            Self::YankPop => "Rotate only while the preceding paste remains wholly queued.",
            Self::Clear => "Send C-l through ordinary child input.",
            Self::ClearScrollback => {
                "Clear retained history and copy projections, preserving live screen/input."
            }
            Self::BeginningOfLine => "Move to the frozen copy line start.",
            Self::EndOfLine => "Move to the frozen copy line end.",
            Self::ResetCursorPoint => "Align copy point with a still-represented live cursor.",
            Self::Undo => "Send C-_ to the child without source undo.",
            Self::Send(key) => key.description(),
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

    fn placement_description(self) -> &'static str {
        match self {
            Self::Open => {
                "Open or select the instance default interactive terminal in the lower panel."
            }
            Self::Toggle => {
                "Hide or restore the lower terminal panel without restarting the child."
            }
            Self::NewBuffer => {
                "Create an independent terminal directly in the selected editing pane."
            }
            Self::MoveToBuffer => "Move the current terminal into the selected editing pane.",
            Self::MoveToPanel => "Dock the same session in the lower panel.",
            Self::New => "Create an independent lower-panel session.",
            Self::List => "Select or inspect retained terminal sessions.",
            Self::Rename => {
                "Give the selected terminal buffer a unique name without changing its identity."
            }
            _ => self.entry_description(),
        }
    }

    fn entry_name(self) -> &'static str {
        match self {
            Self::OpenHere => "terminal-open-here",
            Self::OpenOtherPane => "terminal-open-other-pane",
            _ => unreachable!("directory entry commands only"),
        }
    }

    fn entry_description(self) -> &'static str {
        match self {
            Self::OpenHere => "Create a session from the current resource's directory context.",
            Self::OpenOtherPane => {
                "Select a terminal in the next pane, creating a right split when needed."
            }
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

    pub(crate) fn description(self) -> &'static str {
        match self {
            Self::Return => "Send Return with the live mode-aware VT encoder.",
            Self::Tab => "Send Tab with the live mode-aware VT encoder.",
            Self::Escape => "Send Escape with the live mode-aware VT encoder.",
            Self::Backspace => "Send Backspace with the live mode-aware VT encoder.",
            Self::Delete => "Send Delete with the live mode-aware VT encoder.",
            _ => self.movement_description(),
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

    fn movement_description(self) -> &'static str {
        match self {
            Self::Up => "Send Up with the live mode-aware VT encoder.",
            Self::Down => "Send Down with the live mode-aware VT encoder.",
            Self::Left => "Send Left with the live mode-aware VT encoder.",
            Self::Right => "Send Right with the live mode-aware VT encoder.",
            Self::Prior => "Send PageUp with the live mode-aware VT encoder.",
            Self::Next => "Send PageDown with the live mode-aware VT encoder.",
            Self::Space => "Send Space with the live mode-aware VT encoder.",
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
