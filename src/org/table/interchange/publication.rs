use crate::{buffer::Save, file::Outcome, org::Cancellation};
pub(super) fn publish(save: &Save, cancel: &Cancellation) -> Outcome {
    crate::org::io::publication::publish(save, cancel, || Ok(()))
}
