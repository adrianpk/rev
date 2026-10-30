//! Actual Org owners over the separately admitted release resource fixtures.
#[path = "org/cancellation.rs"]
mod cancellation;
#[path = "org/documents.rs"]
mod documents;
#[path = "org/projections.rs"]
mod projections;
use super::{Result, barrier, emit, require};
use std::{path::Path, time::Instant};

fn rss(field: &str) -> Result<usize> {
    let text = std::fs::read_to_string("/proc/self/status")?;
    Ok(text
        .lines()
        .find_map(|s| s.strip_prefix(field))
        .ok_or("missing Org resident memory")?
        .split_whitespace()
        .next()
        .ok_or("missing Org memory value")?
        .parse()?)
}

// Observe memory while the real result is retained. Resident growth includes
// allocator retention; it is not an exact count of logical Rust heap bytes.
fn observe<T>(
    name: &str,
    mut work: impl FnMut() -> Result<T>,
    details: impl Fn() -> String,
) -> Result<()> {
    let before = rss("VmRSS:")?;
    let mut growth = 0;
    let mut samples = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let retained = work()?;
        samples.push(start.elapsed());
        growth = growth.max(rss("VmRSS:")?.saturating_sub(before));
        std::hint::black_box(&retained);
    }
    let details = details();
    let detail =
        format!("\"baseline_rss_kib\":{before},\"retained_growth_kib\":{growth},{details}");
    emit(name, samples, &detail)
}

pub(super) fn run(root: &Path) -> Result<()> {
    barrier()?;
    documents::source(root)?;
    projections::agenda(root)?;
    projections::habits(root)?;
    documents::formulas()?;
    documents::exports(root)?;
    cancellation::run(root)
}
