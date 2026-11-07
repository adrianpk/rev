use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use serde::Deserialize;
use toml_edit::{Array, ArrayOfTables, DocumentMut, InlineTable, Item, Table, Value, value};

use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    revision: Revision,
    #[serde(default)]
    projects: Vec<ProjectRecord>,
    #[serde(default)]
    workspaces: Vec<WorkspaceRecord>,
    #[serde(default)]
    sessions: Vec<toml::Table>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectRecord {
    id: ProjectId,
    revision: Revision,
    name: Name,
    roots_revision: Revision,
    root: RootRecord,
    session_revision: Revision,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceRecord {
    id: WorkspaceId,
    revision: Revision,
    name: Name,
    roots_revision: Revision,
    roots: Vec<RootRecord>,
    session_revision: Revision,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RootRecord {
    id: RootId,
    label: Name,
    location: PathRecord,
    origin_project_id: Option<ProjectId>,
}
// Same mutually exclusive UTF-8/native-byte representation as bookmarks.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PathRecord {
    path: Option<String>,
    path_bytes: Option<Vec<u8>>,
}
impl PathRecord {
    fn decode(self, absolute: bool) -> Result<NativePath, Error> {
        let path = match (self.path, self.path_bytes) {
            (Some(path), None) => PathBuf::from(path),
            (None, Some(bytes)) => PathBuf::from(OsString::from_vec(bytes)),
            _ => return Err(Error::Invalid("path encoding")),
        };
        if absolute {
            NativePath::absolute(&path)
        } else {
            NativePath::relative(&path)
        }
    }
}
impl RootRecord {
    fn decode(self) -> Result<Root, Error> {
        Ok(Root {
            id: self.id,
            path: self.location.decode(true)?,
            label: self.label,
            origin_project_id: self.origin_project_id,
        })
    }
}
#[derive(Deserialize)]
struct SessionHeader {
    owner: Owner,
    revision: Revision,
    roots_revision: Revision,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionRecord {
    schema_version: u32,
    owner: Owner,
    revision: Revision,
    roots_revision: Revision,
    files: Vec<FileRecord>,
    active_file: Option<usize>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {
    root: Option<RootId>,
    location: PathRecord,
    cursor: SourcePosition,
    viewport: Option<Viewport>,
}

pub(super) fn decode(bytes: &[u8]) -> Result<Catalog, Error> {
    if bytes.len() > MAX_CATALOG_BYTES {
        return Err(Error::Capacity("encoded catalog"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid("TOML encoding"))?;
    let wire: Document = toml::from_str(text).map_err(|_| Error::Invalid("catalog TOML"))?;
    if wire.schema_version != 1 {
        return Err(Error::Unsupported("catalog schema"));
    }
    if wire.projects.len() > MAX_PROJECTS
        || wire.workspaces.len() > MAX_WORKSPACES
        || wire.sessions.len() > MAX_PROJECTS + MAX_WORKSPACES
    {
        return Err(Error::Capacity("catalog records"));
    }
    let raw: DocumentMut = text.parse().map_err(|_| Error::Invalid("catalog TOML"))?;
    let mut catalog = Catalog {
        revision: wire.revision,
        ..Catalog::default()
    };
    for project in wire.projects {
        let owner = Owner::Project(project.id.clone());
        catalog.projects.push(Project {
            id: project.id,
            revision: project.revision,
            name: project.name,
            roots_revision: project.roots_revision,
            root: project.root.decode()?,
            session_ref: SessionRef {
                owner,
                revision: project.session_revision,
            },
        });
    }
    for workspace in wire.workspaces {
        if workspace.roots.len() > MAX_ROOTS {
            return Err(Error::Capacity("Workspace roots"));
        }
        let owner = Owner::Workspace(workspace.id.clone());
        catalog.workspaces.push(Workspace {
            id: workspace.id,
            revision: workspace.revision,
            name: workspace.name,
            roots_revision: workspace.roots_revision,
            roots: workspace
                .roots
                .into_iter()
                .map(RootRecord::decode)
                .collect::<Result<_, _>>()?,
            session_ref: SessionRef {
                owner,
                revision: workspace.session_revision,
            },
        });
    }
    for (index, table) in wire.sessions.into_iter().enumerate() {
        let header: SessionHeader = table
            .clone()
            .try_into()
            .map_err(|_| Error::Invalid("session identity"))?;
        let raw = raw
            .get("sessions")
            .and_then(Item::as_array_of_tables)
            .and_then(|tables| tables.get(index))
            .ok_or(Error::Invalid("session table shape"))?
            .clone();
        let roots = match &header.owner {
            Owner::Project(id) => catalog
                .projects
                .iter()
                .find(|p| p.id == *id)
                .map(|p| std::slice::from_ref(&p.root)),
            Owner::Workspace(id) => catalog
                .workspaces
                .iter()
                .find(|w| w.id == *id)
                .map(|w| w.roots.as_slice()),
        }
        .ok_or(Error::Invalid("orphan session"))?;
        let snapshot = if raw.to_string().len() > MAX_SESSION_BYTES {
            return Err(Error::Capacity("encoded session"));
        } else {
            decode_session(table, roots)
        };
        let preserved = snapshot.is_err().then_some(raw);
        catalog.sessions.push(SavedSession {
            reference: SessionRef {
                owner: header.owner,
                revision: header.revision,
            },
            roots_revision: header.roots_revision,
            snapshot,
            preserved,
        });
    }
    catalog.validate(false)?;
    Ok(catalog)
}
fn decode_session(table: toml::Table, roots: &[Root]) -> Result<Snapshot, Error> {
    let wire: SessionRecord = table
        .try_into()
        .map_err(|_| Error::Invalid("session payload"))?;
    if wire.schema_version != 1 {
        return Err(Error::Unsupported("session schema"));
    }
    // Typed ownership headers are consumed separately, so malformed payloads can
    // be diagnosed without hiding otherwise valid registrations.
    let _ = (wire.owner, wire.revision, wire.roots_revision);
    if wire.files.len() > MAX_FILES {
        return Err(Error::Capacity("session files"));
    }
    let mut files = Vec::with_capacity(wire.files.len());
    for entry in wire.files {
        let file = match entry.root {
            Some(root) => FileReference::RootRelative {
                root,
                path: entry.location.decode(false)?,
            },
            None => FileReference::LooseAbsolute(entry.location.decode(true)?),
        };
        files.push(FileEntry {
            file,
            cursor: entry.cursor,
            viewport: entry.viewport,
        });
    }
    let active_file = wire
        .active_file
        .map(|index| {
            files
                .get(index)
                .map(|entry| entry.file.clone())
                .ok_or(Error::Invalid("active session file"))
        })
        .transpose()?;
    let snapshot = Snapshot { files, active_file };
    super::catalog::validate_snapshot(&snapshot, roots, false)?;
    Ok(snapshot)
}

pub(super) fn encode(catalog: &Catalog) -> Result<Vec<u8>, Error> {
    catalog.validate(false)?;
    let mut document = DocumentMut::new();
    document["schema_version"] = value(1);
    document["revision"] = value(catalog.revision.get() as i64);
    let mut charged = 64;
    let projects = catalog.projects.iter().map(|project| {
        let mut table = record(
            project.id.as_str(),
            project.revision,
            &project.name,
            project.roots_revision,
            project.session_ref.revision,
        );
        table["root"] = Item::Value(root_value(&project.root));
        table
    });
    tables(&mut document, "projects", projects, &mut charged)?;
    let workspaces = catalog.workspaces.iter().map(|workspace| {
        let mut table = record(
            workspace.id.as_str(),
            workspace.revision,
            &workspace.name,
            workspace.roots_revision,
            workspace.session_ref.revision,
        );
        let mut roots = Array::new();
        for root in &workspace.roots {
            roots.push(root_value(root));
        }
        table["roots"] = value(roots);
        table
    });
    tables(&mut document, "workspaces", workspaces, &mut charged)?;
    let mut sessions = Vec::with_capacity(catalog.sessions.len());
    for session in &catalog.sessions {
        let table = match &session.snapshot {
            Ok(snapshot) => session_table(session, snapshot)?,
            Err(_) => session
                .preserved
                .clone()
                .ok_or(Error::Invalid("damaged session"))?,
        };
        if table.to_string().len() > MAX_SESSION_BYTES {
            return Err(Error::Capacity("encoded session"));
        }
        // Charge before retaining another table, rather than materializing all
        // individually admitted snapshots before applying the aggregate cap.
        charge(&table, &mut charged)?;
        sessions.push(table);
    }
    let mut array = ArrayOfTables::new();
    for session in sessions {
        array.push(session);
    }
    if array.is_empty() {
        document["sessions"] = value(Array::new());
    } else {
        document["sessions"] = Item::ArrayOfTables(array);
    }
    let bytes = document.to_string().into_bytes();
    if bytes.len() > MAX_CATALOG_BYTES {
        return Err(Error::Capacity("encoded catalog"));
    }
    Ok(bytes)
}
fn charge(table: &Table, charged: &mut usize) -> Result<(), Error> {
    *charged += table.to_string().len() + 32;
    if *charged > MAX_CATALOG_BYTES {
        return Err(Error::Capacity("encoded catalog"));
    }
    Ok(())
}
fn tables(
    document: &mut DocumentMut,
    key: &str,
    entries: impl Iterator<Item = Table>,
    charged: &mut usize,
) -> Result<(), Error> {
    let mut array = ArrayOfTables::new();
    for table in entries {
        charge(&table, charged)?;
        array.push(table);
    }
    if array.is_empty() {
        document[key] = value(Array::new());
    } else {
        document[key] = Item::ArrayOfTables(array);
    }
    Ok(())
}
fn record(id: &str, revision: Revision, name: &Name, roots: Revision, session: Revision) -> Table {
    let mut table = Table::new();
    table["id"] = value(id);
    table["revision"] = value(revision.get() as i64);
    table["name"] = value(name.as_str());
    table["roots_revision"] = value(roots.get() as i64);
    table["session_revision"] = value(session.get() as i64);
    table
}
fn path_value(path: &NativePath) -> Value {
    let mut table = InlineTable::new();
    match path.as_path().to_str() {
        Some(path) => {
            table.insert("path", path.into());
        }
        None => {
            let mut bytes = Array::new();
            for byte in path.as_path().as_os_str().as_bytes() {
                bytes.push(i64::from(*byte));
            }
            table.insert("path_bytes", Value::Array(bytes));
        }
    }
    Value::InlineTable(table)
}
fn root_value(root: &Root) -> Value {
    let mut table = InlineTable::new();
    table.insert("id", root.id.as_str().into());
    table.insert("label", root.label.as_str().into());
    table.insert("location", path_value(&root.path));
    if let Some(id) = &root.origin_project_id {
        table.insert("origin_project_id", id.as_str().into());
    }
    Value::InlineTable(table)
}
fn owner_value(owner: &Owner) -> Value {
    let (kind, id) = match owner {
        Owner::Project(id) => ("project", id.as_str()),
        Owner::Workspace(id) => ("workspace", id.as_str()),
    };
    let mut table = InlineTable::new();
    table.insert("kind", kind.into());
    table.insert("id", id.into());
    Value::InlineTable(table)
}
fn position_value(position: SourcePosition) -> Value {
    let mut table = InlineTable::new();
    table.insert("line", (position.line as i64).into());
    table.insert("column", (position.column as i64).into());
    Value::InlineTable(table)
}
fn session_table(session: &SavedSession, snapshot: &Snapshot) -> Result<Table, Error> {
    let mut table = Table::new();
    table["schema_version"] = value(1);
    table["owner"] = Item::Value(owner_value(&session.reference.owner));
    table["revision"] = value(session.reference.revision.get() as i64);
    table["roots_revision"] = value(session.roots_revision.get() as i64);
    let mut files = Array::new();
    for entry in &snapshot.files {
        let mut file = InlineTable::new();
        let location = match &entry.file {
            FileReference::RootRelative { root, path } => {
                file.insert("root", root.as_str().into());
                path
            }
            FileReference::LooseAbsolute(path) => path,
        };
        file.insert("location", path_value(location));
        file.insert("cursor", position_value(entry.cursor));
        if let Some(viewport) = entry.viewport {
            let mut view = InlineTable::new();
            view.insert("anchor", position_value(viewport.anchor));
            view.insert(
                "horizontal_cells",
                (viewport.horizontal_cells as i64).into(),
            );
            file.insert("viewport", Value::InlineTable(view));
        }
        files.push(Value::InlineTable(file));
    }
    table["files"] = value(files);
    if let Some(active) = &snapshot.active_file {
        let index = snapshot
            .files
            .iter()
            .position(|entry| entry.file == *active)
            .ok_or(Error::Invalid("active session file"))?;
        table["active_file"] = value(index as i64);
    }
    Ok(table)
}
