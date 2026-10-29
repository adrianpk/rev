use super::*;
pub(super) struct Include {
    name: String,
    lines: Option<(usize, Option<usize>)>,
}
impl Include {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.len() > 4096 {
            return Err("Org include directive exceeds 4096 bytes".into());
        }
        let value = value.trim();
        let tail = value
            .strip_prefix('"')
            .ok_or("Org include requires a quoted local path")?;
        let end = tail.find('"').ok_or("Unclosed Org include path")?;
        let name = &tail[..end];
        crate::config::capture::path(name)?;
        if name.is_empty() || name.contains("::") || name.contains('\\') {
            return Err("Unsupported Org include path".into());
        }
        let rest = tail[end + 1..].trim();
        let lines = if rest.is_empty() {
            None
        } else {
            Some(line_option(rest)?)
        };
        Ok(Self {
            name: name.into(),
            lines,
        })
    }
    pub fn path(&self, parent: Option<&Path>) -> Result<PathBuf, String> {
        let path = Path::new(&self.name);
        if path.is_absolute() {
            return Ok(path.to_owned());
        }
        let parent = parent
            .and_then(Path::parent)
            .ok_or("Relative Org include requires a source file location")?;
        Ok(parent.join(path))
    }
    pub fn range(&self, text: &Rope) -> Result<Range<usize>, String> {
        let Some((start, end)) = self.lines else {
            return Ok(0..text.len_bytes());
        };
        let end = end.unwrap_or(text.len_lines() + 1);
        if start == 0 || start > end || end > text.len_lines() + 1 {
            return Err("Org include line range is outside the source".into());
        }
        Ok(line_byte(text, start)..line_byte(text, end))
    }
}
fn line_byte(text: &Rope, line: usize) -> usize {
    if line > text.len_lines() {
        text.len_bytes()
    } else {
        text.line_to_byte(line - 1)
    }
}
fn line_option(value: &str) -> Result<(usize, Option<usize>), String> {
    let range = value
        .strip_prefix(":lines")
        .ok_or("Unsupported native Org include option")?
        .trim();
    let range = range
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or("Use :lines with a quoted start-end range")?;
    let (start, end) = range
        .split_once('-')
        .ok_or("Org include requires start-end lines")?;
    let start = if start.is_empty() {
        1
    } else {
        start
            .parse()
            .map_err(|_| "Invalid Org include first line")?
    };
    let end = if end.is_empty() {
        None
    } else {
        Some(end.parse().map_err(|_| "Invalid Org include end line")?)
    };
    Ok((start, end))
}
