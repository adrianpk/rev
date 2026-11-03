use super::support::Result;

pub(crate) fn cpu_ticks(process: &str) -> Result<u64> {
    let text = std::fs::read_to_string(format!("/proc/{process}/stat"))?;
    let fields: Vec<_> = text
        .rsplit_once(')')
        .ok_or("invalid Linux stat")?
        .1
        .split_whitespace()
        .collect();
    Ok(fields[11].parse::<u64>()? + fields[12].parse::<u64>()?)
}

pub(crate) fn peak_rss(process: &str) -> Result<String> {
    let status = std::fs::read_to_string(format!("/proc/{process}/status"))?;
    Ok(status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak RSS")?
        .trim()
        .to_owned())
}

pub(crate) fn threads() -> Result<usize> {
    Ok(std::fs::read_dir("/proc/self/task")?.count())
}
