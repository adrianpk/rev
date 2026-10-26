use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use rev::file::{Output, Publication};

use super::*;

struct Changed {
    target: PathBuf,
    other: PathBuf,
    change: &'static str,
    calls: Vec<&'static str>,
}

impl Output for Changed {
    fn write(&mut self, save: &rev::buffer::Save, file: &mut File) -> io::Result<()> {
        self.calls.push("write");
        save.write(file)
    }

    fn flush(&mut self, file: &mut File) -> io::Result<()> {
        self.calls.push("flush");
        file.flush()?;
        match self.change {
            "content" => fs::write(&self.target, b"external replacement"),
            "missing" => fs::remove_file(&self.target),
            _ => {
                fs::remove_file(&self.target)?;
                symlink(&self.other, &self.target)
            }
        }
    }

    fn sync_file(&mut self, file: &File) -> io::Result<()> {
        self.calls.push("sync file");
        file.sync_all()
    }

    fn replace(&mut self, temporary: &Path, target: &Path) -> io::Result<()> {
        self.calls.push("replace");
        fs::rename(temporary, target)
    }

    fn remove(&mut self, temporary: &Path) -> io::Result<()> {
        self.calls.push("cleanup");
        fs::remove_file(temporary)
    }
}

// A real external change during temporary staging must be observed before
// replacement. Only the timing hook is controlled; files, flush and sync are real.
#[test]
fn changed_staging() {
    for change in ["content", "missing", "symlink"] {
        let fixture = Fixture::new();
        let target = fixture.file("target.txt", b"original");
        let other = fixture.file("other.txt", b"outside");
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&target).unwrap();
        edit(buffers.get_mut(id).unwrap(), "retained ");
        let before = snapshot(buffers.get(id).unwrap());
        let save = buffers.prepare_save(id, None).unwrap();
        let mut output = Changed {
            target: target.clone(),
            other: other.clone(),
            change,
            calls: Vec::new(),
        };
        let outcome = buffers.publish_save_with(save, &mut output);
        assert_eq!(outcome.publication(), Publication::NotPublished, "{change}");
        assert!(outcome.error().is_some());
        assert_eq!(output.calls, ["write", "flush", "sync file", "cleanup"]);
        assert_eq!(snapshot(buffers.get(id).unwrap()), before);
        match change {
            "content" => assert_eq!(fs::read(&target).unwrap(), b"external replacement"),
            "missing" => assert!(!target.exists()),
            _ => assert!(fs::symlink_metadata(&target).unwrap().is_symlink()),
        }
        assert_eq!(fs::read(&other).unwrap(), b"outside");
        assert!(fs::read_dir(&fixture.0).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".rev-save-")
        }));
    }
}

// Replacing the captured directory with another directory at the same path
// cannot redirect a pending tree name request to that unapproved owner.
#[test]
fn replaced_directory() {
    use rev::tree::{CreateKind, Tree};
    for kind in [CreateKind::File, CreateKind::Directory] {
        let fixture = Fixture::new();
        let root = fixture.0.join("chosen");
        let prior = fixture.0.join("retained");
        fs::create_dir(&root).unwrap();
        let tree = Tree::new(root.clone());
        let creation = tree.prepare_create(kind).unwrap();
        fs::rename(&root, &prior).unwrap();
        fs::create_dir(&root).unwrap();
        let error = creation.publish("new").unwrap_err();
        assert!(error.to_string().contains("changed"));
        assert!(error.to_string().contains(root.to_str().unwrap()));
        assert!(!root.join("new").exists());
        assert!(!prior.join("new").exists());
    }
}
