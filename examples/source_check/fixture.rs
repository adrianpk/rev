use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) struct Fixture(pub PathBuf);

impl Fixture {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..32 {
            let serial = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("rev-source-check-{}-{serial}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("cannot create owned source-check fixture: {error}"),
            }
        }
        panic!("cannot allocate an exclusive source-check fixture");
    }

    pub fn write(&self, path: &str, source: &str) -> PathBuf {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        path
    }

    pub fn workspace(&self) {
        for (path, source) in [
            (
                "Cargo.toml",
                "[workspace]\nmembers=['sdk']\nresolver='3'\n[package]\nname='z-editor'\nversion='0.1.0'\nedition='2024'\n[[example]]\nname='custom'\npath='tools/check.rs'\n",
            ),
            ("src/lib.rs", "#![forbid(unsafe_code)]\npub mod child;\n"),
            ("src/child.rs", "pub fn value() -> u8 { 1 }\n"),
            ("tools/check.rs", "#![forbid(unsafe_code)]\nfn main() {}\n"),
            ("tests/editor.rs", "#![forbid(unsafe_code)]\n"),
            ("benches/work.rs", "#![forbid(unsafe_code)]\nfn main() {}\n"),
            ("build.rs", "#![forbid(unsafe_code)]\nfn main() {}\n"),
            (
                "sdk/Cargo.toml",
                "[package]\nname='a-sdk'\nversion='0.1.0'\nedition='2024'\n",
            ),
            ("sdk/src/lib.rs", "pub mod child;\n"),
            (
                "sdk/src/child.rs",
                "pub fn pointer() -> *const u8 { std::ptr::null() }\n",
            ),
            ("sdk/tests/client.rs", "#![forbid(unsafe_code)]\n"),
            (
                "vendor/foreign/Cargo.toml",
                "[package]\nname='foreign'\nversion='0.1.0'\n",
            ),
            ("vendor/foreign/src/lib.rs", "unsafe fn foreign() {}\n"),
            ("target/generated.rs", "not Rust"),
            (".git/hidden.rs", "not Rust"),
            (
                "Cargo.lock",
                "version = 4\n[[package]]\nname = 'a-sdk'\nversion = '0.1.0'\n[[package]]\nname = 'z-editor'\nversion = '0.1.0'\n",
            ),
        ] {
            self.write(path, source);
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
