use crate::support::Result;
use rev::context::*;
use std::fs;
use std::path::Path;

pub(crate) const DOCUMENT: &str = "first\r\n界e\u{301} line\r\nlast\r\n";

pub(crate) fn root(path: &Path) -> Result<Root> {
    fs::create_dir_all(path)?;
    Ok(Root {
        id: RootId::new()?,
        path: NativePath::absolute(path)?,
        label: Name::new(path.file_name().unwrap().to_str().unwrap())?,
        origin_project_id: None,
    })
}
pub(crate) fn project(catalog: &mut Catalog, path: &Path, name: &str) -> Result<()> {
    let id = ProjectId::new()?;
    let reference = SessionRef {
        owner: Owner::Project(id.clone()),
        revision: Revision::INITIAL,
    };
    catalog.projects.push(Project {
        id,
        name: Name::new(name)?,
        revision: Revision::INITIAL,
        root: root(path)?,
        roots_revision: Revision::INITIAL,
        session_ref: reference.clone(),
    });
    catalog.sessions.push(SavedSession::new(
        reference,
        Revision::INITIAL,
        Snapshot::default(),
    ));
    Ok(())
}
pub(crate) fn workspace(catalog: &mut Catalog, name: &str, roots: Vec<Root>) -> Result<()> {
    let id = WorkspaceId::new()?;
    let reference = SessionRef {
        owner: Owner::Workspace(id.clone()),
        revision: Revision::INITIAL,
    };
    catalog.workspaces.push(Workspace {
        id,
        name: Name::new(name)?,
        revision: Revision::INITIAL,
        roots,
        roots_revision: Revision::INITIAL,
        session_ref: reference.clone(),
    });
    catalog.sessions.push(SavedSession::new(
        reference,
        Revision::INITIAL,
        Snapshot::default(),
    ));
    Ok(())
}
fn entry(file: FileReference) -> FileEntry {
    FileEntry {
        file,
        cursor: SourcePosition { line: 2, column: 4 },
        viewport: Some(Viewport {
            anchor: SourcePosition { line: 1, column: 1 },
            horizontal_cells: 0,
        }),
    }
}
fn files(catalog: &mut Catalog, index: usize, count: usize, width: usize) -> Result<()> {
    let root = &catalog.projects[index].root;
    let mut snapshot = Snapshot::default();
    for file in 0..count {
        // Each component stays below the native filesystem component bound.
        // Long references are unavailable saved entries, not giant source files.
        let leaf = format!("file-{file:03}");
        let path = if width == 0 {
            fs::write(root.path.as_path().join(&leaf), DOCUMENT)?;
            leaf
        } else {
            let mut prefix = String::new();
            while width - prefix.len() - leaf.len() > 240 {
                prefix.push_str(&"x".repeat(239));
                prefix.push('/');
            }
            let padding = width - prefix.len() - leaf.len();
            if padding > 1 {
                prefix.push_str(&"x".repeat(padding - 1));
                prefix.push('/');
            } else {
                prefix.push_str(&"x".repeat(padding));
            }
            prefix.push_str(&leaf);
            assert_eq!(prefix.len(), width);
            prefix
        };
        snapshot.files.push(entry(FileReference::RootRelative {
            root: root.id.clone(),
            path: NativePath::relative(Path::new(&path))?,
        }));
    }
    snapshot.active_file = snapshot.files.last().map(|f| f.file.clone());
    catalog.sessions[index].snapshot = Ok(snapshot);
    Ok(())
}

pub(crate) fn ordinary(directory: &Path) -> Result<Catalog> {
    let mut catalog = Catalog::default();
    for i in 0..32 {
        project(
            &mut catalog,
            &directory.join(format!("p{i:03}")),
            &format!("P{i:03}"),
        )?;
    }
    for i in 0..18 {
        workspace(
            &mut catalog,
            &format!("W{i:03}"),
            vec![root(&directory.join(format!("w{i:03}")))?],
        )?;
    }
    for session in &mut catalog.sessions {
        let mut snapshot = Snapshot::default();
        for i in 0..20 {
            snapshot
                .files
                .push(entry(FileReference::LooseAbsolute(NativePath::absolute(
                    &directory.join(format!("unavailable-{i:03}-{}", "x".repeat(90))),
                )?)));
        }
        snapshot.active_file = snapshot.files.first().map(|v| v.file.clone());
        session.snapshot = Ok(snapshot);
    }
    Ok(catalog)
}

