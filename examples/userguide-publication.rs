//! Project the canonical Markdown documentation into a Spore publication.
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Component, Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const GUIDE: &str = "tutorials/user-guide";

fn main() -> Result<()> {
    let destination = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Expected publication directory")?,
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    let mut files = Vec::new();
    collect(&root, &root, &mut files)?;
    files.sort();
    if std::env::args().nth(2).as_deref() == Some("--finish") {
        for source in &files {
            if source.extension().is_some_and(|value| value == "md") {
                let input = fs::read_to_string(root.join(source))?;
                let output = destination.join("public").join(output_path(source));
                write(
                    &output,
                    &anchor_html(&fs::read_to_string(&output)?, &anchors(&input))?,
                )?;
            }
        }
        return Ok(());
    }
    let mut fragments = BTreeMap::new();
    for source in &files {
        if source.extension().is_some_and(|value| value == "md") {
            let content = fs::read_to_string(root.join(source))?;
            let title = content
                .lines()
                .find_map(|line| line.strip_prefix("# "))
                .ok_or_else(|| format!("{}: missing title", source.display()))?;
            let projected = project(&content, source, &mut fragments)?;
            let metadata = format!(
                "@meta[page_id = \"{}\", short_title = \"{}\"]\n\n",
                page_id(source),
                quoted(title)
            );
            let output = destination.join(format!("{}.spmd", output_path(source).display()));
            write(&output, &(metadata + &projected))?;
        } else {
            let output = destination.join("downloads").join(output_path(source));
            fs::create_dir_all(output.parent().ok_or("Asset has no parent")?)?;
            fs::copy(root.join(source), output)?;
        }
    }
    for (name, content) in fragments {
        write(&destination.join(name), &content)?;
    }
    extend_tree(&destination, &files)?;
    println!(
        "Projected {} Markdown pages for Spore",
        files
            .iter()
            .filter(|path| path.extension().is_some_and(|value| value == "md"))
            .count()
    );
    Ok(())
}

fn collect(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(root, &entry.path(), files)?;
        } else if kind.is_file() {
            files.push(entry.path().strip_prefix(root)?.to_path_buf());
        } else {
            return Err(format!(
                "Unsupported documentation entry: {}",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(())
}

fn output_path(source: &Path) -> PathBuf {
    let mut output = source
        .strip_prefix(GUIDE)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| Path::new("docs").join(source));
    if source.extension().is_some_and(|value| value == "md") {
        output.set_extension("html");
    }
    output
}

fn page_id(source: &Path) -> String {
    let name = source
        .strip_prefix(GUIDE)
        .map(|path| format!("guide-{}", path.display()))
        .unwrap_or_else(|_| format!("docs-{}", source.display()));
    name.trim_end_matches(".md").replace('/', "-")
}

fn write(path: &Path, content: &str) -> Result<()> {
    fs::create_dir_all(path.parent().ok_or("Output has no parent")?)?;
    fs::write(path, content)?;
    Ok(())
}

fn project(
    content: &str,
    source: &Path,
    fragments: &mut BTreeMap<PathBuf, String>,
) -> Result<String> {
    let lines: Vec<_> = content.lines().collect();
    let mut output = String::new();
    let mut fence: Option<(char, usize)> = None;
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if let Some(marker) = fence_marker(line) {
            match fence {
                None => fence = Some(marker),
                Some((kind, width)) if marker.0 == kind && marker.1 >= width => fence = None,
                _ => {}
            }
            output.push_str(line);
            output.push('\n');
            index += 1;
            continue;
        }
        if fence.is_some() {
            output.push_str(line);
            output.push('\n');
            index += 1;
            continue;
        }
        if line.starts_with('|') && index + 1 < lines.len() && alignment(lines[index + 1]) {
            let fragment = PathBuf::from(format!("_fragments/{}-{index}.sp", page_id(source)));
            let mut table = String::from("@table{\n@thead{\n");
            table.push_str(&table_row(line, source, true)?);
            table.push_str("}\n@tbody{\n");
            index += 2;
            while index < lines.len() && lines[index].starts_with('|') {
                table.push_str(&table_row(lines[index], source, false)?);
                index += 1;
            }
            table.push_str("}\n}\n");
            output.push_str(&format!("@include[path = \"{}\"]\n\n", fragment.display()));
            fragments.insert(fragment, table);
            continue;
        }
        if numbered_item(line).is_some() {
            let fragment =
                PathBuf::from(format!("_fragments/{}-steps-{index}.sp", page_id(source)));
            let mut list = String::from("@list[ordered = true]{\n");
            while index < lines.len() {
                let Some(text) = numbered_item(lines[index]) else {
                    break;
                };
                let mut text = text.to_string();
                index += 1;
                while index < lines.len()
                    && lines[index].starts_with([' ', '\t'])
                    && !lines[index].trim().is_empty()
                {
                    text.push(' ');
                    text.push_str(lines[index].trim());
                    index += 1;
                }
                list.push_str(&format!(
                    "@item{{{}}}\n",
                    inline(&rewrite_links(&text, source)?)?
                ));
            }
            list.push_str("}\n");
            output.push_str(&format!("@include[path = \"{}\"]\n\n", fragment.display()));
            fragments.insert(fragment, list);
            continue;
        }
        output.push_str(&rewrite_links(line, source)?);
        output.push('\n');
        index += 1;
    }
    if fence.is_some() {
        return Err(format!("{}: unclosed code fence", source.display()).into());
    }
    Ok(output)
}

