use std::time::{Duration, Instant};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const DOCUMENT: &str =
    "alpha beta gamma delta epsilon zeta eta theta\nlast\t界e\u{301}\r\n";

pub(crate) fn config(profile: &str) -> String {
    format!(
        "[editor]\ninteraction_profile='{profile}'\n[tree]\nvisible=false\n[completion]\nautomatic=false\n"
    )
}

pub(crate) fn settle(editor: &mut rev::session::Session) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        editor.expire_input(Instant::now());
        editor.frame()?;
        if editor.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "native observation exceeded five seconds"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

// The shared PTY helper uses the existing Linux process observations for its
// bounded idle check; no independent process monitor or resource gate is added.
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
    let text = std::fs::read_to_string(format!("/proc/{process}/status"))?;
    Ok(text
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .ok_or("missing Linux peak RSS")?
        .trim()
        .to_owned())
}
