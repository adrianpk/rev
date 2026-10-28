//! The finite state schema needs only strings and integers. Encode that shape
//! without enabling another TOML writer dependency or normalizing source text.
use super::ClockState;

pub(super) fn encode(clock: Option<&ClockState>) -> Result<String, String> {
    let Some(clock) = clock else {
        return Ok("version = 1\n".into());
    };
    let path = clock
        .path
        .to_str()
        .ok_or("Org clock source path must be UTF-8 for state persistence")?;
    let mut text = format!(
        "version = 1\n[clock]\npath = {}\nheading = {}\nheading_source = {}\nline_start = {}\nline_end = {}\nsource = {}\nstart_seconds = {}\nacknowledged_seconds = {}\n",
        quoted(path),
        clock.heading,
        quoted(&clock.heading_source),
        clock.line_start,
        clock.line_end,
        quoted(&clock.source),
        clock.start_seconds,
        clock.acknowledged_seconds
    );
    text.push_str(&file_fields(&clock.file)?);
    Ok(text)
}
fn file_fields(file: &crate::file::Observation) -> Result<String, String> {
    let device = i64::try_from(file.device)
        .map_err(|_| "Clock source device exceeds state integer bounds")?;
    let inode =
        i64::try_from(file.inode).map_err(|_| "Clock source inode exceeds state integer bounds")?;
    let length = i64::try_from(file.length)
        .map_err(|_| "Clock source length exceeds state integer bounds")?;
    Ok(format!(
        "[clock.file]\ndevice = {device}\ninode = {inode}\nlength = {length}\nmodified = [{}, {}]\nchanged = [{}, {}]\nmode = {}\n",
        file.modified.0, file.modified.1, file.changed.0, file.changed.1, file.mode
    ))
}
fn quoted(value: &str) -> String {
    let mut text = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            c if c.is_control() => text.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => text.push(c),
        }
    }
    text.push('"');
    text
}
