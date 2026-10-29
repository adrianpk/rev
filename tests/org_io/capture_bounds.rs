use super::Fixture;
use super::capture::{choice, discover, heading};
use rev::{
    buffer::{Buffers, edit::Replacement},
    config::{CaptureKind, CaptureTarget, Org},
    coordinates::Byte,
    org::{Cancellation, Job, JobError, capture::Choice, io::DestinationVersion, time::Date},
};
use std::{
    fs,
    time::{Duration, Instant},
};

// Exact source bytes include disk BOM/CRLF. An over-limit file cannot be
// published as an empty/new target, and cancellation leaves no created parent.
#[test]
fn capture_read_bounds() {
    let f = Fixture::new();
    let source = "\u{feff}* Task\r\n";
    let path = f.write("target.org", source);
    let buffers = Buffers::new().unwrap();
    for (name, limit, valid) in [
        ("exact", source.len(), true),
        ("over", source.len() - 1, false),
    ] {
        let mut policy = Org::default();
        policy.limits.agenda_file_bytes = limit;
        let work = choice(&path, CaptureKind::Entry, heading("Task"))
            .discover(&buffers, &policy, Date::new(2024, 3, 1).unwrap())
            .unwrap();
        assert_eq!(work.run(&Cancellation::default()).is_ok(), valid, "{name}");
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
    }
    let target = f.0.join("missing/sub/notes.org");
    let work = choice(&target, CaptureKind::Entry, heading("Task"))
        .discover(&buffers, &Org::default(), Date::new(2024, 3, 1).unwrap())
        .unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(work.run(&cancel).is_err());
    assert!(!f.0.join("missing").exists());
}
// Encoding, nonregular paths, dangling links and malformed structure cannot
// manufacture valid target plans. All original bytes remain unchanged.
#[test]
fn capture_failed_sources() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    for (name, bytes) in [
        ("invalid UTF-8", vec![0xff]),
        ("NUL", b"* Task\0\n".to_vec()),
        ("unclosed block", b"#+begin_src text\nbody\n".to_vec()),
        (
            "unescaped heading",
            b"#+begin_src text\n* Task\n#+end_src\n".to_vec(),
        ),
    ] {
        let path = f.0.join("bad.org");
        fs::write(&path, &bytes).unwrap();
        assert!(
            discover(choice(&path, CaptureKind::Entry, heading("Task")), &buffers).is_err(),
            "{name}"
        );
        assert_eq!(fs::read(path).unwrap(), bytes, "{name}");
    }
    assert!(discover(choice(&f.0, CaptureKind::Entry, heading("Task")), &buffers).is_err());
    let dangling = f.0.join("dangling.org");
    std::os::unix::fs::symlink("absent.org", &dangling).unwrap();
    assert!(
        discover(
            choice(&dangling, CaptureKind::Entry, heading("Task")),
            &buffers
        )
        .is_err()
    );
    assert!(!f.0.join("absent.org").exists());
}
// Date hierarchy plans reuse source date prefixes and insert before a later
// sibling. Duplicate date nodes and a hierarchy beyond the native depth reject.
#[test]
fn capture_date_plans() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    let source = "* 2024\n** 2024-03 Custom month label\n*** 2024-03-02 Saturday\n";
    let path = f.write("dates.org", source);
    let target = CaptureTarget::DateTree {
        file: String::new(),
        outline: vec![],
    };
    let result = discover(choice(&path, CaptureKind::Entry, target.clone()), &buffers).unwrap();
    assert_eq!(result.target.missing.len(), 1);
    assert_eq!(result.target.missing[0].title, "2024-03-01 Friday");
    assert_eq!(
        result.target.missing_at,
        Some(Byte(source.find("***").unwrap()))
    );
    assert_eq!(
        result.target.parent,
        Some(Byte(source.find("** ").unwrap()))
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), source);
    fs::write(&path, "* 2024\n** 2024-03 March\n** 2024-03 Other\n").unwrap();
    assert!(
        discover(choice(&path, CaptureKind::Entry, target), &buffers)
            .err()
            .unwrap()
            .contains("Ambiguous")
    );
    let source = format!("{} Base\n", "*".repeat(254));
    fs::write(&path, &source).unwrap();
    let target = CaptureTarget::DateTree {
        file: String::new(),
        outline: vec!["Base".into()],
    };
    assert!(
        discover(choice(&path, CaptureKind::Entry, target), &buffers)
            .err()
            .unwrap()
            .contains("256-level")
    );
    assert_eq!(fs::read_to_string(path).unwrap(), source);
}
// Snapshot evidence retains real old text/revision across subsequent canonical
// edits; it never relabels old bytes as a current buffer observation.
#[test]
fn capture_revision_evidence() {
    let f = Fixture::new();
    let path = f.write("target.org", "* Task\n");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&path).unwrap();
    let work = choice(&path, CaptureKind::Entry, heading("Task"))
        .discover(&buffers, &Org::default(), Date::new(2024, 3, 1).unwrap())
        .unwrap();
    let buffer = buffers.get_mut(id).unwrap();
    let old = buffer.stamp();
    let edit = Replacement {
        source: buffer.span(0..0).unwrap(),
        text: "# intervening\n".into(),
    };
    let prepared = buffer.prepare(&[edit], &[], None).unwrap();
    buffer.publish(prepared, &mut []).unwrap();
    let current = buffer.stamp();
    let result = work.run(&Cancellation::default()).unwrap();
    assert_eq!(result.source.version, DestinationVersion::Buffer(old));
    assert_ne!(old, current);
    assert_eq!(result.source.text().to_string(), "* Task\n");
    assert_eq!(buffers.get(id).unwrap().history().len(), 1);
    assert_eq!(fs::read_to_string(path).unwrap(), "* Task\n");
}
// Existing session job ownership can cancel/reap capture work without a second
// worker or target write. A newly configured destination remains absent.
#[test]
fn capture_job_ownership() {
    let f = Fixture::new();
    let buffers = Buffers::new().unwrap();
    let path = f.0.join("missing/target.org");
    let mut selected = Choice {
        template: rev::config::CaptureTemplate::task(),
        file: None,
    };
    assert!(
        selected
            .discover(&buffers, &Org::default(), Date::new(2024, 3, 1).unwrap())
            .is_err()
    );
    selected.enter_file(path.to_str().unwrap(), &f.0).unwrap();
    let work = selected
        .discover(&buffers, &Org::default(), Date::new(2024, 3, 1).unwrap())
        .unwrap();
    let buffer = buffers.get(buffers.initial()).unwrap();
    let mut job = Job::default();
    job.start(buffer.stamp(), move |cancel| work.run(cancel))
        .unwrap();
    assert!(job.cancel());
    let until = Instant::now() + Duration::from_secs(2);
    let result = loop {
        if let Some(result) = job.poll(buffer) {
            break result;
        }
        assert!(Instant::now() < until, "owned capture job did not reap");
        std::thread::yield_now();
    };
    assert!(matches!(result, Err(JobError::Cancelled)));
    assert!(!job.is_active());
    assert!(!f.0.join("missing").exists());
}