fn numbered_item(line: &str) -> Option<&str> {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    line.get(digits..)?.strip_prefix(". ")
}

fn anchors(content: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    let mut fence = None;
    for line in content.lines() {
        if let Some(marker) = fence_marker(line) {
            if fence.is_none() {
                fence = Some(marker);
            } else if fence.is_some_and(|(kind, width)| marker.0 == kind && marker.1 >= width) {
                fence = None;
            }
        } else if fence.is_none()
            && let Some(title) = heading(line)
        {
            let base = slug(title);
            let mut id = base.clone();
            let mut number = 1;
            while !seen.insert(id.clone()) {
                id = format!("{base}-{number}");
                number += 1;
            }
            result.push(id);
        }
    }
    result
}

// Spore assigns numbered heading IDs. Retain them and add the canonical
// Markdown IDs so links and published fragments survive the projection.
fn anchor_html(html: &str, ids: &[String]) -> Result<String> {
    let bytes = html.as_bytes();
    let mut output = String::new();
    let mut copied = 0;
    let mut count = 0;
    for index in 0..bytes.len().saturating_sub(3) {
        if bytes[index] == b'<'
            && bytes[index + 1] == b'h'
            && (b'1'..=b'6').contains(&bytes[index + 2])
            && (bytes[index + 3].is_ascii_whitespace() || bytes[index + 3] == b'>')
        {
            let id = ids
                .get(count)
                .ok_or("Rendered heading has no source counterpart")?;
            output.push_str(&html[copied..index]);
            output.push_str(&format!("<span id=\"{id}\"></span>"));
            copied = index;
            count += 1;
        }
    }
    if count != ids.len() {
        return Err("Source and rendered heading counts differ".into());
    }
    output.push_str(&html[copied..]);
    Ok(output)
}

fn fence_marker(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let first = trimmed.chars().next()?;
    let count = trimmed.chars().take_while(|value| *value == first).count();
    (matches!(first, '`' | '~') && count >= 3).then_some((first, count))
}

fn heading(line: &str) -> Option<&str> {
    let width = line.bytes().take_while(|byte| *byte == b'#').count();
    (1..=6)
        .contains(&width)
        .then(|| line.get(width..)?.strip_prefix(' '))
        .flatten()
}

fn slug(title: &str) -> String {
    title
        .trim()
        .trim_end_matches('#')
        .trim()
        .to_lowercase()
        .chars()
        .filter(|value| value.is_alphanumeric() || matches!(value, ' ' | '-' | '_'))
        .map(|value| if value == ' ' { '-' } else { value })
        .collect()
}

fn rewrite_links(line: &str, source: &Path) -> Result<String> {
    let mut output = String::new();
    let mut remaining = line;
    while let Some(start) = remaining.find("](") {
        let destination_start = start + 2;
        let Some(end) = remaining[destination_start..].find(')') else {
            break;
        };
        let end = destination_start + end;
        output.push_str(&remaining[..destination_start]);
        output.push_str(&link(&remaining[destination_start..end], source)?);
        output.push(')');
        remaining = &remaining[end + 1..];
    }
    output.push_str(remaining);
    Ok(output)
}

