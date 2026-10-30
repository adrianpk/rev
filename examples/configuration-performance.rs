#![forbid(unsafe_code)]

use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

#[path = "configuration_performance/candidates.rs"]
mod candidates;
#[path = "configuration_performance/graph.rs"]
mod graph;
#[path = "configuration_performance/interaction.rs"]
mod interaction;
#[path = "configuration_performance/metrics.rs"]
mod metrics;
#[path = "configuration_performance/presentation.rs"]
mod presentation;

#[path = "configuration_performance/org.rs"]
mod org;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn require(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(io::Error::other(message).into());
    }
    Ok(())
}

// A flushed barrier lets the parent drain setup filesystem events before the
// cached workload starts. It is process synchronization, not a readiness sleep.
fn barrier() -> Result<()> {
    println!("ready");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    require(line == "go\n", "expected workload barrier")?;
    metrics::reset()
}

fn emit(name: &str, mut samples: Vec<Duration>, details: &str) -> Result<()> {
    samples.sort();
    require(!samples.is_empty(), "empty measurement")?;
    let median = samples[samples.len() / 2].as_secs_f64() * 1_000_000.0;
    let maximum = samples.last().unwrap().as_secs_f64() * 1_000_000.0;
    let usage = metrics::interval()?;
    let status = std::fs::read_to_string("/proc/self/status")?;
    let peak = status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing resident memory high-water mark")?
        .split_whitespace()
        .next()
        .ok_or("missing memory value")?
        .parse::<usize>()?;
    println!(
        "{{\"workload\":\"{name}\",\"samples\":{},\"median_us\":{median:.3},\"max_us\":{maximum:.3},\"peak_rss_kib\":{peak},{usage},{details}}}",
        samples.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    metrics::reset()?;
    let args: Vec<_> = std::env::args_os().collect();
    require(args.len() == 3, "expected workload and fixture root")?;
    let root = Path::new(&args[2]);
    match args[1].to_str() {
        Some("org") => org::run(root),
        Some("graph") => graph::run(root),
        Some("entries" | "names") => candidates::run(root, args[1].to_str().unwrap()),
        Some("redraw") => presentation::redraw(root),
        Some("syntax") => presentation::syntax(root),
        Some("gutter" | "drag") => interaction::run(root, args[1].to_str().unwrap()),
        _ => Err("unknown workload".into()),
    }
}
