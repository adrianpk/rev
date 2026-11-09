use super::{Result, arguments::Input};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    target_directory: PathBuf,
    workspace_members: Vec<String>,
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    manifest_path: PathBuf,
    targets: Vec<Target>,
}

#[derive(Deserialize)]
struct Target {
    src_path: PathBuf,
}

pub(super) struct Workspace {
    pub root: PathBuf,
    pub packages: BTreeSet<String>,
    pub sources: BTreeMap<PathBuf, bool>,
}

impl Workspace {
    pub fn discover() -> Result<Self> {
        Self::at(&std::env::current_dir()?)
    }

    pub fn at(root: &Path) -> Result<Self> {
        let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["metadata", "--locked", "--no-deps", "--format-version", "1"])
            .current_dir(root)
            .output()?;
        if !output.status.success() {
            return Err(format!(
                "Cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Self::from_metadata(serde_json::from_slice(&output.stdout)?)
    }

    fn from_metadata(metadata: Metadata) -> Result<Self> {
        let mut workspace = Self {
            root: metadata.workspace_root.canonicalize()?,
            packages: metadata.workspace_members.into_iter().collect(),
            sources: BTreeMap::new(),
        };
        for package in metadata.packages {
            if workspace.packages.contains(&package.id) {
                workspace.member(&package, &metadata.target_directory)?;
            }
        }
        if workspace.sources.is_empty() {
            return Err("Cargo workspace contains no first-party Rust sources".into());
        }
        Ok(workspace)
    }

    fn member(&mut self, package: &Package, target: &Path) -> Result<()> {
        let directory = package
            .manifest_path
            .parent()
            .ok_or("manifest has no parent")?
            .canonicalize()?;
        let mut paths = BTreeSet::new();
        collect(&directory, &directory, target, &mut paths)?;
        for path in paths {
            self.sources.entry(path).or_insert(false);
        }
        for target in &package.targets {
            let root = target.src_path.canonicalize()?;
            if !self.sources.contains_key(&root) {
                return Err(format!(
                    "Cargo target is outside its first-party source owner: {}",
                    root.display()
                )
                .into());
            }
            self.sources.insert(root, true);
        }
        Ok(())
    }

    pub fn select(&self, scopes: &[PathBuf]) -> Result<Vec<Input>> {
        let scopes = self.scopes(scopes)?;
        Ok(self
            .sources
            .iter()
            .filter(|(path, _)| {
                scopes.is_empty() || scopes.iter().any(|scope| path.starts_with(scope))
            })
            .map(|(path, root)| Input {
                path: path.to_owned(),
                root: *root,
            })
            .collect())
    }

    fn scopes(&self, paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
        paths.iter().map(|path| self.scope(path)).collect()
    }

    fn scope(&self, path: &Path) -> Result<PathBuf> {
        if path.as_os_str().is_empty() {
            return Err("empty source scope".into());
        }
        let path = self.root.join(path).canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err("source scope is outside the Cargo workspace".into());
        }
        if !self.sources.keys().any(|source| source.starts_with(&path)) {
            return Err(format!(
                "scope contains no first-party Rust sources: {}",
                path.display()
            )
            .into());
        }
        Ok(path)
    }

    pub fn display<'a>(&self, path: &'a Path) -> &'a Path {
        path.strip_prefix(&self.root).unwrap_or(path)
    }
}

fn collect(
    directory: &Path,
    owner: &Path,
    target: &Path,
    paths: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() && admitted_directory(&path, owner, target) {
            collect(&path, owner, target, paths)?;
        } else {
            collect_file(&path, kind, paths)?;
        }
    }
    Ok(())
}

fn admitted_directory(path: &Path, owner: &Path, target: &Path) -> bool {
    if path.starts_with(target) || path.file_name().is_some_and(|name| name == ".git") {
        return false;
    }
    path == owner || !path.join("Cargo.toml").exists()
}

fn collect_file(path: &Path, kind: fs::FileType, paths: &mut BTreeSet<PathBuf>) -> Result<()> {
    if kind.is_file() && path.extension().is_some_and(|extension| extension == "rs") {
        paths.insert(path.canonicalize()?);
    }
    Ok(())
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
