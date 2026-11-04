use crate::command::{Binding, Command, Motion, Scroll};

use super::{Context, Error, Site};

impl Context {
    pub(super) fn target(
        self,
        sequence: &str,
        name: &str,
        commands: Option<&std::collections::BTreeMap<String, crate::config::AiCommand>>,
    ) -> Result<Binding, Error> {
        let target = Binding::named(name)
            .or_else(|| {
                let key = name.strip_prefix("ai-")?;
                if commands.is_some_and(|commands| !commands.contains_key(key)) {
                    return None;
                }
                crate::command::AiName::new(key)
                    .ok()
                    .map(|name| Binding::Command(Command::AiDefined(name)))
            })
            .ok_or_else(|| {
                Error::new(
                    self,
                    sequence,
                    Site::Value,
                    "expected an existing command name; unbinding values are unsupported",
                )
            })?;
        if !self.local_available(target) {
            return Err(Error::new(
                self,
                sequence,
                Site::Value,
                format!("'{name}' is unavailable in {} context", self.name()),
            ));
        }
        Ok(target)
    }

    // Calendar inheritance admits prompt editing only while that prompt owns
    // focus. Local calendar overrides and host roles retain narrower authority.
    pub(crate) fn local_available(self, target: Binding) -> bool {
        match self {
            Self::OrgCalendar => calendar(target),
            _ => self.available(target),
        }
    }

    fn editing_context(self) -> bool {
        matches!(
            self,
            Self::Editor
                | Self::GitCommit
                | Self::Org
                | Self::OrgCapture
                | Self::OrgSourceEdit
                | Self::VimNormal
                | Self::VimInsert
                | Self::VimReplace
                | Self::VimVisualCharacter
                | Self::VimVisualLine
                | Self::VimVisualBlock
        )
    }

    pub fn available(self, target: Binding) -> bool {
        if self.editing_context() {
            return matches!(target, Binding::Command(_));
        }
        match self {
            Self::Completion => completion(target),
            Self::OrgAgenda => org_navigation(target) || agenda_action(target),
            Self::OrgCalendar => calendar(target) || minibuffer(target),
            Self::Minibuffer => minibuffer(target),
            Self::Tree => tree(target),
            Self::BookmarkList => bookmark_list(target),
            Self::Git | Self::GitHistory | Self::GitInspect => matches!(
                target,
                Binding::Command(
                    Command::Git(_)
                        | Command::Quit
                        | Command::KeyboardQuit
                        | Command::ExecuteExtended
                        | Command::Scroll(
                            Scroll::NextLine
                                | Scroll::PreviousLine
                                | Scroll::PageForward
                                | Scroll::PageBackward
                        )
                        | Command::Motion(Motion::BufferStart | Motion::BufferEnd)
                )
            ),
            _ => unreachable!("editing context handled"),
        }
    }
}

fn minibuffer(target: Binding) -> bool {
    let Binding::Command(command) = target else {
        return true;
    };
    match command {
        Command::KeyboardQuit | Command::BackwardDeleteChar | Command::DeleteChar => true,
        Command::SearchForward | Command::SearchBackward => true,
        Command::Motion(motion) => character_motion(motion),
        _ => false,
    }
}

fn character_motion(motion: Motion) -> bool {
    matches!(
        motion,
        Motion::BackwardChar | Motion::ForwardChar | Motion::LineStart | Motion::LineEnd
    )
}

fn tree(target: Binding) -> bool {
    let Binding::Command(command) = target else {
        return false;
    };
    matches!(
        command,
        Command::Tree(_)
            | Command::Git(
                crate::command::GitCommand::Status | crate::command::GitCommand::Processes
            )
            | Command::KeyboardQuit
            | Command::Quit
            | Command::MouseMode
            | Command::ExecuteExtended
            | Command::FindFile
            | Command::InsertTab
            | Command::Newline
    ) || tree_navigation(command)
}

fn tree_navigation(command: Command) -> bool {
    matches!(
        command,
        Command::Scroll(Scroll::NextLine | Scroll::PreviousLine)
            | Command::Motion(Motion::ForwardChar | Motion::BackwardChar)
    )
}

fn org_navigation(target: Binding) -> bool {
    matches!(
        target,
        Binding::Command(
            Command::KeyboardQuit
                | Command::ExecuteExtended
                | Command::Motion(_)
                | Command::Scroll(_)
                | Command::Quit
        )
    )
}

fn calendar(target: Binding) -> bool {
    org_navigation(target) || matches!(target, Binding::Command(Command::Org(c)) if c.calendar())
}

fn agenda_action(target: Binding) -> bool {
    matches!(target,Binding::Command(Command::Org(command)) if command.habit() || command.agenda_control() || command.agenda_query() || command.agenda_bulk() || command.agenda_source_action() || command.agenda_custom() || command.estimate() || command==crate::command::OrgCommand::TimerSet)
}

fn bookmark_list(target: Binding) -> bool {
    use crate::command::BookmarkCommand as B;
    matches!(
        target,
        Binding::Command(
            Command::Git(
                crate::command::GitCommand::Status | crate::command::GitCommand::Processes
            ) | Command::Bookmark(
                B::List
                    | B::MenuVisit
                    | B::MenuDelete
                    | B::MenuUnmark
                    | B::MenuExecute
                    | B::Save
                    | B::Reload
            ) | Command::KeyboardQuit
                | Command::ExecuteExtended
                | Command::Quit
                | Command::Split(_)
                | Command::Window(_)
                | Command::Scroll(
                    Scroll::NextLine
                        | Scroll::PreviousLine
                        | Scroll::PageForward
                        | Scroll::PageBackward
                )
                | Command::Motion(Motion::BufferStart | Motion::BufferEnd)
        )
    )
}

fn completion(target: Binding) -> bool {
    matches!(
        target,
        Binding::Command(
            Command::Completion(
                crate::command::CompletionCommand::Next
                    | crate::command::CompletionCommand::Previous
                    | crate::command::CompletionCommand::Accept
                    | crate::command::CompletionCommand::Cancel
            ) | Command::KeyboardQuit
        )
    )
}
