use super::*;
use crate::{fixture::Fixture, org::Cancellation};
use operation::Output;
use std::{
    fs::File,
    io,
    path::Path,
    time::{Duration, Instant},
};
fn context(f: &Fixture) -> Context {
    Context {
        base: f.0.clone(),
        dir: Some("assets".into()),
        id: None,
        policy: crate::config::Org::default(),
    }
}
fn plan(f: &Fixture, method: Method) -> Plan {
    Plan::prepare(context(f), "input.bin", method, &Cancellation::default()).unwrap()
}
struct Fault {
    step: &'static str,
    syncs: usize,
}
impl Output for Fault {
    fn write(&mut self, file: &mut File, bytes: &[u8]) -> io::Result<()> {
        if self.step == "write" {
            Err(io::Error::other("injected write failure"))
        } else {
            std::io::Write::write_all(file, bytes)
        }
    }
    fn sync_directory(&mut self, file: &File) -> io::Result<()> {
        self.syncs += 1;
        if (self.step == "destination sync" && self.syncs == 1)
            || (self.step == "source sync" && self.syncs == 2)
        {
            Err(io::Error::other("injected sync failure"))
        } else {
            file.sync_all()
        }
    }
    fn remove_input(&mut self, path: &Path) -> io::Result<()> {
        if self.step == "remove" {
            Err(io::Error::other("injected input removal failure"))
        } else {
            std::fs::remove_file(path)
        }
    }
}
// Real publication and observations remain native while the narrow write/sync/
// unlink boundary fails. Effects and source retention reflect the actual files.
#[test]
fn attachments_partial_effects() {
    for (step, effect, source_exists, target) in [
        ("write", Effect::Rejected, true, b"old".as_slice()),
        (
            "destination sync",
            Effect::Published,
            true,
            b"new".as_slice(),
        ),
        ("remove", Effect::Published, true, b"new".as_slice()),
        ("source sync", Effect::Removed, false, b"new".as_slice()),
    ] {
        let f = Fixture::new();
        let source = f.write("input.bin", "new");
        let destination = f.write("assets/input.bin", "old");
        let result = plan(&f, Method::Move)
            .run_with(&Cancellation::default(), &mut Fault { step, syncs: 0 });
        assert_eq!(result.effect, effect, "{step}: {:?}", result.error);
        assert!(result.error.is_some(), "{step}");
        assert_eq!(source.exists(), source_exists, "{step}");
        assert_eq!(std::fs::read(destination).unwrap(), target, "{step}");
        assert_eq!(
            std::fs::read_dir(f.0.join("assets")).unwrap().count(),
            1,
            "owned staging name leaked: {step}"
        );
    }
}
// Changed inputs, newly competing destinations and replaced directory owners
// reject before publication. An external actor's bytes are never overwritten.
#[test]
fn attachments_revalidation() {
    for scenario in [
        "input",
        "new collision",
        "changed collision",
        "directory alias",
    ] {
        let f = Fixture::new();
        let source = f.write("input.bin", "input");
        std::fs::create_dir(f.0.join("assets")).unwrap();
        if scenario == "changed collision" {
            f.write("assets/input.bin", "old");
        }
        let p = plan(&f, Method::Copy);
        match scenario {
            "input" => {
                std::fs::rename(f.write("replacement.bin", "external input"), &source).unwrap();
            }
            "new collision" | "changed collision" => {
                f.write("assets/input.bin", "external target");
            }
            _ => {
                std::fs::rename(f.0.join("assets"), f.0.join("previous")).unwrap();
                std::fs::create_dir(f.0.join("outside")).unwrap();
                std::os::unix::fs::symlink(f.0.join("outside"), f.0.join("assets")).unwrap();
            }
        }
        let before = std::fs::read(f.0.join("assets/input.bin")).ok();
        let result = p.run(&Cancellation::default());
        assert_eq!(result.effect, Effect::Rejected, "{scenario}");
        assert!(result.error.is_some(), "{scenario}");
        assert_eq!(
            std::fs::read(f.0.join("assets/input.bin")).ok(),
            before,
            "{scenario}"
        );
        assert!(source.exists());
    }
}
struct CancelAfterPublish<'a>(&'a Cancellation);
impl Output for CancelAfterPublish<'_> {
    fn sync_directory(&mut self, file: &File) -> io::Result<()> {
        file.sync_all()?;
        self.0.cancel();
        Ok(())
    }
}
// Cancellation after durable publication must survive worker reaping as an
// actual copied-file effect, with the move input still present and no idle job.
#[test]
fn attachments_late_cancel() {
    let f = Fixture::new();
    f.write("input.bin", "payload");
    let buffers = crate::buffer::Buffers::new().unwrap();
    let stamp = buffers.get(buffers.initial()).unwrap().stamp();
    let p = plan(&f, Method::Move);
    let mut job = crate::org::Job::default();
    job.start(stamp, move |cancel| {
        p.run_with(cancel, &mut CancelAfterPublish(cancel))
    })
    .unwrap();
    let end = Instant::now() + Duration::from_secs(2);
    let result = loop {
        if let Some(result) = job.poll_effect() {
            break result.unwrap().1;
        }
        assert!(Instant::now() < end);
        std::thread::yield_now();
    };
    assert_eq!(result.effect, Effect::Published);
    assert!(result.error.unwrap().contains("cancelled"));
    assert_eq!(
        std::fs::read(f.0.join("assets/input.bin")).unwrap(),
        b"payload"
    );
    assert_eq!(std::fs::read(f.0.join("input.bin")).unwrap(), b"payload");
    assert!(!job.is_active());
}
