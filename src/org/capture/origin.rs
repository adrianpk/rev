use crate::{
    buffer::{Buffer, edit::Point},
    coordinates::Coordinates,
};
/// A link records a checked source point at capture start. An unnamed origin
/// has no invented path; callers may use empty annotation/link text explicitly.
pub fn origin_link(buffer: &Buffer, point: Point) -> Result<Option<String>, String> {
    buffer.region(point).map_err(|e| e.to_string())?;
    let line = Coordinates::new(buffer)
        .map_err(|e| e.to_string())?
        .to_line(point.position)
        .map_err(|e| e.to_string())?
        .row
        + 1;
    let Some(file) = buffer.file() else {
        return Ok(None);
    };
    let path = file
        .path()
        .to_str()
        .ok_or("Capture origin file path is not UTF-8")?;
    if path.len() > 4096 {
        return Err("Capture origin path exceeds 4096 bytes".into());
    }
    let mut escaped = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"/-_.~".contains(&byte) {
            escaped.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(escaped, "%{byte:02X}").map_err(|e| e.to_string())?;
        }
    }
    Ok(Some(format!("[[file:{escaped}::{line}]]")))
}
