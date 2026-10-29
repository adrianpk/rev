use super::*;
use crate::org::attachment::directory::Directory;
use std::{fs, path::Path};
pub(super) struct Entry {
    pub path: PathBuf,
    pub relative: PathBuf,
    pub asset: bool,
    pub error: Option<String>,
}
struct Walk<'a> {
    selection: &'a Selection,
    budget: &'a Budget<'a>,
    entries: usize,
    files: Vec<Entry>,
    limit: usize,
}
pub(super) fn scan(
    selection: &Selection,
    limits: &config::OrgLimits,
    budget: &Budget<'_>,
) -> Result<Vec<Entry>, String> {
    Directory::observe(&selection.output)?;
    let mut walk = Walk {
        selection,
        budget,
        entries: 0,
        files: Vec::new(),
        limit: limits.agenda_files,
    };
    for root in &selection.roots {
        walk.root(root)?;
    }
    walk.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(walk.files)
}
impl Walk<'_> {
    fn root(&mut self, root: &Path) -> Result<(), String> {
        self.budget.check()?;
        if root.starts_with(&self.selection.output) {
            return Err("Project source roots cannot be inside the output directory".into());
        }
        match fs::symlink_metadata(root) {
            Ok(m) if m.is_dir() => self.directory(root),
            Ok(_) => self.selected(
                root,
                root.file_name().map(PathBuf::from).unwrap_or_default(),
            ),
            Err(e) => self.push(Entry {
                path: root.into(),
                relative: root.file_name().map(PathBuf::from).unwrap_or_default(),
                asset: false,
                error: Some(e.to_string()),
            }),
        }
    }
    fn directory(&mut self, root: &Path) -> Result<(), String> {
        Directory::observe(root)?.open()?;
        let mut directories = vec![(
            root.to_owned(),
            fs::read_dir(root).map_err(|e| e.to_string())?,
        )];
        while let Some((parent, read)) = directories.last_mut() {
            self.boundary()?;
            let Some(entry) = read.next() else {
                directories.pop();
                continue;
            };
            let parent = parent.clone();
            self.child(root, &parent, entry, &mut directories)?;
        }
        Ok(())
    }
    fn child(
        &mut self,
        root: &Path,
        parent: &Path,
        entry: std::io::Result<fs::DirEntry>,
        directories: &mut Vec<(PathBuf, fs::ReadDir)>,
    ) -> Result<(), String> {
        let entry = entry.map_err(|e| format!("Project directory {}: {e}", parent.display()))?;
        let path = entry.path();
        if path.starts_with(&self.selection.output) {
            return Ok(());
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_owned();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            self.descend(&path, directories)
        } else {
            self.selected(&path, relative)
        }
    }
    fn descend(
        &self,
        path: &Path,
        directories: &mut Vec<(PathBuf, fs::ReadDir)>,
    ) -> Result<(), String> {
        if directories.len() >= 64 {
            return Err("Project directory depth exceeds 64".into());
        }
        Directory::observe(path)?.open()?;
        directories.push((path.into(), fs::read_dir(path).map_err(|e| e.to_string())?));
        Ok(())
    }
    fn boundary(&mut self) -> Result<(), String> {
        self.budget.check()?;
        self.entries += 1;
        if self.entries > 65536 {
            return Err("Project discovery exceeds 65536 entries".into());
        }
        Ok(())
    }
    fn selected(&mut self, path: &Path, relative: PathBuf) -> Result<(), String> {
        let name = relative.to_str().ok_or("Project paths require UTF-8")?;
        if name.len() > 4096 {
            return Err("Project relative path exceeds 4096 bytes".into());
        }
        let project = &self.selection.project;
        if matches(&project.exclude, name) {
            return Ok(());
        }
        let asset = matches(&project.assets, name);
        let document =
            path.extension().is_some_and(|e| e == "org") && matches(&project.include, name);
        if !asset && !document {
            return Ok(());
        }
        let error = if asset && document {
            Some("File selects both document export and asset copy".into())
        } else {
            regular(path).err()
        };
        self.push(Entry {
            path: path.into(),
            relative,
            asset,
            error,
        })
    }
    fn push(&mut self, entry: Entry) -> Result<(), String> {
        if self.files.len() >= self.limit {
            return Err("Project source count exceeds org.limits.agenda_files".into());
        }
        self.files.push(entry);
        Ok(())
    }
}
pub(super) fn regular(path: &Path) -> Result<(), String> {
    Directory::observe(path.parent().ok_or("Project source has no parent")?)?.open()?;
    if !fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .is_file()
    {
        return Err("Project source is a symlink or non-regular file".into());
    }
    Ok(())
}
fn matches(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| glob(p, name))
}
fn glob(pattern: &str, name: &str) -> bool {
    let names: Vec<_> = name.split('/').collect();
    let mut active = vec![false; names.len() + 1];
    active[0] = true;
    for part in pattern.split('/') {
        let mut next = vec![false; names.len() + 1];
        for index in 0..=names.len() {
            next[index] = glob_step(part, index, &names, (&active, &next));
        }
        active = next;
    }
    active[names.len()]
}
fn glob_step(part: &str, index: usize, names: &[&str], states: (&[bool], &[bool])) -> bool {
    let (active, next) = states;
    if part == "**" {
        return active[index] || (index > 0 && next[index - 1]);
    }
    index > 0 && active[index - 1] && component(part, names[index - 1])
}
fn component(pattern: &str, name: &str) -> bool {
    let chars: Vec<_> = name.chars().collect();
    let mut active = vec![false; chars.len() + 1];
    active[0] = true;
    for p in pattern.chars() {
        let mut next = vec![false; chars.len() + 1];
        next[0] = p == '*' && active[0];
        for index in 1..=chars.len() {
            next[index] = if p == '*' {
                active[index] || next[index - 1]
            } else {
                active[index - 1] && (p == '?' || p == chars[index - 1])
            };
        }
        active = next;
    }
    active[chars.len()]
}
