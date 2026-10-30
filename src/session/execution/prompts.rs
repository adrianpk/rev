use super::*;
impl Session {
    pub(super) fn confirm_execution(&mut self) -> Result<(), String> {
        let work = self
            .execution
            .as_ref()
            .and_then(|s| s.work.as_ref())
            .ok_or("No selected source program")?;
        self.input
            .argument(
                &format!("Execute {} with host authority? yes/no: ", work.label()),
                "",
            )
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::Execution(Prompt::Confirm));
        self.message("Confirm selected source execution; source headers cannot grant authority");
        Ok(())
    }
    pub(in crate::session) fn execution_argument(&mut self, prompt: Prompt, value: &str) -> Action {
        if let Prompt::Inspect = prompt {
            return self.execution_inspection(value);
        }
        let result = match prompt {
            Prompt::Profile => self.choose_execution(value.trim()),
            Prompt::Confirm => match value.trim() {
                "yes" => self.launch_execution(),
                "no" => {
                    return self.cancel_execution();
                }
                _ => self.confirm_execution(),
            },
            Prompt::Inspect => unreachable!("handled inspection"),
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => {
                self.execution = None;
                self.error(&e)
            }
        }
    }
    pub(super) fn inspect_execution(&mut self) -> Action {
        let Some(out) = &self.execution_result else {
            return self.message("No source execution outcome");
        };
        let summary = format!(
            "Source {:?}, status {:?}, stdout {} bytes, stderr {} bytes; {}",
            out.state,
            out.status,
            out.stdout.len(),
            out.stderr.len(),
            out.detail
        );
        match self.input.argument(
            "Execution detail (stdout/stderr/source; empty summary): ",
            "",
        ) {
            Ok(()) => {
                self.pending = Some(Pending::Execution(Prompt::Inspect));
                self.message(&summary)
            }
            Err(e) => self.error(&e.to_string()),
        }
    }
    fn execution_inspection(&mut self, value: &str) -> Action {
        let Some(out) = &self.execution_result else {
            return self.message("No source execution outcome");
        };
        let bytes = match value.trim() {
            "stdout" => &out.stdout[..],
            "stderr" => &out.stderr[..],
            "source" => out.definition.draft.as_bytes(),
            "" => {
                return self.message(&format!(
                    "Source {:?}, status {:?}; {}",
                    out.state, out.status, out.detail
                ));
            }
            _ => return self.error("Choose stdout, stderr, source or empty summary"),
        };
        let text = String::from_utf8_lossy(bytes)
            .chars()
            .take(1024)
            .flat_map(char::escape_default)
            .collect::<String>();
        self.message(&format!("{value} (first 1024 characters): {text}"))
    }
}
