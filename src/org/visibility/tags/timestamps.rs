use super::*;
use crate::org::timestamp::Target;

impl Outline {
    pub(crate) fn planning_target(
        &self,
        buffer: &Buffer,
        point: Scalar,
        name: &str,
    ) -> Result<Target, String> {
        let index = self.tag_target(buffer, point)?;
        if let Some(range) = &self.tags.metadata.entries[index].planning {
            return in_planning(buffer, range.clone(), name);
        }
        let at = self.headings[index].body;
        let prefix = line_prefix(buffer, at)?;
        let suffix = if at == buffer.len_bytes() && !prefix.is_empty() {
            ""
        } else {
            buffer.newline()
        };
        Ok(Target::new(buffer, at..at)?.decorated(format!("{prefix}{name}: "), suffix.into()))
    }
}
pub(super) fn in_planning(
    buffer: &Buffer,
    range: Range<usize>,
    name: &str,
) -> Result<Target, String> {
    let text = planning_text(buffer, range.clone())?;
    let marker = format!("{name}:");
    let mut matches = text.match_indices(&marker);
    if let Some((start, _)) = matches.next() {
        if matches.next().is_some() {
            return Err(format!("Duplicate Org {name} timestamps"));
        }
        return existing_target(
            buffer,
            range.start + start + marker.len(),
            &text[start + marker.len()..],
            name,
        );
    }
    let text = text.trim_end_matches(['\r', '\n']);
    let at = range.start + text.len();
    let separator = if text.ends_with(char::is_whitespace) {
        ""
    } else {
        " "
    };
    Ok(Target::new(buffer, at..at)?.decorated(format!("{separator}{name}: "), String::new()))
}
fn line_prefix(buffer: &Buffer, at: usize) -> Result<&'static str, String> {
    if at == 0 {
        return Ok("");
    }
    let point = buffer
        .byte_span(at..at)
        .map_err(|e| e.to_string())?
        .range
        .start;
    let span = buffer.span(point - 1..point).map_err(|e| e.to_string())?;
    let previous = buffer.slice(&span).map_err(|e| e.to_string())?;
    Ok(if previous == "\n" {
        ""
    } else {
        buffer.newline()
    })
}

fn planning_text(buffer: &Buffer, range: Range<usize>) -> Result<String, String> {
    let span = buffer.byte_span(range).map_err(|e| e.to_string())?;
    let text = buffer.slice(&span).map_err(|e| e.to_string())?;
    if text.len_bytes() > 4096 {
        return Err("Org planning line exceeds 4096 bytes".into());
    }
    Ok(text.to_string())
}
fn existing_target(
    buffer: &Buffer,
    start: usize,
    tail: &str,
    name: &str,
) -> Result<Target, String> {
    let spaces = tail.len() - tail.trim_start().len();
    let scalar = Coordinates::new(buffer)
        .map_err(|e| e.to_string())?
        .from_byte(crate::coordinates::Byte(start + spaces))
        .map_err(|e| e.to_string())?;
    let target = Target::at(buffer, scalar)?;
    if !target.exists() {
        return Err(format!("Malformed Org {name} timestamp"));
    }
    Ok(target)
}
