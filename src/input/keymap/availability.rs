use crate::command::{Binding, Command, Motion, Scroll};

use super::{Context, Error, Site};

impl Context {
    pub(super) fn target(self, sequence: &str, name: &str) -> Result<Binding, Error> {
        let target = Binding::named(name).ok_or_else(|| {
            Error::new(
                self,
                sequence,
                Site::Value,
                "expected an existing command name; unbinding values are unsupported",
            )
        })?;
        if !self.available(target) {
            return Err(Error::new(
                self,
                sequence,
                Site::Value,
                format!("'{name}' is unavailable in {} context", self.name()),
            ));
        }
        Ok(target)
    }

    pub fn available(self, target: Binding) -> bool {
        match self {
            Self::Editor => matches!(target, Binding::Command(_)),
            Self::Minibuffer => minibuffer(target),
            Self::Tree => tree(target),
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
            | Command::KeyboardQuit
            | Command::Quit
            | Command::MouseMode
            | Command::ExecuteExtended
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
