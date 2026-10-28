use super::*;

use std::sync::{Arc, Barrier};

#[path = "../../../tests/configuration/fixture.rs"]
mod fixture;
use fixture::Fixture;

struct Fault {
    step: &'static str,
    removals: usize,
}

fn checkpoint(actual: &str, selected: &str) -> io::Result<()> {
    if actual == selected {
        return Err(io::Error::other(format!("injected {actual}")));
    }
    Ok(())
}

impl crate::file::Output for Fault {
    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        checkpoint("flush", self.step)?;
        file.flush()
    }

    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        checkpoint("sync file", self.step)?;
        file.sync_all()
    }

    fn sync_directory(&mut self, directory: &File) -> io::Result<()> {
        checkpoint("sync directory", self.step)?;
        directory.sync_all()
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        self.removals += 1;
        if self.removals == 1 {
            checkpoint("remove", self.step)?;
        }
        fs::remove_file(path)
    }
}

impl Output for Fault {
    fn template(&mut self, file: &mut File) -> io::Result<()> {
        file.write_all(b"partial")?;
        if self.step == "unwind" {
            panic!("injected interrupted write");
        }
        checkpoint("write", self.step)?;
        file.set_len(0)?;
        std::io::Seek::rewind(file)?;
        file.write_all(default_template().as_bytes())
    }

    fn link(&mut self, temporary: &Path, root: &Path) -> io::Result<()> {
        checkpoint("link", self.step)?;
        fs::hard_link(temporary, root)
    }
}

// Every fallible boundary reports its stage. Pre-publication faults expose no
// destination; later faults retain only a complete file and never a temp leak.
#[test]
fn publication_faults() {
    for step in [
        "write",
        "flush",
        "sync file",
        "link",
        "remove",
        "sync directory",
    ] {
        let fixture = Fixture::new();
        let root = fixture.0.join("user/config.toml");
        let mut output = Fault { step, removals: 0 };
        let error = publish(&root, &mut output).unwrap_err();
        assert!(
            error.to_string().contains(&format!("injected {step}")),
            "{error}"
        );
        let published = ["remove", "sync directory"].contains(&step);
        assert_eq!(root.exists(), published, "{step}");
        assert_eq!(
            fs::read_dir(root.parent().unwrap()).unwrap().count(),
            usize::from(published)
        );
        if published {
            assert_eq!(fs::read(&root).unwrap(), default_template().as_bytes());
        }
    }
}

// Unwinding after a short write drops the existing native temp owner before any
// destination is visible. This proves cleanup beyond ordinary Result errors.
#[test]
fn interrupted_write() {
    let fixture = Fixture::new();
    let retained = fixture.write("user/retained.toml", "editor.tab_width = 8");
    let root = fixture.0.join("user/config.toml");
    let result = std::panic::catch_unwind(|| {
        publish(
            &root,
            &mut Fault {
                step: "unwind",
                removals: 0,
            },
        )
    });
    assert!(result.is_err());
    assert!(!root.exists());
    assert_eq!(fs::read_dir(root.parent().unwrap()).unwrap().count(), 1);
    assert_eq!(
        fs::read_to_string(retained).unwrap(),
        "editor.tab_width = 8"
    );
}

struct Race(Arc<Barrier>);
impl crate::file::Output for Race {}
impl Output for Race {
    fn link(&mut self, temporary: &Path, root: &Path) -> io::Result<()> {
        self.0.wait();
        fs::hard_link(temporary, root)
    }
}

// Synchronize four real prepared publishers at the no-clobber syscall. Every
// contender succeeds, readers see a complete file, and all private names retire.
#[test]
fn simultaneous_publishers() {
    let fixture = Fixture::new();
    let root = fixture.0.join("user/config.toml");
    let barrier = Arc::new(Barrier::new(4));
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                let root = &root;
                scope.spawn(move || {
                    publish(root, &mut Race(barrier)).unwrap();
                    assert_eq!(fs::read(root).unwrap(), default_template().as_bytes());
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
    assert_eq!(fs::read_dir(root.parent().unwrap()).unwrap().count(), 1);
}

struct Winner(&'static str);
impl crate::file::Output for Winner {}
impl Output for Winner {
    fn link(&mut self, temporary: &Path, root: &Path) -> io::Result<()> {
        fs::write(root, self.0)?;
        fs::hard_link(temporary, root)
    }
}

// A winner appearing after staging keeps its exact custom or invalid bytes.
// The losing publisher delegates semantics to the normal loader and resolver.
#[test]
fn concurrent_winner() {
    for (text, width, rejected) in [
        ("editor.indent_width = 2", 2, false),
        ("editor.indent_width = 0", 4, true),
    ] {
        let fixture = Fixture::new();
        let root = fixture.0.join("user/config.toml");
        publish(&root, &mut Winner(text)).unwrap();
        let startup = super::super::prepare(&root);
        assert_eq!(startup.snapshot.settings().editor.indent_width, width);
        assert_eq!(startup.diagnostic.is_some(), rejected);
        assert_eq!(fs::read_to_string(&root).unwrap(), text);
        assert_eq!(fs::read_dir(root.parent().unwrap()).unwrap().count(), 1);
    }
}
