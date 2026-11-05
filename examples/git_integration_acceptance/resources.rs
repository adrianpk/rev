use crate::{
    repository::{Repository, finish},
    support::*,
};
use rev::git::{Completeness, Family, Outcome, Repositories, SESSION_BYTES};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const POLICY: &str = "max_status_paths=32\nmax_path_bytes=4096\nmax_output_bytes=262144\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024\nmax_process_records=4\nmax_process_record_bytes=2097152\n";

// Real large Status/output producers establish byte/record admission. RSS is
// measured separately: it cannot substitute for charged Git-owned capacities.
pub(crate) fn run() -> Result<()> {
    let before = descriptors()?;
    let threads = crate::metrics::threads()?;
    let started = Instant::now();
    fair_contexts()?;
    output_retention()?;
    malformed()?;
    assert_eq!(descriptors()?, before, "Git fixture leaked descriptors");
    assert_eq!(crate::metrics::threads()?, threads, "Git worker leaked");
    println!(
        "PASS Git resources elapsed_ms={} descriptors={before} threads={threads} peak_rss={}",
        started.elapsed().as_millis(),
        peak_rss("self")?
    );
    Ok(())
}

fn descriptors() -> Result<usize> {
    Ok(fs::read_dir("/proc/self/fd")?.count())
}

fn large() -> Result<Repository> {
    let repository = Repository::new("sha1");
    repository.commit();
    for index in 0..512 {
        fs::write(
            repository
                .root
                .join(format!("{index:04}-{}", "name".repeat(20))),
            "payload\n",
        )?;
    }
    Ok(repository)
}

fn fair_contexts() -> Result<()> {
    let repositories: Vec<_> = (0..4).map(|_| large()).collect::<Result<_>>()?;
    byte_refusal(&repositories)?;
    let wakes = Arc::new(AtomicUsize::new(0));
    let notifications = wakes.clone();
    let mut owner = Repositories::new(
        &repositories[0].snapshot(POLICY),
        Arc::new(move || {
            notifications.fetch_add(1, Ordering::Relaxed);
        }),
    );
    for repository in &repositories {
        assert!(repository.open(&mut owner, &repository.root).published);
    }
    let retained = owner.contexts()[0].observation.clone();
    let ids: Vec<_> = owner.contexts().iter().map(|c| c.id).collect();
    for id in &ids {
        owner.refresh(*id)?;
    }
    assert_eq!(owner.progress().count(), 4);
    let (turns, longest) = collect(&mut owner)?;
    check_contexts(&owner);
    let charged = owner.admitted_bytes();
    drop(retained);
    assert!(
        owner.admitted_bytes() < charged,
        "retained observation must stay charged"
    );
    let fifth = Repository::new("sha1");
    assert!(!fifth.open(&mut owner, &fifth.root).published);
    assert_eq!(owner.contexts().len(), 4);
    for id in ids {
        assert!(owner.context(id).is_some());
    }
    quiet(&owner, &wakes)?;
    println!(
        "MEASURE four real repositories: input_paths=2048 retained_paths=128 charged_bytes={charged} ceiling={SESSION_BYTES} turns={turns} longest_poll_us={longest} wake_count={}",
        wakes.load(Ordering::Relaxed)
    );
    Ok(())
}

fn byte_refusal(repositories: &[Repository]) -> Result<()> {
    let policy = POLICY.replace(
        "max_process_record_bytes=2097152",
        "max_process_record_bytes=262144",
    );
    let mut tight = repositories[0].owner(&policy);
    for repository in &repositories[..2] {
        assert!(repository.open(&mut tight, &repository.root).published);
    }
    let first = tight.contexts()[0].id;
    let second = tight.contexts()[1].id;
    tight.refresh(first)?;
    assert!(
        tight.refresh(second).is_err(),
        "concurrent output reservations must fit the record-byte budget"
    );
    assert!(finish(&mut tight).published);
    assert_eq!(tight.contexts().len(), 2);
    println!(
        "PASS record-byte refusal: 262144-byte policy preserves live work; four-child fixture uses the approved 2097152-byte default"
    );
    Ok(())
}

