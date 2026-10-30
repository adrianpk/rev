#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const EDITOR: &[&str] = &[
    "01-first-file.md",
    "02-edit-text.md",
    "03-buffers-and-panes.md",
    "04-search-and-commands.md",
    "05-file-tree.md",
    "09-preferences.md",
];
const ORG: &[&str] = &[
    "06-org-notes.md",
    "07-org-tasks-and-agenda.md",
    "08-org-tables.md",
];

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = markdown_files(&root.join("docs"))?;
    let count = paths.len();
    paths.extend([root.join("README.md"), root.join("CHANGELOG.md")]);
    let mut graph = BTreeMap::new();
    let mut links = 0;
    for path in paths {
        let source = fs::read_to_string(&path)?;
        let targets = check_links(&path, &source)?;
        links += targets.len();
        graph.insert(path.canonicalize()?, targets);
    }
    check_reachability(&graph, &root.join("docs/index.md").canonicalize()?)?;
    let guide = root.join("docs/tutorials/user-guide");
    check_path(&guide, "index.md", EDITOR)?;
    check_path(&guide, "org/index.md", ORG)?;
    println!(
        "Documentation passed: {count} reachable pages, {links} local links and anchors, independent editor and Org learning paths."
    );
    Ok(())
}

fn require(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

fn markdown_files(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut pending = vec![directory.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            match (kind.is_dir(), kind.is_file()) {
                (true, _) => pending.push(entry.path()),
                (_, true) if entry.path().extension().is_some_and(|ext| ext == "md") => {
                    paths.push(entry.path());
                }
                _ => {}
            }
        }
    }
    paths.sort();
    Ok(paths)
}

fn fence(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start_matches(' ');
    if line.len() - trimmed.len() > 3 {
        return None;
    }
    let marker = trimmed.chars().next()?;
    if !matches!(marker, '`' | '~') {
        return None;
    }
    let length = trimmed.chars().take_while(|ch| *ch == marker).count();
    (length >= 3).then_some((marker, length, &trimmed[length..]))
}

fn prose(source: &str) -> String {
    let mut active = None;
    let mut result = String::new();
    for line in source.lines() {
        match (active, fence(line)) {
            (None, Some((marker, length, _))) => active = Some((marker, length)),
            (Some((marker, length)), Some((end, count, rest)))
                if marker == end && count >= length && rest.trim().is_empty() =>
            {
                active = None;
            }
            (None, None) => {
                result.push_str(line);
                result.push('\n');
            }
            _ => {}
        }
    }
    result
}

fn without_code(source: &str) -> String {
    let mut rest = source;
    let mut result = String::new();
    while let Some(start) = rest.find('`') {
        result.push_str(&rest[..start]);
        rest = &rest[start..];
        let length = rest.bytes().take_while(|byte| *byte == b'`').count();
        let marker = &rest[..length];
        let Some(end) = rest[length..].find(marker) else {
            result.push_str(rest);
            return result;
        };
        rest = &rest[length + end + length..];
    }
    result.push_str(rest);
    result
}

fn link_targets(source: &str) -> Vec<String> {
    let text = without_code(&prose(source));
    let mut rest = text.as_str();
    let mut links = Vec::new();
    while let Some(start) = rest.find("](") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find(')') else {
            break;
        };
        links.push(rest[..end].to_owned());
        rest = &rest[end + 1..];
    }
    links
}

fn anchors(source: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    for line in prose(source).lines() {
        let title = line.trim_start_matches('#');
        let depth = line.len() - title.len();
        if !(1..=6).contains(&depth) || !title.starts_with(' ') {
            continue;
        }
        let slug = title.trim().trim_end_matches('#').trim().to_lowercase();
        let slug: String = slug
            .chars()
            .filter(|ch| ch.is_alphanumeric() || "_ -".contains(*ch))
            .map(|ch| if ch == ' ' { '-' } else { ch })
            .collect();
        let mut unique = slug.clone();
        let mut suffix = 0;
        while result.contains(&unique) {
            suffix += 1;
            unique = format!("{slug}-{suffix}");
        }
        result.insert(unique);
    }
    result
}

fn check_links(path: &Path, source: &str) -> Result<Vec<PathBuf>> {
    let mut local = Vec::new();
    for target in link_targets(source) {
        if target.contains("://") || target.starts_with("mailto:") {
            continue;
        }
        local.push(local_target(path, &target)?);
    }
    Ok(local)
}

fn local_target(path: &Path, target: &str) -> Result<PathBuf> {
    let (name, anchor) = target.split_once('#').unwrap_or((target, ""));
    let destination = if name.is_empty() {
        path.to_path_buf()
    } else {
        path.parent().ok_or("missing document parent")?.join(name)
    };
    require(
        destination.is_file(),
        format!("{}: missing target {target}", path.display()),
    )?;
    if !anchor.is_empty() {
        require(
            anchors(&fs::read_to_string(&destination)?).contains(anchor),
            format!("{}: missing anchor {target}", path.display()),
        )?;
    }
    Ok(destination.canonicalize()?)
}

