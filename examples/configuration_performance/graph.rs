use std::path::Path;
use std::time::Instant;

use rev::config::{Snapshot, load, prepare, resolve};

use super::{Result, barrier, emit, require};

pub(super) fn settings(root: &Path) -> Result<Snapshot> {
    let startup = prepare(&root.join("user/rev/config.toml"));
    require(startup.diagnostic.is_none(), "configuration was rejected")?;
    Ok(startup.snapshot)
}

pub(super) fn run(root: &Path) -> Result<()> {
    let path = root.join("user/rev/config.toml");
    let mut samples = Vec::new();
    barrier()?;
    for _ in 0..3 {
        let start = Instant::now();
        let sources = load(&path)?;
        let snapshot = resolve(&sources)?;
        samples.push(start.elapsed());
        require(
            sources.sources.len() == 64,
            "source occurrence maximum missing",
        )?;
        require(
            sources.total_bytes == 2 * 1024 * 1024,
            "total byte maximum missing",
        )?;
        require(depth(&sources) == 16, "include depth maximum missing")?;
        require(
            snapshot.settings().editor.tab_width == 8,
            "graph was not applied",
        )?;
    }
    emit(
        "graph",
        samples,
        "\"sources\":64,\"edges\":16,\"total_bytes\":2097152",
    )
}

fn depth(sources: &rev::config::Sources) -> usize {
    let mut depths = vec![0; sources.sources.len()];
    for (id, source) in sources.sources.iter().enumerate() {
        depths[id] = source.parent.map_or(0, |parent| depths[parent] + 1);
    }
    depths.into_iter().max().unwrap_or(0)
}
