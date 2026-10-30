use std::time::{Duration, Instant};

fn stopped(pid: &str) -> bool {
    std::fs::read_to_string(format!("/proc/{}/stat", pid.trim()))
        .map_or(true, |s| s.split_once(") ").unwrap().1.starts_with('Z'))
}
pub(super) fn descendant_stopped(pid: &str, mode: &str) {
    // A successful group signal and leader reap do not synchronize the kernel's
    // descendant state transition. Observe that same required stopped state
    // within the existing 150 ms response budget, without a blind grace sleep.
    let started = Instant::now();
    let deadline = started + Duration::from_millis(150);
    while !stopped(pid) {
        assert!(Instant::now() < deadline, "{mode}: descendant running");
        std::thread::sleep(Duration::from_millis(1));
    }
    eprintln!(
        "{mode}: descendant stop observed in {:?}",
        started.elapsed()
    );
}
