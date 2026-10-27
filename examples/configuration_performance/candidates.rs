use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use rev::file::completion::{ENTRY_LIMIT, NAME_BYTE_LIMIT, Provider};

use super::{Result, barrier, emit, require};

pub(super) fn run(root: &Path, kind: &str) -> Result<()> {
    let shallow = Provider::observe(&root.join("small"))?;
    require(
        shallow.entries().len() == 3,
        "shallow traversal control missing",
    )?;
    let directory = root.join(kind);
    let mut samples = Vec::new();
    for _ in 0..3 {
        let start = Instant::now();
        let provider = Provider::observe(&directory)?;
        samples.push(start.elapsed());
        admitted(&provider, kind)?;
    }
    let provider = Provider::observe(&directory)?;
    emit(
        &format!("{kind}_observe"),
        samples,
        &format!(
            "\"entries\":{},\"name_bytes\":{}",
            provider.entries().len(),
            provider.name_bytes()
        ),
    )?;
    barrier()?;
    filtering(&provider, kind)
}

fn admitted(provider: &Provider, kind: &str) -> Result<()> {
    require(
        provider.diagnostic().is_some(),
        "bounded observation was not incomplete",
    )?;
    require(
        provider.name_bytes() <= NAME_BYTE_LIMIT,
        "name budget exceeded",
    )?;
    let expected = if kind == "entries" {
        provider.entries().len() == ENTRY_LIMIT
    } else {
        provider.entries().len() < ENTRY_LIMIT && provider.name_bytes() > NAME_BYTE_LIMIT - 255
    };
    require(expected, "candidate maximum missing")
}

fn filtering(provider: &Provider, kind: &str) -> Result<()> {
    let mut samples = Vec::new();
    for index in 0..256 {
        let prefix = ["", "0", "10", "absent"][index % 4];
        let start = Instant::now();
        let matches = provider.matching(prefix);
        black_box(
            matches
                .iter()
                .map(|entry| entry.label().len())
                .sum::<usize>(),
        );
        samples.push(start.elapsed());
        require(
            matches.len() <= ENTRY_LIMIT,
            "filter allocation escaped entry bound",
        )?;
    }
    emit(&format!("{kind}_filter"), samples, "\"iterations\":256")
}
