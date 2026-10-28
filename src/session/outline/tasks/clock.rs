//! Task logging observes the selected operation zone once and retains its stamp
//! through note entry, without rereading environment or rules when publishing.
use crate::org::time::{ZoneEnvironment, ZoneSelection, ZonedOperation};
use std::time::SystemTime;

pub(super) fn now(policy: &crate::config::Time) -> Result<String, String> {
    let selection = ZoneSelection::parse(&policy.zone)?;
    let env = ZoneEnvironment::system();
    let operation = ZonedOperation::capture(&selection, &env, || Ok(SystemTime::now()))?;
    Ok(operation.now().format_timestamp(false))
}
