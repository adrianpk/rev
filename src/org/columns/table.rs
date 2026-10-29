use super::Format;
use crate::org::{Cancellation, agenda::QueryResult};
pub(super) fn render(
    result: &QueryResult,
    format: &Format,
    newline: &str,
    cancel: &Cancellation,
) -> Result<String, String> {
    let mut text = String::new();
    record(
        &mut text,
        format.columns.iter().map(|c| c.key.as_str()),
        newline,
    )?;
    text.push('|');
    text.push_str(&vec!["---"; format.columns.len()].join("+"));
    text.push('|');
    text.push_str(newline);
    for row in &result.rows {
        if cancel.is_cancelled() {
            return Err("Column report cancelled".into());
        }
        let values = format
            .columns
            .iter()
            .map(|c| row.fields.get(&c.key).map_or("", String::as_str));
        record(&mut text, values, newline)?;
    }
    Ok(text)
}
fn record<'a>(
    text: &mut String,
    values: impl Iterator<Item = &'a str>,
    newline: &str,
) -> Result<(), String> {
    text.push('|');
    for value in values {
        if value.contains(['|', '\r', '\n', '\u{2028}', '\u{2029}'])
            || value.chars().any(|c| c.is_control() && c != '\t')
        {
            return Err("Column report value cannot be represented as a native table cell".into());
        }
        text.push(' ');
        text.push_str(value);
        text.push_str(" |");
        if text.len() > 1048576 {
            return Err("Column report exceeds 1048576 bytes".into());
        }
    }
    text.push_str(newline);
    Ok(())
}
