use super::*;
impl Session {
    pub(in crate::session) fn export_argument(&mut self, prompt: Prompt, draft: &str) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.error("Export source or view changed; destination unchanged");
        }
        match self.export_selection(prompt, draft) {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        }
    }
    fn export_selection(&mut self, prompt: Prompt, draft: &str) -> Result<(), String> {
        let Prompt {
            request,
            backend,
            stage,
        } = prompt;
        match stage {
            Stage::Scope => {
                let scope = self.export_scope(draft)?;
                self.export_prompt(
                    Prompt {
                        request,
                        backend,
                        stage: Stage::Options(scope),
                    },
                    "Export OPTIONS (empty uses file): ",
                )
            }
            Stage::Options(scope) => {
                let options = if draft.is_empty() {
                    None
                } else {
                    Some(Options::parse(draft)?)
                };
                self.export_prompt(
                    Prompt {
                        request,
                        backend,
                        stage: Stage::Path(scope, options),
                    },
                    "Export to local file: ",
                )
            }
            Stage::Path(scope, options) => {
                self.export_path(request, (backend, scope, options), draft)
            }
            Stage::Confirm(prepared) => match draft {
                "yes" => self.publish_export(request, *prepared),
                "no" => {
                    self.message("Org export declined; destination unchanged");
                    Ok(())
                }
                _ => Err("Export replacement requires yes or no".into()),
            },
        }
    }
    fn export_scope(&self, value: &str) -> Result<Scope, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        match value {
            "" | "buffer" => Ok(Scope::Buffer),
            "subtree" => Ok(Scope::Subtree(
                Coordinates::new(buffer)
                    .map_err(|e| e.to_string())?
                    .to_byte(self.view().point.position)
                    .map_err(|e| e.to_string())?,
            )),
            "region" => self.export_region(),
            _ => Err("Choose buffer, subtree or region export scope".into()),
        }
    }
    fn export_region(&self) -> Result<Scope, String> {
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let span = buffer
            .region(self.view().point)
            .map_err(|e| e.to_string())?
            .ok_or("Select an active region for export")?;
        let source = Source::new(buffer);
        let span_all = buffer
            .span(0..buffer.len_chars())
            .map_err(|e| e.to_string())?;
        let text = buffer.slice(&span_all).map_err(|e| e.to_string())?;
        Ok(Scope::Region(
            source
                .range(
                    Byte(text.char_to_byte(span.range.start))
                        ..Byte(text.char_to_byte(span.range.end)),
                )
                .map_err(|e| e.to_string())?,
        ))
    }
    fn export_path(
        &mut self,
        request: Request,
        selection: (ExportBackend, Scope, Option<Options>),
        draft: &str,
    ) -> Result<(), String> {
        let (backend, scope, options) = selection;
        let path = self.export_destination(draft)?;
        let target = file::resolve(&path).map_err(|e| e.to_string())?;
        self.export_target_available(&target)?;
        let policy = &self.buffers.snapshot().settings().org;
        let time = self
            .org_clock
            .unwrap_or(super::super::outline::timestamps::capture)(&policy.time)?;
        let selected = export::Request {
            backend,
            scope,
            options,
            time,
        };
        let work = export::Work::new(
            &self.buffers,
            request.origin.point.stamp.buffer,
            selected,
            policy,
        )?;
        self.launch_export(request, Work::Prepare(Box::new(work), target))
    }
    fn export_destination(&self, draft: &str) -> Result<PathBuf, String> {
        crate::config::capture::path(draft)?;
        if draft.is_empty() || draft.len() > 4096 || draft.chars().any(char::is_control) {
            return Err("Enter a local export path of at most 4096 bytes without controls".into());
        }
        let base = self
            .buffer()
            .map_err(|e| e.to_string())?
            .file()
            .and_then(|f| f.path().parent())
            .unwrap_or(Path::new("."));
        Ok(base.join(draft))
    }
}
