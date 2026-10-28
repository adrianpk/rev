use std::{
    collections::{BTreeSet, VecDeque},
    fs::{self, ReadDir},
    path::PathBuf,
};
/// Directory iteration is streaming and non-recursive. Entry limits cover even
/// ignored names so a directory with no .org files cannot consume endless work.
pub(super) enum Step {
    Path(PathBuf),
    Progress,
    End,
}
pub(super) struct Walk {
    roots: VecDeque<PathBuf>,
    directory: Option<(PathBuf, ReadDir)>,
    seen: BTreeSet<PathBuf>,
    entries: usize,
}
impl Walk {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            roots: roots.into(),
            directory: None,
            seen: BTreeSet::new(),
            entries: 0,
        }
    }
    pub fn next(&mut self) -> Result<Step, (Option<PathBuf>, String)> {
        if self.directory.is_some() {
            return self.child();
        }
        let Some(path) = self.roots.pop_front() else {
            return Ok(Step::End);
        };
        match fs::metadata(&path) {
            Ok(m) if m.is_dir() => self.directory(path),
            _ => Ok(Step::Path(path)),
        }
    }
    fn directory(&mut self, path: PathBuf) -> Result<Step, (Option<PathBuf>, String)> {
        let canonical = fs::canonicalize(&path).map_err(|e| (Some(path.clone()), e.to_string()))?;
        if self.seen.insert(canonical) {
            let read = fs::read_dir(&path).map_err(|e| (Some(path.clone()), e.to_string()))?;
            self.directory = Some((path.clone(), read));
        }
        // A directory or ignored child is a progress boundary, never end-of-walk.
        Ok(Step::Progress)
    }
    fn child(&mut self) -> Result<Step, (Option<PathBuf>, String)> {
        let (path, read) = self.directory.as_mut().expect("directory iteration owned");
        let Some(entry) = read.next() else {
            self.directory = None;
            return Ok(Step::Progress);
        };
        self.entries += 1;
        if self.entries > 65536 {
            self.directory = None;
            self.roots.clear();
            return Err((None, "Agenda directory entry limit 65536 reached".into()));
        }
        let entry = entry.map_err(|e| (Some(path.clone()), e.to_string()))?;
        let child = entry.path();
        let kind = entry
            .file_type()
            .map_err(|e| (Some(child.clone()), e.to_string()))?;
        if child.extension().is_some_and(|e| e == "org") && !kind.is_dir() {
            return Ok(Step::Path(child));
        }
        Ok(Step::Progress)
    }
}