// One count-bound catalog also approaches the aggregate byte ceiling. All four
// large snapshots independently fit 2 MiB; their combined size admits no fifth.
pub(crate) fn boundary(directory: &Path) -> Result<Catalog> {
    let mut catalog = Catalog::default();
    boundary_projects(&mut catalog, directory)?;
    boundary_workspaces(&mut catalog, directory)?;
    files(&mut catalog, 0, MAX_FILES, 0)?;
    for index in 1..=4 {
        files(&mut catalog, index, MAX_FILES, 1000)?;
    }
    let measured = catalog.encode()?.len();
    let target = MAX_CATALOG_BYTES - 32768;
    assert!(measured < target);
    let width = 1000 + (target - measured) / (4 * MAX_FILES);
    for index in 1..=4 {
        files(&mut catalog, index, MAX_FILES, width)?;
    }
    let bytes = catalog.encode()?.len();
    assert!((MAX_CATALOG_BYTES - 65536..=MAX_CATALOG_BYTES).contains(&bytes));
    Ok(catalog)
}

pub(crate) fn overflow(catalog: &Catalog) -> Result<()> {
    let mut candidate = catalog.clone();
    let mut snapshot = candidate.sessions[1].snapshot.as_ref().unwrap().clone();
    for entry in &mut snapshot.files {
        if let FileReference::RootRelative { root, .. } = &mut entry.file {
            *root = candidate.projects[5].root.id.clone();
        }
    }
    snapshot.active_file = snapshot.files.last().map(|f| f.file.clone());
    candidate.sessions[5].snapshot = Ok(snapshot);
    assert!(matches!(
        candidate.encode(),
        Err(Error::Capacity("encoded catalog"))
    ));
    Ok(())
}

// Isolate the per-session byte ceiling from the aggregate/count boundary. Use
// the parsed native table shape that the codec charges, including path encoding.
pub(crate) fn session_boundary(directory: &Path) -> Result<(Catalog, usize)> {
    let mut catalog = Catalog::default();
    project(&mut catalog, &directory.join("root"), "Large snapshot")?;
    files(&mut catalog, 0, MAX_FILES, 1000)?;
    let measured = catalog.encode()?.len();
    let width = 1000 + (MAX_SESSION_BYTES - 2048 - measured) / MAX_FILES;
    files(&mut catalog, 0, MAX_FILES, width)?;
    let bytes = catalog.encode()?;
    let document: toml_edit::DocumentMut = std::str::from_utf8(&bytes)?.parse()?;
    let charged = document["sessions"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .next()
        .unwrap()
        .to_string()
        .len();
    assert!((MAX_SESSION_BYTES - 4096..=MAX_SESSION_BYTES).contains(&charged));
    let mut rejected = catalog.clone();
    files(&mut rejected, 0, MAX_FILES, width + 16)?;
    assert!(matches!(
        rejected.encode(),
        Err(Error::Capacity("encoded session"))
    ));
    assert_eq!(catalog.encode()?, bytes);
    Ok((catalog, charged))
}

fn boundary_projects(catalog: &mut Catalog, directory: &Path) -> Result<()> {
    for i in 0..MAX_PROJECTS {
        let name = if i == 0 {
            format!("P000{}", "x".repeat(MAX_NAME_BYTES - 4))
        } else {
            format!("P{i:03}")
        };
        project(catalog, &directory.join(format!("p{i:03}")), &name)?;
    }
    Ok(())
}

fn boundary_workspaces(catalog: &mut Catalog, directory: &Path) -> Result<()> {
    for i in 0..MAX_WORKSPACES {
        let roots = if i == 0 {
            (0..MAX_ROOTS)
                .map(|n| root(&directory.join(format!("r{n:03}"))))
                .collect::<Result<Vec<_>>>()?
        } else {
            vec![]
        };
        workspace(catalog, &format!("W{i:03}"), roots)?;
    }
    Ok(())
}
