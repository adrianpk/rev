use crate::support::{Result, children, key, peak_rss, text};
use crossterm::event::KeyCode;
use rev::{
    command::{Command, CompletionCommand, History, Motion},
    session::{CompletionSource, Session},
};
use std::time::{Duration, Instant};

pub(crate) struct Sample {
    pub disk: &'static str,
    pub unsaved: &'static str,
    label: &'static str,
    needle: &'static str,
}

pub(crate) const RUST: Sample = Sample {
    disk: "struct Widget { disk_field: u32 }\nfn main() { let item = Widget { disk_field: 1 }; let _ = item.disk_field; }\n",
    unsaved: "struct Widget { unsaved_field: u32 }\nfn unsaved_native_marker() {}\nfn main() { let item = Widget { unsaved_field: 1 }; let _ = item.uns; }\n",
    label: "unsaved_field",
    needle: "item.uns",
};
pub(crate) const GO: Sample = Sample {
    disk: "package main\ntype Widget struct { DiskField int }\nfunc main() { item := Widget{DiskField: 1}; _ = item.DiskField }\n",
    unsaved: "package main\ntype Widget struct { UnsavedField int }\nfunc UnsavedNativeMarker() {}\nfunc main() { item := Widget{UnsavedField: 1}; _ = item.Uns }\n",
    label: "UnsavedField",
    needle: "item.Uns",
};

// Actual installed servers must use the unsaved document and yield one semantic
// provider. Acceptance is one history operation; undo and disk bytes are exact.
pub(crate) fn run(s: &mut Session, sample: &Sample) -> Result<()> {
    document(s, sample)?;
    let started = Instant::now();
    ready(s, sample.label)?;
    assert!(matches!(
        s.completion_source(),
        Some(CompletionSource::Semantic { .. })
    ));
    accept(s, sample)?;
    for pid in children()? {
        println!(
            "MEASURE active server: field={} peak_rss={}",
            sample.label,
            peak_rss(&pid)?
        );
    }
    println!(
        "PASS actual semantic field={} readiness_and_edit_ms={}; unsaved-only field, exclusive source, one undo",
        sample.label,
        started.elapsed().as_millis()
    );
    Ok(())
}

pub(crate) fn settle(s: &mut Session, deadline: Instant) -> Result<()> {
    loop {
        s.expire_input(Instant::now());
        s.poll_frame()?;
        if s.remaining_input(Instant::now()).is_none() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("semantic work exceeded its deadline".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn ready(s: &mut Session, label: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        s.execute(Command::Completion(CompletionCommand::AtPoint));
        settle(s, deadline)?;
        if matches!(
            s.completion_source(),
            Some(CompletionSource::Semantic { .. })
        ) && s
            .completion()
            .is_some_and(|p| p.candidates().iter().any(|c| c.text() == label))
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "semantic {label} expired: {:?}; {}",
                s.lsp_status(),
                s.echo()
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn document(s: &mut Session, sample: &Sample) -> Result<()> {
    let before = text(s);
    s.replace(0..before.chars().count(), sample.unsaved)?;
    s.execute(Command::Motion(Motion::BufferStart));
    let point = sample
        .unsaved
        .find(sample.needle)
        .ok_or("missing semantic point")?
        + sample.needle.len();
    for _ in 0..point {
        s.execute(Command::Motion(Motion::ForwardChar));
    }
    Ok(())
}

fn accept(s: &mut Session, sample: &Sample) -> Result<()> {
    let labels: Vec<_> = s
        .completion()
        .ok_or("missing semantic popup")?
        .candidates()
        .iter()
        .map(|c| c.text().to_owned())
        .collect();
    assert_eq!(labels.iter().filter(|s| *s == sample.label).count(), 1);
    assert!(!labels.iter().any(|s| {
        [
            "disk_field",
            "DiskField",
            "unsaved_native_marker",
            "UnsavedNativeMarker",
        ]
        .contains(&s.as_str())
    }));
    let selected = labels
        .iter()
        .position(|s| s == sample.label)
        .ok_or("missing field")?;
    for _ in 0..selected {
        key(s, KeyCode::Down);
    }
    let history = s.buffer()?.history().len();
    key(s, KeyCode::Tab);
    settle(s, Instant::now() + Duration::from_secs(5))?;
    let expected = sample
        .unsaved
        .replace(sample.needle, &format!("item.{}", sample.label));
    assert_eq!(text(s), expected);
    assert_eq!(s.buffer()?.history().len(), history + 1);
    s.execute(Command::History(History::Undo));
    assert_eq!(text(s), sample.unsaved);
    s.execute(Command::Completion(CompletionCommand::Cancel));
    Ok(())
}
