use crate::config::OrgLimits;
pub(super) fn validate(limits: &OrgLimits) -> Result<(), String> {
    for (value, max) in [
        (limits.include_depth, 64),
        (limits.include_files, 512),
        (limits.expansion_bytes, 8388608),
        (limits.export_bytes, 134217728),
        (limits.agenda_file_bytes, 33554432),
        (limits.agenda_total_bytes, 268435456),
        (limits.job_seconds, 600),
    ] {
        if value == 0 || value > max {
            return Err("Org export requires finite configured resource limits".into());
        }
    }
    Ok(())
}