fn link(destination: &str, source: &Path) -> Result<String> {
    if destination.starts_with(['#', '/']) || destination.contains(':') {
        return Ok(destination.into());
    }
    let suffix = destination.find(['#', '?']).unwrap_or(destination.len());
    let mut resolved = PathBuf::new();
    for component in source
        .parent()
        .unwrap_or(Path::new(""))
        .join(&destination[..suffix])
        .components()
    {
        match component {
            Component::Normal(value) => resolved.push(value),
            Component::CurDir => {}
            Component::ParentDir if resolved.pop() => {}
            _ => {
                return Err(format!(
                    "{}: link escapes documentation: {destination}",
                    source.display()
                )
                .into());
            }
        }
    }
    let from = output_path(source);
    let target = output_path(&resolved);
    let from: Vec<_> = from
        .parent()
        .unwrap_or(Path::new(""))
        .components()
        .collect();
    let to: Vec<_> = target.components().collect();
    let shared = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let mut result = PathBuf::new();
    for _ in shared..from.len() {
        result.push("..");
    }
    for part in &to[shared..] {
        result.push(part.as_os_str());
    }
    Ok(format!(
        "{}{suffix}",
        result.display(),
        suffix = &destination[suffix..]
    ))
}

fn cells(line: &str) -> Vec<String> {
    let trimmed = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut code = 0;
    let mut chars = trimmed.chars().peekable();
    while let Some(value) = chars.next() {
        if value == '\\' {
            cell.push(value);
            if let Some(next) = chars.next() {
                cell.push(next);
            }
        } else if value == '`' {
            let mut width = 1;
            while chars.peek() == Some(&'`') {
                chars.next();
                width += 1;
            }
            if code == 0 {
                code = width;
            } else if code == width {
                code = 0;
            }
            cell.push_str(&"`".repeat(width));
        } else if value == '|' && code == 0 {
            cells.push(cell.trim().into());
            cell.clear();
        } else {
            cell.push(value);
        }
    }
    cells.push(cell.trim().into());
    cells
}

fn alignment(line: &str) -> bool {
    line.starts_with('|')
        && cells(line).iter().all(|cell| {
            let value = cell.trim_matches(':');
            value.len() >= 3 && value.chars().all(|character| character == '-')
        })
}

fn table_row(line: &str, source: &Path, header: bool) -> Result<String> {
    let mut result = String::from("@row{\n");
    for cell in cells(&rewrite_links(line, source)?) {
        result.push_str(if header {
            "@cell[header = true]{"
        } else {
            "@cell{"
        });
        result.push_str(&inline(&cell)?);
        result.push_str("}\n");
    }
    result.push_str("}\n");
    Ok(result)
}

fn inline(text: &str) -> Result<String> {
    let mut result = String::new();
    let mut remaining = text;
    while !remaining.is_empty() {
        let special = remaining.find(['`', '[']).unwrap_or(remaining.len());
        if special > 0 {
            result.push_str(&verbatim(&remaining[..special]));
            remaining = &remaining[special..];
            continue;
        }
        if remaining.starts_with('`') {
            let width = remaining.bytes().take_while(|byte| *byte == b'`').count();
            let marker = "`".repeat(width);
            if let Some(end) = remaining[width..].find(&marker) {
                let end = width + end;
                result.push_str(&format!("@code{{{}}}", verbatim(&remaining[width..end])));
                remaining = &remaining[end + width..];
                continue;
            }
        } else if let Some(close) = remaining.find("](")
            && let Some(end) = remaining[close + 2..].find(')')
        {
            let end = close + 2 + end;
            result.push_str(&format!(
                "@link[to = \"{}\"]{{{}}}",
                quoted(&remaining[close + 2..end]),
                verbatim(&remaining[1..close])
            ));
            remaining = &remaining[end + 1..];
            continue;
        }
        let width = remaining
            .chars()
            .next()
            .ok_or("Missing table character")?
            .len_utf8();
        result.push_str(&verbatim(&remaining[..width]));
        remaining = &remaining[width..];
    }
    Ok(result)
}

fn quoted(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}
fn verbatim(text: &str) -> String {
    format!(
        "@verbatim{{{}}}",
        text.replace("\\|", "|")
            .replace('\\', "\\\\")
            .replace('{', "\\{")
            .replace('}', "\\}")
    )
}

