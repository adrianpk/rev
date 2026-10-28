use std::cell::Cell;
use std::time::Instant;

use super::Result;

thread_local! {
    static LAST: Cell<Option<(Instant, u64)>> = const { Cell::new(None) };
}

fn ticks() -> Result<u64> {
    let source = std::fs::read_to_string("/proc/self/stat")?;
    let (_, tail) = source.rsplit_once(')').ok_or("missing process fields")?;
    let fields: Vec<_> = tail.split_whitespace().collect();
    let user: u64 = fields.get(11).ok_or("missing user CPU")?.parse()?;
    let system: u64 = fields.get(12).ok_or("missing system CPU")?.parse()?;
    Ok(user + system)
}

pub(super) fn reset() -> Result<()> {
    let value = (Instant::now(), ticks()?);
    LAST.set(Some(value));
    Ok(())
}

pub(super) fn interval() -> Result<String> {
    let now = Instant::now();
    let cpu = ticks()?;
    let (started, before) = LAST
        .replace(Some((now, cpu)))
        .ok_or("missing work measurement")?;
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    Ok(format!(
        "\"elapsed_ms\":{elapsed:.3},\"cpu_ticks\":{}",
        cpu - before
    ))
}
