use std::sync::mpsc;
use std::time::{Duration, Instant};

use rev::buffer::{Buffer, Buffers};
use rev::org::{Cancellation, Coverage, Job, JobError, Limits, Scan};

const DEADLINE: Duration = Duration::from_secs(2);

fn result<T>(job: &mut Job<T>, buffer: &Buffer) -> Result<T, JobError> {
    let started = Instant::now();
    loop {
        if let Some(result) = job.poll(buffer) {
            return result;
        }
        assert!(
            started.elapsed() < DEADLINE,
            "owned observation did not finish"
        );
        std::thread::yield_now();
    }
}

// Cancellation between compute steps leaves truthful partial coverage and
// cannot mark a cancelled observation complete or consume the remaining source.
#[test]
fn bounded_cancel() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create(&"* Heading\nbody\n".repeat(10_000)).unwrap();
    let mut scan = Scan::new(buffers.get(id).unwrap(), Limits::default()).unwrap();
    let cancellation = Cancellation::default();
    assert_eq!(scan.advance(&cancellation), Coverage::Pending);
    let before = scan.scanned_bytes();
    assert!(before.0 <= 4096);
    cancellation.cancel();
    assert_eq!(scan.advance(&cancellation), Coverage::Cancelled);
    assert_eq!(scan.scanned_bytes(), before);
    assert_eq!(scan.into_index().coverage(), Coverage::Cancelled);
}

// A cancelled worker retains the shared slot until it finishes and is reaped.
// An owned channel makes this boundary observable without timing-based sleeps.
#[test]
fn cancelled_slot() {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let mut job = Job::default();
    let (ready, observed) = mpsc::sync_channel(1);
    let (release, blocked) = mpsc::sync_channel(1);
    job.start(buffer.stamp(), move |cancel| {
        ready.send(()).unwrap();
        blocked.recv_timeout(DEADLINE).unwrap();
        assert!(cancel.is_cancelled());
        7
    })
    .unwrap();
    observed.recv_timeout(DEADLINE).unwrap();
    assert!(job.cancel());
    assert!(job.is_active());
    assert!(matches!(
        job.start(buffer.stamp(), |_| 8),
        Err(JobError::Busy)
    ));
    assert!(job.poll(buffer).is_none());
    release.send(()).unwrap();
    assert!(matches!(result(&mut job, buffer), Err(JobError::Cancelled)));
    assert!(!job.is_active());
    assert!(!job.cancel());
    assert!(job.poll(buffer).is_none());
    job.start(buffer.stamp(), |_| 9).unwrap();
    assert_eq!(result(&mut job, buffer).unwrap(), 9);
}

// An actual index worker borrows shared snapshot nodes. Editing the live buffer
// cannot make its old result current, even when source offsets still fit.
#[test]
fn stale_worker() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.create("* Before\nbody\n").unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let scan = Scan::new(buffer, Limits::default()).unwrap();
    let mut job = Job::default();
    job.start_index(scan).unwrap();
    super::incremental::replace(buffer, 2..8, "After");
    assert!(matches!(
        result(&mut job, buffer),
        Err(JobError::Source(rev::buffer::Error::Stale { .. }))
    ));
    assert!(buffer.org_index().is_none());
    job.start_index(Scan::new(buffer, Limits::default()).unwrap())
        .unwrap();
    let index = result(&mut job, buffer).unwrap();
    assert_eq!(index.coverage(), Coverage::Complete);
    buffer.install_org_index(index).unwrap();
    assert_eq!(buffer.org_index().unwrap().stamp(), buffer.stamp());
}

// A worker result is document-qualified, and a panic is an explicit failure.
// Both outcomes release the slot without manufacturing a successful result.
#[test]
fn failed_workers() {
    let mut buffers = Buffers::new().unwrap();
    let first = buffers.initial();
    let second = buffers.create("").unwrap();
    let mut job = Job::default();
    job.start(buffers.get(first).unwrap().stamp(), |_| 1)
        .unwrap();
    assert!(matches!(
        result(&mut job, buffers.get(second).unwrap()),
        Err(JobError::Source(rev::buffer::Error::Foreign { .. }))
    ));
    assert!(!job.is_active());
    job.start(buffers.get(first).unwrap().stamp(), |_| {
        panic!("owned test panic")
    })
    .unwrap();
    assert!(matches!(
        result(&mut job, buffers.get(first).unwrap()),
        Err(JobError::Panicked)
    ));
    assert!(!job.is_active());
}

// Dropping an active owner requests cancellation and joins its worker. Repeated
// owners cannot leave detached work alive after their lifecycle ends.
#[test]
fn owned_cleanup() {
    let buffers = Buffers::new().unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    for _ in 0..8 {
        let mut job = Job::default();
        let (done, completed) = mpsc::sync_channel(1);
        job.start(buffer.stamp(), move |cancel| {
            let started = Instant::now();
            while !cancel.is_cancelled() {
                assert!(started.elapsed() < DEADLINE);
                std::thread::yield_now();
            }
            done.send(()).unwrap();
        })
        .unwrap();
        drop(job);
        assert!(completed.try_recv().is_ok());
    }
}
