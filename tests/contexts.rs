#![forbid(unsafe_code)]

#[path = "contexts/domain.rs"]
mod domain;
#[path = "contexts/editor.rs"]
mod editor;
#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "contexts/storage.rs"]
mod storage;

use rev::context::*;
use std::path::Path;

fn root(path: &Path) -> Root {
    Root {
        id: RootId::new().unwrap(),
        path: NativePath::absolute(path).unwrap(),
        label: Name::new("Source").unwrap(),
        origin_project_id: None,
    }
}
fn project(catalog: &mut Catalog, path: &Path, name: &str) -> ProjectId {
    let id = ProjectId::new().unwrap();
    let reference = SessionRef {
        owner: Owner::Project(id.clone()),
        revision: Revision::INITIAL,
    };
    catalog.projects.push(Project {
        id: id.clone(),
        revision: Revision::INITIAL,
        name: Name::new(name).unwrap(),
        roots_revision: Revision::INITIAL,
        root: root(path),
        session_ref: reference.clone(),
    });
    catalog.sessions.push(SavedSession::new(
        reference,
        Revision::INITIAL,
        Snapshot::default(),
    ));
    id
}
fn workspace(catalog: &mut Catalog, name: &str, roots: Vec<Root>) -> WorkspaceId {
    let id = WorkspaceId::new().unwrap();
    let reference = SessionRef {
        owner: Owner::Workspace(id.clone()),
        revision: Revision::INITIAL,
    };
    catalog.workspaces.push(Workspace {
        id: id.clone(),
        revision: Revision::INITIAL,
        name: Name::new(name).unwrap(),
        roots_revision: Revision::INITIAL,
        roots,
        session_ref: reference.clone(),
    });
    catalog.sessions.push(SavedSession::new(
        reference,
        Revision::INITIAL,
        Snapshot::default(),
    ));
    id
}
fn position(line: usize, column: usize) -> SourcePosition {
    SourcePosition { line, column }
}
fn entry(file: FileReference) -> FileEntry {
    FileEntry {
        file,
        cursor: position(2, 3),
        viewport: Some(Viewport {
            anchor: position(1, 1),
            horizontal_cells: 4,
        }),
    }
}
