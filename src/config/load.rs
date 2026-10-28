use std::fs::{self, File, Metadata};
use std::io::{self, Read};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::{Diagnostic, Document, parse};

const MAX_DEPTH: usize = 16;
pub(super) const MAX_SOURCES: usize = 64;
pub(super) const SOURCE_BYTES: usize = 256 * 1024;
const TOTAL_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}

/// One occurrence in the explicit graph. Its index in Sources::sources is its
/// occurrence ID; parent refers to the declaring occurrence, not a deduplicated
/// file. Path retains the selected spelling for relative include resolution.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub parent: Option<usize>,
    pub declaration: usize,
    pub document: Document,
    identity: Identity,
}

/// Validated sources in preorder, with a depth-first postorder for layering:
/// each declared include precedes its parent, and the root is last.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sources {
    pub sources: Vec<Source>,
    pub order: Vec<usize>,
    pub total_bytes: usize,
}

#[derive(Default)]
struct Walk {
    graph: Sources,
    active: Vec<usize>,
}

struct Input {
    bytes: Vec<u8>,
    canonical: PathBuf,
    identity: Identity,
}

/// Load the selected optional root and its explicit includes. A nonexistent
/// root returns an empty graph for built-in defaults; any present broken root
/// or explicit source rejects the entire graph. Nothing is created or rewritten.
pub fn load(root: &Path) -> Result<Sources, Diagnostic> {
    match fs::symlink_metadata(root) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Sources::default()),
        Err(error) => {
            return Err(Diagnostic::new(
                root,
                format!("inspect configuration: {error}"),
            ));
        }
        Ok(_) => (),
    }
    let mut walk = Walk::default();
    walk.visit(root, None, 0)?;
    Ok(walk.graph)
}

impl Walk {
    fn visit(
        &mut self,
        path: &Path,
        parent: Option<usize>,
        declaration: usize,
    ) -> Result<(), Diagnostic> {
        self.admit(path, parent)?;
        let input = read_file(path, TOTAL_BYTES - self.graph.total_bytes)
            .map_err(|error| self.error(path, parent, format!("read configuration: {error}")))?;
        self.cycle(path, parent, &input)?;
        let document = parse(path, &input.bytes)?;
        self.graph.total_bytes += input.bytes.len();
        let id = self.graph.sources.len();
        self.graph.sources.push(Source {
            path: path.to_owned(),
            canonical_path: input.canonical,
            parent,
            declaration,
            document,
            identity: input.identity,
        });
        self.active.push(id);
        self.includes(id)?;
        self.active.pop();
        self.graph.order.push(id);
        Ok(())
    }

    fn admit(&self, path: &Path, parent: Option<usize>) -> Result<(), Diagnostic> {
        if self.active.len() > MAX_DEPTH {
            return Err(self.error(path, parent, "include depth exceeds 16 edges".into()));
        }
        if self.graph.sources.len() >= MAX_SOURCES {
            return Err(self.error(
                path,
                parent,
                "configuration exceeds 64 source occurrences".into(),
            ));
        }
        Ok(())
    }

    fn cycle(&self, path: &Path, parent: Option<usize>, input: &Input) -> Result<(), Diagnostic> {
        let repeated = self
            .active
            .iter()
            .map(|id| &self.graph.sources[*id])
            .find(|source| {
                source.canonical_path == input.canonical || source.identity == input.identity
            });
        if let Some(source) = repeated {
            return Err(self.error(
                path,
                parent,
                format!("include cycle reaches {}", source.path.display()),
            ));
        }
        Ok(())
    }

    fn includes(&mut self, id: usize) -> Result<(), Diagnostic> {
        let count = self.graph.sources[id]
            .document
            .include
            .as_ref()
            .map_or(0, Vec::len);
        for declaration in 0..count {
            let path = self.include_path(id, declaration)?;
            self.visit(&path, Some(id), declaration)?;
        }
        Ok(())
    }

    fn include_path(&self, id: usize, declaration: usize) -> Result<PathBuf, Diagnostic> {
        let source = &self.graph.sources[id];
        let include = &source
            .document
            .include
            .as_ref()
            .expect("declared include list")[declaration];
        if include.is_empty() || include.contains('\0') || include.contains("://") {
            return Err(self.error(
                Path::new(include),
                Some(id),
                "expected a nonempty local include path".into(),
            ));
        }
        let parent = source.path.parent().unwrap_or(Path::new("."));
        Ok(parent.join(include))
    }

    fn error(&self, path: &Path, parent: Option<usize>, message: String) -> Diagnostic {
        let Some(id) = parent else {
            return Diagnostic::new(path, message);
        };
        let mut error = Diagnostic::new(
            &self.graph.sources[id].path,
            format!("include {}: {message}", path.display()),
        );
        error.key.push("include".into());
        error
    }
}

fn open(path: &Path) -> io::Result<(File, Metadata, PathBuf)> {
    regular(&fs::metadata(path)?)?;
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    regular(&metadata)?;
    let canonical = fs::canonicalize(path)?;
    Ok((file, metadata, canonical))
}

fn regular(metadata: &Metadata) -> io::Result<()> {
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular configuration file",
        ));
    }
    Ok(())
}

fn read_file(path: &Path, remaining: usize) -> io::Result<Input> {
    let (mut file, metadata, canonical) = open(path)?;
    let limit = SOURCE_BYTES.min(remaining);
    size(metadata.len(), limit, remaining)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    // Take enforces the byte budget during IO, even if the file grows after
    // inspection. A second size check rejects growth beyond that budget without
    // reading an extra sentinel byte past either admitted limit.
    file.by_ref().take(limit as u64).read_to_end(&mut bytes)?;
    size(file.metadata()?.len(), limit, remaining)?;
    Ok(Input {
        bytes,
        canonical,
        identity: Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        },
    })
}

fn size(length: u64, limit: usize, remaining: usize) -> io::Result<()> {
    if length <= limit as u64 {
        return Ok(());
    }
    let message = if remaining < SOURCE_BYTES {
        "configuration exceeds 2 MiB total bytes"
    } else {
        "configuration source exceeds 256 KiB"
    };
    Err(io::Error::new(io::ErrorKind::InvalidData, message))
}
