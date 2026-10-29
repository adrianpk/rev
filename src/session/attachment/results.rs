use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingStatus {
    Unchanged,
    Changed,
    Unresolved,
}
pub struct AttachmentResult {
    pub file: FileOutcome,
    pub heading: HeadingStatus,
    pub source_error: Option<String>,
}
impl Session {
    pub fn attachment_results(&self) -> &VecDeque<AttachmentResult> {
        &self.attachment_results
    }
    pub(in crate::session) fn finish_attachment_effect(&mut self, file: FileOutcome) {
        let mut result = AttachmentResult {
            file,
            heading: HeadingStatus::Unchanged,
            source_error: None,
        };
        if result.file.effect != Effect::Rejected {
            match self.publish_attachment_heading(&result.file) {
                Ok(()) => result.heading = HeadingStatus::Changed,
                Err(e) => {
                    result.heading = HeadingStatus::Unresolved;
                    result.source_error = Some(e);
                }
            }
        }
        self.attachment = None;
        self.attachment_results.push_front(result);
        self.attachment_results.truncate(16);
        self.show_attachment_results();
    }
    fn publish_attachment_heading(&mut self, file: &FileOutcome) -> Result<(), String> {
        let edits = self.attachment_metadata(file.id.as_deref())?;
        self.publish_org_edits(&edits).map_err(|e| e.to_string())?;
        let target = format!(
            "attachment:{}",
            crate::org::links::encode(
                file.destination
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("Attached filename is not UTF-8")?
            )
        );
        self.store_attachment_link(
            target,
            file.destination
                .file_name()
                .expect("actual attachment")
                .to_string_lossy()
                .into(),
        );
        Ok(())
    }
    pub(in crate::session) fn show_attachment_results(&mut self) -> Action {
        let Some(result) = self.attachment_results.front() else {
            return self.message("No attachment operation results");
        };
        let text = format!(
            "Attachment result: file {:?}; heading {:?}; {} -> {}; ID {}; created dirs {}; {} {}",
            result.file.effect,
            result.heading,
            result.file.source.display(),
            result.file.destination.display(),
            result.file.id.as_deref().unwrap_or("existing"),
            result.file.created.len(),
            result.file.error.as_deref().unwrap_or(""),
            result.source_error.as_deref().unwrap_or("")
        );
        self.message(&text)
    }
}
