//! One session-owned draft, separate from canonical source publication.
mod finalize;
mod lifecycle;
mod origin;
mod prompt;
use super::{Action, Pending, Session};
use crate::{
    buffer::{Buffer, BufferId, edit::Point},
    command::{Command, OrgCommand as C, WindowCommand},
    org::{
        Cancellation,
        capture::{self, Choice, Context, Discovery, DraftText, Expansion, Resolved, Step},
    },
    window::ViewId,
};
use std::{path::PathBuf, time::Duration};

pub(super) struct State {
    origin: Origin,
    stage: Stage,
}
struct Origin {
    view: ViewId,
    point: Point,
    directory: PathBuf,
    context: Context,
}
enum Stage {
    Template,
    File(Choice),
    Expanding(Choice, Box<Expansion>),
    Queued(Box<Discovery>, DraftText),
    Waiting(DraftText),
    Draft(BufferId, Box<Resolved>),
    Finalizing(Box<finalize::Validation>),
    Published(Box<finalize::Stored>),
}
#[derive(Clone)]
pub(super) enum Prompt {
    Template,
    File,
    Expansion(capture::Prompt),
}
impl Session {
    /// Borrow the transient owner. Its real identity supports ordinary checked
    /// edits/history but grants no canonical source or filesystem authority.
    pub fn capture_draft(&self) -> Option<&Buffer> {
        self.buffers.get(self.capture.as_ref()?.stage.draft()?).ok()
    }
    pub fn capture_target(&self) -> Option<&Resolved> {
        self.capture.as_ref()?.stage.target()
    }
    pub(super) fn capture_selected(&self) -> bool {
        self.capture_draft()
            .is_some_and(|b| b.id() == self.view().point.stamp.buffer)
    }
    pub(super) fn capture_blocks_input(&self) -> bool {
        self.capture
            .as_ref()
            .is_some_and(|s| !matches!(s.stage, Stage::Draft(..) | Stage::Published(_)))
    }
    pub(super) fn capture_command(&mut self, command: Command) -> Option<Action> {
        self.capture.as_ref()?;
        if command == Command::Org(C::CaptureFinalize) {
            return Some(self.begin_capture_finalize());
        }
        if let Some(action) = self.capture_cancellation(command) {
            return Some(action);
        }
        if self.capture_blocks_input() {
            return Some(self.message("Capture input pending; C-g cancels"));
        }
        if protected(command) || self.capture_selected() && draft_protected(command) {
            return Some(
                self.message("Capture draft retained; C-c C-k aborts and returns to origin"),
            );
        }
        None
    }
    fn capture_cancellation(&mut self, command: Command) -> Option<Action> {
        if command == Command::Org(C::CaptureKill) {
            return Some(self.abort_capture());
        }
        if command != Command::KeyboardQuit {
            return None;
        }
        if self.capture_validating() {
            return Some(self.cancel_capture_validation());
        }
        self.capture_blocks_input().then(|| self.abort_capture())
    }
    pub(super) fn capture_remaining(&self) -> Option<Duration> {
        let state = self.capture.as_ref()?;
        if self.pending.is_some() || self.org.busy() {
            return None;
        }
        let active = matches!(state.stage, Stage::Expanding(..) | Stage::Queued(..))
            || matches!(&state.stage, Stage::Finalizing(v) if v.work.is_some());
        active.then_some(Duration::ZERO)
    }
    pub(super) fn expire_capture(&mut self) -> Action {
        if self.capture_remaining().is_none() {
            return Action::Wait;
        }
        self.advance_capture();
        Action::Redraw
    }
    pub(super) fn advance_capture(&mut self) {
        if self.pending.is_some() || self.org.busy() {
            return;
        }
        let Some(state) = self.capture.take() else {
            return;
        };
        if matches!(state.stage, Stage::Finalizing(_)) {
            self.launch_capture_validation(state);
            return;
        }
        if !matches!(state.stage, Stage::Expanding(..) | Stage::Queued(..)) {
            self.capture = Some(state);
            return;
        }
        if !self.capture_current(&state.origin) {
            self.error("Capture origin changed before draft preparation");
            return;
        }
        let State { origin, stage } = state;
        match self.capture_step(&origin, stage) {
            Ok(stage) => self.capture = Some(State { origin, stage }),
            Err(error) => {
                self.error(&error);
            }
        }
    }
    fn capture_step(&mut self, origin: &Origin, stage: Stage) -> Result<Stage, String> {
        match stage {
            Stage::Expanding(choice, mut work) => match work.advance(&Cancellation::default())? {
                Step::Progress => Ok(Stage::Expanding(choice, work)),
                Step::Prompt(prompt) => {
                    self.capture_prompt(Prompt::Expansion(prompt))?;
                    Ok(Stage::Expanding(choice, work))
                }
                Step::Ready(text) => {
                    let policy = &self.buffers.snapshot().settings().org;
                    let discovery =
                        choice.discover(&self.buffers, policy, origin.context.now.date)?;
                    Ok(Stage::Queued(Box::new(discovery), text))
                }
            },
            Stage::Queued(work, text) => {
                self.org.start_capture(origin.point.stamp, *work)?;
                self.message("Preparing capture destination; C-g cancels");
                Ok(Stage::Waiting(text))
            }
            _ => unreachable!("active capture stage"),
        }
    }
}
fn protected(command: Command) -> bool {
    matches!(
        command,
        Command::Quit
            | Command::Buffer(_)
            | Command::Window(WindowCommand::Delete | WindowCommand::Keep)
    )
}
fn draft_protected(command: Command) -> bool {
    matches!(
        command,
        Command::SaveBuffer
            | Command::WriteFile
            | Command::FindFile
            | Command::Mode(_)
            | Command::Split(_)
            | Command::Tree(_)
            | Command::Org(_)
    )
}

impl Stage {
    fn draft(&self) -> Option<BufferId> {
        match self {
            Self::Draft(id, _) => Some(*id),
            Self::Finalizing(v) => Some(v.draft),
            Self::Published(p) => Some(p.draft),
            _ => None,
        }
    }
    fn target(&self) -> Option<&Resolved> {
        match self {
            Self::Draft(_, target) => Some(target),
            Self::Finalizing(v) => Some(&v.target),
            Self::Published(p) => Some(&p.target),
            _ => None,
        }
    }
}