fn check_reachability(graph: &BTreeMap<PathBuf, Vec<PathBuf>>, start: &Path) -> Result<()> {
    let mut pending = vec![start.to_path_buf()];
    let mut seen = BTreeSet::new();
    while let Some(path) = pending.pop() {
        if seen.insert(path.clone()) {
            pending.extend(graph.get(&path).into_iter().flatten().cloned());
        }
    }
    let missing: Vec<_> = graph
        .keys()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name != "README.md" && name != "CHANGELOG.md")
        })
        .filter(|path| !seen.contains(*path))
        .collect();
    require(
        missing.is_empty(),
        format!("unreachable documentation: {missing:?}"),
    )
}

fn navigation(source: &str, prefix: &str) -> Vec<String> {
    source
        .lines()
        .filter(|line| line.starts_with(prefix))
        .flat_map(link_targets)
        .take(1)
        .collect()
}

fn check_path(guide: &Path, index: &str, chapters: &[&str]) -> Result<()> {
    let path = guide.join(index);
    let source = fs::read_to_string(&path)?;
    let parent = path.parent().ok_or("missing index parent")?;
    let actual: Vec<_> = source
        .lines()
        .filter(|line| line.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
        .flat_map(link_targets)
        .map(|target| parent.join(target).canonicalize())
        .collect::<std::io::Result<_>>()?;
    let expected: Vec<_> = chapters
        .iter()
        .map(|name| guide.join(name).canonicalize())
        .collect::<std::io::Result<_>>()?;
    require(
        actual == expected,
        format!("{index}: incorrect learning sequence"),
    )?;
    for (position, name) in chapters.iter().enumerate() {
        check_chapter(
            &guide.join(name),
            index,
            position.checked_sub(1).map(|pos| chapters[pos]),
            chapters.get(position + 1).copied(),
        )?;
    }
    Ok(())
}

fn check_chapter(
    path: &Path,
    index: &str,
    previous: Option<&str>,
    next: Option<&str>,
) -> Result<()> {
    let source = fs::read_to_string(path)?;
    for (prefix, expected) in [("[Previous]", previous), ("Next:", next)] {
        let expected: Vec<_> = expected.into_iter().map(str::to_owned).collect();
        require(
            navigation(&source, prefix) == expected,
            format!("{}: incorrect {prefix} chapter", path.display()),
        )?;
    }
    require(
        link_targets(&source).contains(&index.to_owned()),
        format!("{}: missing guide return", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Literal Markdown examples cannot create links or headings; real linked
    // labels may contain code spans and duplicate headings need distinct anchors.
    #[test]
    fn markdown_literals() {
        let source = "# Real `title`\n# Real title\n```md\n[no](missing)\n# Hidden\n```\n~~~\n[no](also-missing)\n~~~\n`[no](inline)`\n``[no](more)``\n[Use `save`](page.md#real-title)";
        assert_eq!(link_targets(source), ["page.md#real-title"]);
        assert_eq!(
            anchors(source),
            BTreeSet::from(["real-title".into(), "real-title-1".into()])
        );
    }

    // Cycles are allowed, but an unlinked page must fail even when its outgoing
    // link points back to the index. This protects navigation after extraction.
    #[test]
    fn orphan_rejection() {
        for (name, reachable) in [("linked", true), ("orphan", false)] {
            let index = PathBuf::from("index.md");
            let page = PathBuf::from("page.md");
            let links = if reachable {
                vec![page.clone()]
            } else {
                Vec::new()
            };
            let graph = BTreeMap::from([(index.clone(), links), (page, vec![index.clone()])]);
            assert_eq!(
                check_reachability(&graph, &index).is_ok(),
                reachable,
                "{name}"
            );
        }
    }

    // Existing files are insufficient: deleted targets and stale heading
    // fragments must fail through the same resolver used for real documents.
    #[test]
    fn target_rejection() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/index.md");
        for (name, source, valid) in [
            ("existing heading", "[docs](#documentation)", true),
            (
                "missing heading",
                "[docs](#missing-documentation-anchor)",
                false,
            ),
            (
                "missing file",
                "[docs](missing-documentation-page.md)",
                false,
            ),
            ("external", "[site](https://example.org/page)", true),
        ] {
            assert_eq!(check_links(&path, source).is_ok(), valid, "{name}");
        }
    }

    // Navigation must reject a jump into the other learning path, including
    // an extra next chapter after the documented end of the Org sequence.
    #[test]
    fn chapter_navigation() {
        let guide = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/tutorials/user-guide");
        for (name, chapter, index, next, valid) in [
            (
                "editor next",
                "05-file-tree.md",
                "index.md",
                Some("09-preferences.md"),
                true,
            ),
            (
                "crossed editor next",
                "05-file-tree.md",
                "index.md",
                Some("06-org-notes.md"),
                false,
            ),
            ("Org end", "08-org-tables.md", "org/index.md", None, true),
            (
                "crossed Org next",
                "08-org-tables.md",
                "org/index.md",
                Some("09-preferences.md"),
                false,
            ),
        ] {
            let previous = if chapter == "05-file-tree.md" {
                "04-search-and-commands.md"
            } else {
                "07-org-tasks-and-agenda.md"
            };
            assert_eq!(
                check_chapter(&guide.join(chapter), index, Some(previous), next).is_ok(),
                valid,
                "{name}"
            );
        }
    }
}
