use super::*;
use crate::{
    buffer::Save,
    file::{self, Location, Publication},
    org::{
        io::{CaptureRead, DestinationSource, DestinationVersion},
        table::interchange::Work,
    },
};
use std::path::{Path, PathBuf};
pub(in crate::session) struct Prompt {
    request: Request,
    save: Option<Save>,
}
impl Session {
    pub(super) fn table_file_prompt(&mut self, request: Request) -> Result<(), String> {
        let label = if request.command == C::TableImport {
            "Import table file (.csv/.tsv): "
        } else {
            "Export table file (.csv/.tsv): "
        };
        self.input.argument(label, "").map_err(|e| e.to_string())?;
        self.pending = Some(Pending::TableFile(Box::new(Prompt {
            request,
            save: None,
        })));
        self.message("Enter a literal local path; C-g cancels");
        Ok(())
    }
    pub(in crate::session) fn table_file_argument(
        &mut self,
        prompt: Prompt,
        draft: &str,
    ) -> Action {
        if !self.org_source_current(&prompt.request) {
            return self.error("Table source or view changed; files unchanged");
        }
        let result = if let Some(save) = prompt.save {
            self.table_export_confirm(prompt.request, save, draft)
        } else {
            self.table_file_path(prompt.request, draft)
        };
        match result {
            Ok(()) => Action::Redraw,
            Err(e) => self.error(&e),
        }
    }
    fn table_file_path(&mut self, request: Request, draft: &str) -> Result<(), String> {
        let path = self.table_path(draft)?;
        let policy = &self.buffers.snapshot().settings().org;
        let snapshot = Snapshot::new(
            self.buffer().map_err(|e| e.to_string())?,
            request.origin.point,
            policy,
        )?;
        let work = if request.command == C::TableImport {
            let limits = crate::org::table::interchange::input_limits(&policy.limits);
            let read = CaptureRead::new(&self.buffers, &path, &limits)?;
            Work::import(snapshot, read, &path)?
        } else {
            let target = file::resolve(&path).map_err(|e| e.to_string())?;
            self.table_target_available(&target)?;
            Work::export(snapshot, target, &path)?
        };
        self.launch_table_file(request, work)
    }
    fn table_path(&self, draft: &str) -> Result<PathBuf, String> {
        if draft.is_empty() || draft.len() > 4096 || draft.chars().any(char::is_control) {
            return Err("Enter a local file path of at most 4096 bytes without controls".into());
        }
        let base = self
            .buffer()
            .map_err(|e| e.to_string())?
            .file()
            .and_then(|f| f.path().parent())
            .unwrap_or(Path::new("."));
        Ok(base.join(draft))
    }
    pub(super) fn table_export_review(
        &mut self,
        text: String,
        target: Location,
        request: Request,
    ) -> Result<(), String> {
        self.table_target_available(&target)?;
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let save = Save::projection(buffer, text, target.clone());
        if !target.is_present() {
            return self.launch_table_file(request, Work::Publish(save));
        }
        self.input
            .argument(
                &format!("Replace table data {}? yes/no: ", target.path().display()),
                "",
            )
            .map_err(|e| e.to_string())?;
        self.pending = Some(Pending::TableFile(Box::new(Prompt {
            request,
            save: Some(save),
        })));
        self.message("Confirm replacement of this observed file; C-g cancels");
        Ok(())
    }
    fn table_export_confirm(
        &mut self,
        request: Request,
        save: Save,
        draft: &str,
    ) -> Result<(), String> {
        match draft {
            "yes" => self.launch_table_file(request, Work::Publish(save)),
            "no" => {
                self.message("Table export declined; files unchanged");
                Ok(())
            }
            _ => Err("Table export requires yes or no; files unchanged".into()),
        }
    }
    fn launch_table_file(&mut self, request: Request, work: Work) -> Result<(), String> {
        if let Work::Publish(save) = &work {
            let target = file::resolve(save.target()).map_err(|e| e.to_string())?;
            self.table_target_available(&target)?;
        }
        let seconds = self.buffers.snapshot().settings().org.limits.job_seconds;
        self.org
            .start_table_file(request.origin.point.stamp, work, seconds)?;
        self.table_request = Some(request);
        self.message("Org table data work running; C-g cancels");
        Ok(())
    }
    fn table_target_available(&self, target: &Location) -> Result<(), String> {
        let identity = target.observation().map(|o| (o.device, o.inode));
        for file in self.buffers.iter().filter_map(|b| b.file()) {
            let same =
                identity.is_some() && file.observation().map(|o| (o.device, o.inode)) == identity;
            if file.path() == target.path() || same {
                return Err(
                    "Table export destination is owned by a live buffer; choose another path"
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub(super) fn table_import_current(&self, source: &DestinationSource) -> Result<(), String> {
        let current = match &source.version {
            DestinationVersion::Buffer(stamp) => self
                .buffers
                .get(stamp.buffer)
                .is_ok_and(|b| b.stamp() == *stamp),
            DestinationVersion::Disk(observed) => {
                file::observe(&source.path)
                    .map_err(|e| e.to_string())?
                    .as_ref()
                    == Some(observed)
            }
            DestinationVersion::Missing => false,
        };
        if current {
            Ok(())
        } else {
            Err("Table import source changed; destination unchanged".into())
        }
    }
    pub(super) fn table_export_result(&mut self, outcome: file::Outcome) {
        let path = outcome.target().display();
        let detail = outcome.error().map(ToString::to_string).unwrap_or_default();
        match outcome.publication() {
            Publication::Complete => {
                self.message(&format!(
                    "Exported table data to {path}; Org source unchanged"
                ));
            }
            Publication::Uncertain => {
                self.error(&format!(
                    "Published table data to {path}; completion unresolved: {detail}"
                ));
            }
            Publication::NotPublished => {
                self.error(&format!("Table data not published to {path}: {detail}"));
            }
        }
    }
}
