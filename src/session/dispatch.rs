//! Native command routing after session admission and navigation bookkeeping.
use crate::command::Command;

use super::{Action, Session, ai};

impl Session {
    pub(super) fn dispatch(&mut self, command: Command) -> Action {
        self.navigation(command);
        match command {
            Command::Plugin(command) => self.plugin_command(command),
            Command::PluginDefined(name) => self.plugin_defined(name),
            Command::Quit => self.quit(),
            Command::MacroNamed(name) => self.macro_named_call(name, Default::default()),
            Command::MacroBinding(id) => self.macro_bound_call(id, Default::default()),
            _ => self.dispatch_source_command(command),
        }
    }

    fn dispatch_source_command(&mut self, command: Command) -> Action {
        match command {
            Command::FindFile => self.find_file(),
            Command::GenerateNewBuffer => self.generate_new_buffer(),
            Command::OpenDirectory => self.open_directory(),
            Command::ContextClose => self.close_context(),
            Command::DashboardOpen => self.dashboard_open(),
            Command::SaveBuffer => self.save_buffer(),
            Command::RevertBuffer => self.revert_buffer(),
            Command::WriteFile => self.write_file(),
            _ => self.dispatch_project_command(command),
        }
    }

    fn dispatch_project_command(&mut self, command: Command) -> Action {
        match command {
            Command::ProjectCreate => self.project_create(),
            Command::ProjectPromote => self.project_promote(),
            Command::ProjectSwitch | Command::ProjectList => self.project_list(),
            command @ (Command::ProjectRename
            | Command::ProjectForget
            | Command::ProjectSetRoot
            | Command::ProjectPromoteToWorkspace
            | Command::ProjectDuplicateAsWorkspace) => self.manage_context(command),
            _ => self.dispatch_workspace_command(command),
        }
    }

    fn dispatch_workspace_command(&mut self, command: Command) -> Action {
        match command {
            Command::WorkspaceCreate => self.workspace_create(),
            Command::WorkspaceOpen | Command::WorkspaceList => self.workspace_list(),
            Command::WorkspaceAddDirectory => self.workspace_add_directory(),
            Command::WorkspaceAddProject => self.workspace_add_project(),
            Command::WorkspaceRemoveRoot => self.workspace_remove_root(),
            command @ (Command::WorkspaceRename
            | Command::WorkspaceForget
            | Command::WorkspaceSetRootPath) => self.manage_context(command),
            _ => self.dispatch_ai_request(command),
        }
    }

    fn dispatch_ai_request(&mut self, command: Command) -> Action {
        match command {
            Command::AiSaveCommand => self.ai_save_command(),
            Command::AiDefined(name) => self.ai_defined(name),
            Command::AiExplain => self.ai_command(ai::Kind::Explain),
            Command::AiPrompt => self.ai_command(ai::Kind::Prompt),
            Command::AiImplement => self.ai_command(ai::Kind::Implement),
            _ => self.dispatch_ai_control(command),
        }
    }

    fn dispatch_ai_control(&mut self, command: Command) -> Action {
        match command {
            Command::AiApplyResult => self.ai_apply_result(),
            Command::AiDiscardResult => self.ai_discard_result(),
            Command::AiSetDirectory => self.ai_set_directory(),
            Command::AiSelectModel => self.ai_select_model(),
            Command::AiResetSession => self.ai_reset_session(),
            Command::AiDisconnect => self.ai_disconnect(),
            Command::AiCancel => self.ai_cancel_command(),
            _ => self.dispatch_shell_command(command),
        }
    }

    fn dispatch_shell_command(&mut self, command: Command) -> Action {
        match command {
            Command::ShellCommand => self.shell_command(crate::shell::Kind::Command),
            Command::ShellCommandOnRegion => self.shell_command(crate::shell::Kind::Region),
            Command::ShellFilterRegion => self.shell_command(crate::shell::Kind::Filter),
            Command::ShellCommandInsert => self.shell_command(crate::shell::Kind::Insert),
            Command::ShellCommandCancel => self.shell_cancel(),
            Command::ShellCommandOutput => self.shell_output(),
            _ => self.dispatch_editing_command(command),
        }
    }

    fn dispatch_editing_command(&mut self, command: Command) -> Action {
        match command {
            Command::Buffer(command) => self.buffer_command(command),
            Command::Motion(motion) => self.motion(motion),
            Command::Scroll(scroll) => self.scroll(scroll),
            Command::History(history) => self.history(history),
            Command::KeyboardQuit => self.keyboard_quit(),
            _ => self.edit_command(command),
        }
    }
}
