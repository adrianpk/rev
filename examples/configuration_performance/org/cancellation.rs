use super::{Result, emit, require, rss};
use rev::{
    buffer::Buffers,
    org::{Coverage, Job, JobError, Limits, Scan},
};
use std::{
    path::Path,
    sync::mpsc,
    time::{Duration, Instant},
};

fn owners() -> Result<(usize, usize, String)> {
    Ok((
        std::fs::read_dir("/proc/self/task")?.count(),
        std::fs::read_dir("/proc/self/fd")?.count(),
        std::fs::read_to_string("/proc/thread-self/children")?,
    ))
}

fn reaped(job: &mut Job<Coverage>, buffer: &rev::buffer::Buffer) -> Result<Duration> {
    let start = Instant::now();
    loop {
        if let Some(result) = job.poll(buffer) {
            require(
                matches!(result, Err(JobError::Cancelled)),
                "cancelled work was published",
            )?;
            return Ok(start.elapsed());
        }
        require(
            start.elapsed() <= Duration::from_millis(150),
            "cancel/reap exceeded 150ms",
        )?;
        std::thread::yield_now();
    }
}

fn released(before: &(usize, usize, String)) -> Result<()> {
    let start = Instant::now();
    while owners()? != *before {
        require(
            start.elapsed() <= Duration::from_millis(150),
            "owned thread/descriptor/child leaked",
        )?;
        std::thread::yield_now();
    }
    Ok(())
}

fn cycle(buffer: &rev::buffer::Buffer) -> Result<Duration> {
    let mut scan = Scan::new(buffer, Limits::default())?;
    let mut job = Job::default();
    let (sender, receiver) = mpsc::sync_channel(0);
    let observed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let worker = observed.clone();
    job.start(buffer.stamp(), move |cancel| {
        let first = scan.advance(cancel);
        sender.send(first).expect("owned setup receiver");
        while scan.advance(cancel) == Coverage::Pending {
            std::thread::yield_now();
        }
        let coverage = scan.index().coverage();
        worker.store(
            coverage == Coverage::Cancelled,
            std::sync::atomic::Ordering::Release,
        );
        coverage
    })?;
    require(
        receiver.recv_timeout(Duration::from_millis(150))? == Coverage::Pending,
        "cancel fixture finished before progress",
    )?;
    require(
        matches!(
            job.start(buffer.stamp(), |_| Coverage::Complete),
            Err(JobError::Busy)
        ),
        "second job admitted",
    )?;
    let start = Instant::now();
    require(job.cancel(), "active cancel missing")?;
    reaped(&mut job, buffer)?;
    require(
        observed.load(std::sync::atomic::Ordering::Acquire),
        "worker did not observe cancellation",
    )?;
    require(!job.is_active(), "cancel retained job slot")?;
    Ok(start.elapsed())
}

pub(super) fn run(root: &Path) -> Result<()> {
    let mut buffers = Buffers::new()?;
    let id = buffers.visit(&root.join("work/large.org"))?;
    let buffer = buffers.get(id)?;
    let before = owners()?;
    require(
        before.2.trim().is_empty(),
        "native fixture has unexpected children",
    )?;
    let memory = rss("VmRSS:")?;
    let mut samples = Vec::new();
    for _ in 0..16 {
        samples.push(cycle(buffer)?);
        released(&before)?;
    }
    require(
        buffer.history().is_empty() && !buffer.is_modified(),
        "cancel changed canonical source",
    )?;
    let growth = rss("VmRSS:")?.saturating_sub(memory);
    emit(
        "org_cancel",
        samples,
        &format!(
            "\"cycles\":16,\"baseline_rss_kib\":{memory},\"retained_growth_kib\":{growth},\"threads_before\":{},\"threads_after\":{},\"fds_before\":{},\"fds_after\":{},\"children_after\":0,\"busy_rejected\":16,\"cancelled\":16",
            before.0,
            owners()?.0,
            before.1,
            owners()?.1
        ),
    )
}
