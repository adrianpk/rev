use crate::buffer::edit::{Error, Replacement};
use crate::completion::edits::Edits;
use crate::coordinates::Scalar;
use crate::macros::{CompletionEdit, LiteralCompletion, Step, StopReason};
use crate::session::Session;

impl Session {
    pub(in crate::session) fn macro_capture_completion(
        &mut self,
        edits: &Edits,
    ) -> Result<(), Error> {
        if self.macros.expanded || self.macros.recorder.is_none() {
            return Ok(());
        }
        let point = self.view().point.position.0;
        let captured = self.literal_completion(edits, point);
        let result = captured.and_then(|literal| {
            let recorder = self.macros.recorder.as_mut().expect("capture owner");
            recorder.discard_pending();
            recorder.stage(Step::Completion(literal))?;
            recorder.accept()
        });
        if let Err(reason) = result {
            self.stop_macro(reason, "Macro completion exceeds the capture contract");
            return Err(Error::Capacity);
        }
        Ok(())
    }

    fn literal_completion(
        &self,
        edits: &Edits,
        point: usize,
    ) -> Result<LiteralCompletion, StopReason> {
        if edits.0.len() > crate::macros::STEP_LIMIT {
            return Err(StopReason::CaptureLimit);
        }
        let buffer = self.buffer().map_err(|_| StopReason::NativeFailure)?;
        let mut values = Vec::new();
        let mut bytes: usize = 0;
        for edit in &edits.0 {
            let source = buffer
                .slice(&edit.source)
                .map_err(|_| StopReason::NativeFailure)?;
            bytes = bytes
                .saturating_add(source.len_bytes())
                .saturating_add(edit.text.len());
            if bytes > crate::macros::DEFINITION_BYTES {
                return Err(StopReason::CaptureLimit);
            }
            values.push(CompletionEdit {
                start: relative(edit.source.range.start, point)?,
                end: relative(edit.source.range.end, point)?,
                expected: source.to_string(),
                text: edit.text.clone(),
            });
        }
        Ok(LiteralCompletion {
            edits: values,
            cursor: relative(edits.1.0, point)?,
        })
    }

    pub(super) fn replay_macro_completion(
        &mut self,
        literal: LiteralCompletion,
    ) -> Result<(), String> {
        if !self.macro_source() {
            return Err("Macro completion requires source editing".into());
        }
        let buffer = self.buffer().map_err(|e| e.to_string())?;
        let point = self.view().point.position.0;
        let mut edits = Vec::new();
        for edit in literal.edits {
            let range = absolute(point, edit.start)?..absolute(point, edit.end)?;
            let source = buffer.span(range).map_err(|e| e.to_string())?;
            if buffer.slice(&source).map_err(|e| e.to_string())? != edit.expected {
                return Err("Macro completion source precondition differs".into());
            }
            edits.push(Replacement {
                source,
                text: edit.text,
            });
        }
        if edits.is_empty() {
            return Err("Macro completion has no literal edits".into());
        }
        self.apply_completion((edits, Scalar(absolute(point, literal.cursor)?)))
            .map_err(|e| e.to_string())
    }
}

fn relative(value: usize, point: usize) -> Result<i64, StopReason> {
    let value = i64::try_from(value).map_err(|_| StopReason::CaptureLimit)?;
    let point = i64::try_from(point).map_err(|_| StopReason::CaptureLimit)?;
    value.checked_sub(point).ok_or(StopReason::CaptureLimit)
}

fn absolute(point: usize, offset: i64) -> Result<usize, String> {
    i64::try_from(point)
        .ok()
        .and_then(|at| at.checked_add(offset))
        .and_then(|at| usize::try_from(at).ok())
        .ok_or_else(|| "Macro completion range is outside the source".into())
}