fn collect(owner: &mut Repositories) -> Result<(usize, u128)> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut completed = Vec::new();
    let mut turns = 0;
    let mut longest = 0;
    while completed.len() < 4 {
        let started = Instant::now();
        let updates = owner.poll();
        longest = longest.max(started.elapsed().as_micros());
        assert!(
            updates.len() <= 64,
            "coalesced event service exceeds turn bound"
        );
        for update in updates {
            assert!(update.published);
            let context = update.context.unwrap();
            assert!(
                !completed.contains(&context),
                "duplicate context publication"
            );
            completed.push(context);
        }
        turns += 1;
        assert!(owner.admitted_bytes() <= SESSION_BYTES);
        assert!(
            Instant::now() < deadline,
            "four repositories must all make progress"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(completed.len(), 4, "one repository starved another");
    Ok((turns, longest))
}

fn check_contexts(owner: &Repositories) {
    for context in owner.contexts() {
        let status = context.observation.status.as_ref().unwrap();
        assert_eq!(status.paths.len(), 32);
        assert!(matches!(
            status.completeness,
            Completeness::Truncated { .. }
        ));
        assert!(
            status
                .paths
                .iter()
                .map(|p| p.path.as_os_str().as_encoded_bytes().len())
                .sum::<usize>()
                <= 4096
        );
    }
    assert!(!owner.busy());
    assert!(owner.remaining().is_none());
}

fn quiet(owner: &Repositories, wakes: &AtomicUsize) -> Result<()> {
    let before = wakes.load(Ordering::Relaxed);
    let cpu = cpu_ticks("self")?;
    std::thread::sleep(Duration::from_millis(350));
    let ticks = cpu_ticks("self")?.saturating_sub(cpu);
    assert!(ticks <= 1, "quiet Git consumed {ticks} CPU ticks");
    assert_eq!(
        wakes.load(Ordering::Relaxed),
        before,
        "quiet Git woke the editor"
    );
    assert!(owner.remaining().is_none());
    println!("MEASURE native Git idle: interval_ms=350 cpu_ticks={ticks} pending_timer=false");
    Ok(())
}

fn output_retention() -> Result<()> {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("large"), vec![b'x'; 262144])?;
    repository.commit();
    let mut owner = repository.owner("max_output_bytes=4096\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024\nmax_process_records=3\nmax_process_record_bytes=65536\n");
    assert!(repository.open(&mut owner, &repository.root).published);
    let context = owner.contexts()[0].id;
    for _ in 0..4 {
        assert!(repository.open(&mut owner, &repository.root).published);
        owner.prepare_family(
            &repository.root,
            Some(context),
            Family::RunCommand,
            vec!["show".into(), "HEAD:large".into()],
        )?;
        assert!(finish(&mut owner).published);
        let plan = owner.family_plan().unwrap().clone();
        let id = owner.execute_family(plan)?;
        finish(&mut owner);
        let record = owner.record(id).unwrap();
        assert_eq!(record.outcome, Outcome::OutputLimit);
        assert!(record.output_truncated);
        assert!(record.stdout.len() + record.stderr.len() <= 4096);
        assert!(owner.records().len() <= 3);
        assert!(owner.admitted_bytes() <= SESSION_BYTES);
    }
    let retained: usize = owner
        .records()
        .iter()
        .map(|r| r.stdout.capacity() + r.stderr.capacity())
        .sum();
    assert!(retained <= 65536);
    println!(
        "MEASURE real Git large blob: producer_bytes=262144 job_cap=4096 retained_stream_capacity={retained} records={}",
        owner.records().len()
    );
    Ok(())
}

fn malformed() -> Result<()> {
    let repository = Repository::new("sha256");
    repository.commit();
    let mut owner = repository.owner("");
    assert!(repository.open(&mut owner, &repository.root).published);
    let retained = owner.contexts()[0].observation.clone();
    // Installed Git cannot emit corrupt porcelain deterministically. This
    // explicit parser-boundary double proves refusal, not real Git corruption.
    fs::write(
        &repository.program,
        "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$2\" = status ]; then printf 'invalid record\\000'; exit 0; fi\nexec /usr/bin/git \"$@\"\n",
    )?;
    let id = owner.refresh(retained.context)?;
    assert!(!finish(&mut owner).published);
    assert!(Arc::ptr_eq(
        &retained,
        &owner.context(retained.context).unwrap().observation
    ));
    assert_eq!(owner.context(retained.context).unwrap().failure, Some(id));
    assert_eq!(owner.record(id).unwrap().outcome, Outcome::Failed);
    assert!(!owner.busy());
    println!(
        "PASS malformed parser boundary: prior confirmed observation retained, explicit failed freshness"
    );
    Ok(())
}