fn extend_tree(destination: &Path, files: &[PathBuf]) -> Result<()> {
    let tree_path = destination.join("index.sptree");
    let existing = fs::read_to_string(&tree_path)?;
    let mut tree = existing
        .trim_end()
        .strip_suffix('}')
        .ok_or("Missing book root")?
        .to_string();
    for (directory, title) in [
        ("", "Documentation"),
        ("how-to", "How-to guides"),
        ("reference", "Reference"),
        ("explanation", "Explanation"),
    ] {
        let opening = Path::new(directory).join("index.md");
        tree.push_str(&format!(
            "\n@part[id = \"{title}\", opening = \"{}\"]{{\n",
            page_id(&opening)
        ));
        for source in files {
            let selected = if directory.is_empty() {
                source == Path::new("tutorials/index.md")
            } else {
                source.starts_with(directory)
            };
            if selected
                && source != &opening
                && source.extension().is_some_and(|value| value == "md")
            {
                tree.push_str(&format!("@page[id = \"{}\"]\n", page_id(source)));
            }
        }
        tree.push_str("}\n");
    }
    tree.push_str("}\n");
    write(&tree_path, &tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Site URLs must stay within the publication across guide/support boundaries.
    #[test]
    fn local_routes() -> Result<()> {
        let cases = [
            (
                "tutorials/user-guide/org/index.md",
                "../../../how-to/capture-task.md",
                "../docs/how-to/capture-task.html",
            ),
            (
                "how-to/capture-task.md",
                "../tutorials/user-guide/index.md",
                "../../index.html",
            ),
            (
                "tutorials/user-guide/09-preferences.md",
                "../../reference/defaults.md#startup-configuration",
                "docs/reference/defaults.html#startup-configuration",
            ),
        ];
        for (source, target, expected) in cases {
            assert_eq!(
                link(target, Path::new(source))?,
                expected,
                "{source}: {target}"
            );
        }
        Ok(())
    }

    /// Projection cannot alter editor examples or drop canonical heading anchors.
    #[test]
    fn source_fidelity() -> Result<()> {
        let source =
            "# Example\n\n## Startup configuration\n\n```text\n[label](raw.md)\n| literal |\n```\n";
        let output = project(
            source,
            Path::new("reference/example.md"),
            &mut BTreeMap::new(),
        )?;
        assert!(anchors(source).contains(&"startup-configuration".into()));
        assert!(output.contains("```text\n[label](raw.md)\n| literal |\n```"));
        Ok(())
    }

    /// Escaped pipes and inline code must remain inside their original cells.
    #[test]
    fn table_cells() -> Result<()> {
        assert_eq!(
            cells("| `a|b` | one \\| two |"),
            vec!["`a|b`", "one \\| two"]
        );
        let mut fragments = BTreeMap::new();
        let output = project(
            "# Sample\n\n| Key | Description |\n| --- | --- |\n| `C-x` | [Save](save.md) |\n",
            Path::new("reference/example.md"),
            &mut fragments,
        )?;
        assert!(output.contains("@include["));
        let fragment = fragments.values().next().ok_or("Missing table")?;
        assert!(fragment.contains("@table{"));
        assert!(fragment.contains("@code{@verbatim{C-x}}"));
        assert!(fragment.contains("@link[to = \"save.html\"]"));
        Ok(())
    }

    /// Canonical aliases must coexist with native Spore heading identifiers.
    #[test]
    fn heading_aliases() -> Result<()> {
        let html = anchor_html(
            "<h1>Example</h1><h2 id=\"heading-1\">Details</h2>",
            &["example".into(), "details".into()],
        )?;
        assert!(html.contains("<span id=\"details\"></span><h2 id=\"heading-1\">"));
        assert!(anchor_html("<h1>Example</h1>", &[]).is_err());
        Ok(())
    }

    /// Instruction numbering and wrapped item text must survive Spore projection.
    #[test]
    fn numbered_steps() -> Result<()> {
        let mut fragments = BTreeMap::new();
        project(
            "# Example\n\n1. Open `notes.txt`.\n   Keep it open.\n2. Save.\n",
            Path::new("how-to/example.md"),
            &mut fragments,
        )?;
        let list = fragments.values().next().ok_or("Missing steps")?;
        assert!(list.starts_with("@list[ordered = true]"));
        assert_eq!(list.matches("@item{").count(), 2);
        assert!(list.contains("Keep it open."));
        Ok(())
    }
}
