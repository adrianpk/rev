use crate::{
    ai::pi::{Model, Operation, Output},
    input::Focus,
    session::{Action, Pending, Session},
};
use std::path::PathBuf;

pub(super) enum Outcome {
    Generated(Output),
    Models(Vec<Model>),
}
impl Session {
    pub(super) fn ai_root(&self) -> PathBuf {
        self.ai
            .directory
            .clone()
            .or_else(|| self.source_root().map(ToOwned::to_owned))
            .or_else(|| {
                self.buffer()
                    .ok()
                    .and_then(|b| b.file())
                    .and_then(|f| f.path().parent())
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| self.ai.launch.clone())
    }
    pub(super) fn ai_reap_idle(&mut self) -> Result<(), String> {
        if self
            .ai
            .idle
            .as_ref()
            .is_some_and(Operation::worker_finished)
        {
            let mut old = self.ai.idle.take().expect("finished Pi owner");
            self.ai.cleanup_failed |= old.cleanup_failed();
            if let Some(Err(error)) = old.take() {
                return Err(error);
            }
        }
        if self.ai.cleanup_failed {
            return Err("Pi cleanup failed; this session cannot start another child".into());
        }
        Ok(())
    }
    fn ai_close_idle(&mut self) {
        if let Some(old) = self.ai.idle.take() {
            old.cancel();
            self.ai.operation = Some(old);
            self.ai.source = None;
            self.ai.query = false;
        }
    }
    fn ai_control_input(&self) -> bool {
        self.input.focus() == Focus::Editing && self.pending.is_none()
    }
    pub(in crate::session) fn ai_set_directory(&mut self) -> Action {
        if !self.ai_control_input() {
            return self.error("Finish the current input before choosing an AI directory");
        }
        self.cancel_ai();
        self.ai_close_idle();
        let initial = self.ai_root().to_string_lossy().into_owned();
        if let Err(error) = self.input.argument("AI directory: ", &initial) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::AiDirectory);
        self.message(
            "Choose a session-local AI context directory; source and workspace stay independent",
        )
    }
    pub(in crate::session) fn ai_directory_argument(&mut self, value: &str) -> Action {
        if value.is_empty() || value.len() > 4096 || value.contains('\0') {
            return self.error("AI directory must contain 1..=4096 bytes without NUL");
        }
        let path = PathBuf::from(value);
        let path = if path.is_absolute() {
            path
        } else {
            self.ai_root().join(path)
        };
        match path.canonicalize() {
            Ok(root) if root.is_dir() => {
                self.ai.directory = Some(root);
                self.message("AI directory selected for later invocations; history crosses an isolation boundary")
            }
            _ => self.error("AI directory is unavailable or is not a directory"),
        }
    }
    pub(in crate::session) fn ai_select_model(&mut self) -> Action {
        if !self.ai_control_input() {
            return self.error("Finish the current input before choosing an AI model");
        }
        if let Err(error) = self.ai_reap_idle() {
            return self.error(&error);
        }
        if self.ai.operation.is_some() {
            self.cancel_ai();
            return self.message("Pi stopping; retry ai-select-model after owned cleanup");
        }
        let Some(epoch) = self.ai.epoch.checked_add(1) else {
            return self.error("Pi invocation identity exhausted");
        };
        let operation = if let Some(mut idle) = self.ai.idle.take() {
            if let Err(error) = idle.query_models(epoch) {
                idle.cancel();
                self.ai.operation = Some(idle);
                return self.error(&error);
            }
            idle
        } else {
            let mut policy = self.buffers.snapshot().settings().ai.pi.clone();
            policy.model = self.ai.model.clone().or(policy.model);
            match Operation::models(policy, self.ai_root(), epoch) {
                Ok(op) => op,
                Err(error) => return self.error(&error),
            }
        };
        self.ai.epoch = epoch;
        self.ai.operation = Some(operation);
        self.ai.query = true;
        self.ai.accepted = false;
        self.message("Reading Pi-reported models; C-g cancels")
    }
    pub(super) fn ai_model_choices(&mut self, models: Vec<Model>) -> Action {
        if models.is_empty() {
            return self.error("Pi reported no models; configure models/authentication in Pi");
        }
        if !self.ai_control_input() {
            return self.message(
                "Pi models completed while another input owns focus; retry ai-select-model",
            );
        }
        let names = models.iter().map(Model::reference).collect();
        if let Err(error) = self.input.names_argument("AI model: ", "", names) {
            return self.error(&error.to_string());
        }
        self.pending = Some(Pending::AiModel(models));
        self.message(
            "Choose a Pi-reported provider/model; selection is local to this editor session",
        )
    }
    pub(in crate::session) fn ai_model_argument(
        &mut self,
        models: Vec<Model>,
        value: &str,
    ) -> Action {
        let Some(model) = models.into_iter().find(|m| m.reference() == value) else {
            return self.error("Choose exactly a Pi-reported provider/model");
        };
        self.ai_close_idle();
        self.ai.model = Some(model.reference());
        self.message(
            "AI model selected; compatible history will be chosen on the next explicit invocation",
        )
    }
    pub(in crate::session) fn ai_reset_session(&mut self) -> Action {
        if !self.ai_control_input() {
            return self.error("Finish the current input before resetting AI history");
        }
        self.cancel_ai();
        self.ai_close_idle();
        let root = match self.ai_root().canonicalize() {
            Ok(p) => p,
            Err(_) => return self.error("AI directory unavailable"),
        };
        let model = self
            .ai
            .model
            .clone()
            .or_else(|| self.buffers.snapshot().settings().ai.pi.model.clone());
        let Some(epoch) = self.ai.epoch.checked_add(1) else {
            return self.error("Pi invocation identity exhausted");
        };
        self.ai.reset = Some((root, model, epoch));
        self.message("Next invocation under this AI directory/model starts fresh owned history; source is retained")
    }
    pub(in crate::session) fn ai_disconnect(&mut self) -> Action {
        self.cancel_ai();
        self.ai_close_idle();
        self.message("Pi disconnect requested; owned cleanup finishes before another child; private history is retained")
    }
    pub(in crate::session) fn ai_cancel_command(&mut self) -> Action {
        self.cancel_ai_input()
            .unwrap_or_else(|| self.message("No active AI invocation or owned input to cancel"))
    }
}
